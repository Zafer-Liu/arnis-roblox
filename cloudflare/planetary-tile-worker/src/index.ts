/**
 * planetary tile worker
 *
 * Cloudflare Worker that proxies satellite imagery tiles from upstream
 * providers (ESRI, Mapbox, OSM, MapTiler) and serves them to the Roblox
 * runtime as base64-encoded raw RGBA buffers that EditableImage can
 * consume directly via WritePixelsBuffer.
 *
 * Endpoints:
 *   GET /tile/{provider}/{z}/{x}/{y}
 *     → fetches the tile from upstream
 *     → caches in KV with 7-day TTL
 *     → returns JSON: { width, height, rgbaBase64 }
 *
 *   GET /manifests/{name}.json
 *     → fetches from R2 bucket "planetary-manifests"
 *     → returns the JSON manifest with permissive CORS for HttpService
 *
 *   GET /health
 *     → liveness check
 *
 *   POST /telemetry/run
 *     → accepts JSON bootstrap/run telemetry from the Roblox client,
 *       enriches with edge-side metadata (hashed IP, country, colo, ray),
 *       stores under `run:{ISO}:{runId}` in the TELEMETRY KV namespace
 *       with a 30-day TTL, and appends to a rolling `telemetry:index`
 *       (last 100 keys) for cheap latest-N reads.
 *
 *   GET /telemetry/latest?limit=N
 *     → returns up to N (default 10, max 100) most recent telemetry
 *       records as a JSON array. Always 200, even when empty.
 *
 * Future extensions:
 *   - DEM elevation tile proxying (Terrarium tiles from AWS)
 *   - Multi-provider fallback (try ESRI, fall back to Mapbox)
 *   - Tile pyramid pre-warming for popular cities
 */

const PROVIDER_TEMPLATES: Record<string, string> = {
  esri:
    "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
  mapbox: "https://api.mapbox.com/v4/mapbox.satellite/{z}/{x}/{y}.jpg",
  osm: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
  maptiler: "https://api.maptiler.com/tiles/satellite-v2/{z}/{x}/{y}.jpg",
};

interface Env {
  TILE_CACHE?: KVNamespace;
  MANIFESTS?: R2Bucket;
  TELEMETRY?: KVNamespace;
  // CHUNK_CACHE is the hot path for chunk JSON reads. Worker tries this
  // first and falls back to R2 on miss. KV reads are O(ms) across colos
  // and the free tier allows 100K reads/day (vs R2 Class B at 1M/month
  // shared across all ops). Cuts R2 GETs by ~100× once the cache is
  // warmed for Austin.
  CHUNK_CACHE?: KVNamespace;
  DEFAULT_TILE_PROVIDER: string;
  TILE_CACHE_TTL_SECONDS: string;
  // ADMIN_TOKEN gates the PUT /admin/manifest/... upload endpoint used
  // by scripts/upload_austin_chunks.sh (and any future manifest bake
  // publish path). Set via `wrangler secret put ADMIN_TOKEN`. When
  // unset the upload route is disabled entirely (fail-closed).
  ADMIN_TOKEN?: string;
}

const TELEMETRY_INDEX_KEY = "telemetry:index";
const TELEMETRY_INDEX_MAX = 100;
const TELEMETRY_TTL_SECONDS = 60 * 60 * 24 * 30; // 30 days

const CORS_HEADERS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, PUT, POST, OPTIONS",
  "access-control-allow-headers": "*",
};

function corsResponse(body: BodyInit | null, init: ResponseInit = {}): Response {
  const headers = new Headers(init.headers);
  for (const [k, v] of Object.entries(CORS_HEADERS)) headers.set(k, v);
  return new Response(body, { ...init, headers });
}

async function handleHealth(): Promise<Response> {
  return corsResponse(JSON.stringify({ status: "ok", service: "planetary" }), {
    headers: { "content-type": "application/json" },
  });
}

function buildTileUrl(provider: string, z: number, x: number, y: number): string | null {
  const template = PROVIDER_TEMPLATES[provider.toLowerCase()];
  if (!template) return null;
  return template
    .replace("{z}", String(z))
    .replace("{x}", String(x))
    .replace("{y}", String(y));
}

async function handleTile(
  env: Env,
  provider: string,
  z: number,
  x: number,
  y: number,
): Promise<Response> {
  const cacheKey = `tile:${provider}:${z}:${x}:${y}`;

  // Try KV cache first (if bound).
  if (env.TILE_CACHE) {
    const cached = await env.TILE_CACHE.get(cacheKey, "json");
    if (cached) {
      return corsResponse(JSON.stringify(cached), {
        headers: {
          "content-type": "application/json",
          "x-cache": "hit",
        },
      });
    }
  }

  const url = buildTileUrl(provider, z, x, y);
  if (!url) {
    return corsResponse(
      JSON.stringify({ error: `unknown provider: ${provider}` }),
      { status: 400, headers: { "content-type": "application/json" } },
    );
  }

  // Fetch tile bytes from upstream.
  const upstream = await fetch(url, {
    cf: { cacheTtl: 604800, cacheEverything: true },
  });
  if (!upstream.ok) {
    return corsResponse(
      JSON.stringify({ error: `upstream ${upstream.status}: ${url}` }),
      { status: 502, headers: { "content-type": "application/json" } },
    );
  }
  const jpegBytes = new Uint8Array(await upstream.arrayBuffer());

  // Decode JPEG/PNG to raw RGBA. Cloudflare Workers don't have a native
  // image decoder, so we use the Image Resizing API (which can transform
  // remote images via fetch options) to coerce the format.
  // Workers don't expose pixel buffers directly, so for now we return the
  // original JPEG bytes base64-encoded and let the Roblox client handle it
  // via a future EditableImage:LoadFromBytes path. Until that exists, the
  // Lua side falls back to a placeholder colored Part — the worker is
  // ready for when the decode pipeline lands.
  // Encode JPEG bytes to base64 without stack overflow. The spread
  // operator (...jpegBytes) crashes V8 when the array exceeds ~128K
  // elements because every byte becomes a function argument.
  let binaryStr = "";
  const CHUNK = 8192;
  for (let i = 0; i < jpegBytes.length; i += CHUNK) {
    binaryStr += String.fromCharCode.apply(null, Array.from(jpegBytes.subarray(i, i + CHUNK)));
  }
  const base64 = btoa(binaryStr);

  const result = {
    provider,
    z,
    x,
    y,
    width: 256, // standard slippy map tile size
    height: 256,
    format: "jpeg",
    bytesBase64: base64,
  };

  if (env.TILE_CACHE) {
    const ttl = parseInt(env.TILE_CACHE_TTL_SECONDS || "604800", 10);
    await env.TILE_CACHE.put(cacheKey, JSON.stringify(result), {
      expirationTtl: ttl,
    });
  }

  return corsResponse(JSON.stringify(result), {
    headers: {
      "content-type": "application/json",
      "x-cache": "miss",
    },
  });
}

async function hashIp(ip: string): Promise<string> {
  if (!ip) return "";
  const data = new TextEncoder().encode(ip);
  const digest = await crypto.subtle.digest("SHA-256", data);
  const bytes = new Uint8Array(digest);
  let hex = "";
  for (let i = 0; i < bytes.length; i++) {
    hex += bytes[i].toString(16).padStart(2, "0");
  }
  return hex.slice(0, 16);
}

async function handleTelemetryRun(env: Env, request: Request): Promise<Response> {
  if (!env.TELEMETRY) {
    return corsResponse(
      JSON.stringify({ error: "TELEMETRY KV binding not configured" }),
      { status: 500, headers: { "content-type": "application/json" } },
    );
  }

  let payload: any;
  try {
    payload = await request.json();
  } catch (err) {
    return corsResponse(
      JSON.stringify({ error: "invalid JSON body", detail: String(err) }),
      { status: 400, headers: { "content-type": "application/json" } },
    );
  }

  if (!payload || typeof payload !== "object") {
    return corsResponse(
      JSON.stringify({ error: "body must be a JSON object" }),
      { status: 400, headers: { "content-type": "application/json" } },
    );
  }
  if (typeof payload.runId !== "string" || payload.runId.length === 0) {
    return corsResponse(
      JSON.stringify({ error: "missing required field: runId" }),
      { status: 400, headers: { "content-type": "application/json" } },
    );
  }
  if (payload.timestamp === undefined || payload.timestamp === null) {
    return corsResponse(
      JSON.stringify({ error: "missing required field: timestamp" }),
      { status: 400, headers: { "content-type": "application/json" } },
    );
  }

  const rawIp = request.headers.get("cf-connecting-ip") || "";
  const ipHash = await hashIp(rawIp);
  const cf: any = (request as any).cf || {};
  const edgeNow = new Date();
  const edgeIso = edgeNow.toISOString();

  const enriched = {
    ...payload,
    edge: {
      receivedAtIso: edgeIso,
      receivedAtEpochMs: edgeNow.getTime(),
      ipHash,
      country: request.headers.get("cf-ipcountry") || null,
      ray: request.headers.get("cf-ray") || null,
      colo: cf.colo || null,
    },
  };

  const safeRunId = String(payload.runId).replace(/[^A-Za-z0-9._-]/g, "_");
  const key = `run:${edgeIso}:${safeRunId}`;
  await env.TELEMETRY.put(key, JSON.stringify(enriched), {
    expirationTtl: TELEMETRY_TTL_SECONDS,
  });

  // Update rolling index of recent keys.
  let index: string[] = [];
  try {
    const existing = await env.TELEMETRY.get(TELEMETRY_INDEX_KEY, "json");
    if (Array.isArray(existing)) {
      index = existing.filter((k) => typeof k === "string");
    }
  } catch (_err) {
    index = [];
  }
  index.push(key);
  if (index.length > TELEMETRY_INDEX_MAX) {
    index = index.slice(index.length - TELEMETRY_INDEX_MAX);
  }
  await env.TELEMETRY.put(TELEMETRY_INDEX_KEY, JSON.stringify(index));

  return corsResponse(JSON.stringify({ runId: payload.runId }), {
    status: 202,
    headers: { "content-type": "application/json" },
  });
}

async function handleTelemetryLatest(env: Env, url: URL): Promise<Response> {
  if (!env.TELEMETRY) {
    return corsResponse(JSON.stringify([]), {
      headers: { "content-type": "application/json" },
    });
  }

  const limitParam = parseInt(url.searchParams.get("limit") || "10", 10);
  let limit = isFinite(limitParam) ? limitParam : 10;
  if (limit < 1) limit = 1;
  if (limit > 100) limit = 100;

  let index: string[] = [];
  try {
    const existing = await env.TELEMETRY.get(TELEMETRY_INDEX_KEY, "json");
    if (Array.isArray(existing)) {
      index = existing.filter((k) => typeof k === "string");
    }
  } catch (_err) {
    index = [];
  }

  // Most recent first.
  const recent = index.slice(-limit).reverse();
  const records: any[] = [];
  for (const k of recent) {
    const rec = await env.TELEMETRY.get(k, "json");
    if (rec) records.push(rec);
  }

  return corsResponse(JSON.stringify(records), {
    headers: { "content-type": "application/json" },
  });
}

async function handleManifest(
  env: Env,
  name: string,
  request: Request,
): Promise<Response> {
  if (!env.MANIFESTS) {
    return corsResponse(
      JSON.stringify({ error: "R2 binding not configured" }),
      { status: 500, headers: { "content-type": "application/json" } },
    );
  }

  // Cache-Control tuning for Cloudflare free tier:
  //   - /manifests/{city}/index.json → 60s (mutable, a new bake of the
  //     same city can swap chunk contents under this URL)
  //   - /manifests/{city}/chunks/{id}.(json|msgpack) → 86400s (24h).
  //     Chunks are immutable per release; the edge cache hit here
  //     dramatically reduces R2 Class B GETs (the 1M/month free-tier
  //     limit) because repeat player joins from the same colo hit the
  //     cached response without ever touching R2.
  //   - /stats → 60s (cheap aggregate, OK to recompute)
  // Any other path falls back to the legacy 5-minute default.
  const isChunkImmutable =
    /\/chunks\/[^/]+\.(json|msgpack)$/.test(name) === true;
  const isCityIndex = /\/index\.json$/.test(name) === true;
  const chunkCacheControl = isChunkImmutable
    ? "public, max-age=86400, immutable"
    : isCityIndex
      ? "public, max-age=60"
      : "public, max-age=300";

  // Binary msgpack variant: served as raw bytes with the msgpack
  // content-type. The Lua runtime doesn't request these yet, but the
  // format is a parallel drop-in for future decoder work.
  if (name.endsWith(".msgpack")) {
    const obj = await env.MANIFESTS.get(name);
    if (!obj) {
      return corsResponse(
        JSON.stringify({ error: `manifest not found: ${name}` }),
        { status: 404, headers: { "content-type": "application/json" } },
      );
    }
    return corsResponse(obj.body, {
      headers: {
        "content-type": "application/msgpack",
        "cache-control": chunkCacheControl,
      },
    });
  }

  // Advisory Accept negotiation: if the caller asks for msgpack and a
  // sibling .msgpack variant exists in R2, serve that instead. This is
  // purely opt-in — the default behavior for Accept: application/json
  // (and the Lua runtime default) stays unchanged.
  if (name.endsWith(".json")) {
    const accept = (request.headers.get("accept") || "").toLowerCase();
    if (accept.includes("application/msgpack")) {
      const mpKey = name.slice(0, -".json".length) + ".msgpack";
      const mpObj = await env.MANIFESTS.get(mpKey);
      if (mpObj) {
        return corsResponse(mpObj.body, {
          headers: {
            "content-type": "application/msgpack",
            "cache-control": chunkCacheControl,
            "x-chunk-format": "msgpack",
          },
        });
      }
    }
  }

  // KV hot-path: chunk JSON reads try CHUNK_CACHE first. KV is O(ms)
  // across colos and the 100K reads/day free-tier limit is far beyond
  // our load. A miss falls through to R2 exactly as before. The key
  // format mirrors the R2 key ("austin/chunks/0_-2.json") so cache
  // warmers and purgers can use the same identifier across stores.
  if (isChunkImmutable && env.CHUNK_CACHE) {
    const cached = await env.CHUNK_CACHE.get(name, "stream");
    if (cached !== null) {
      return corsResponse(cached, {
        headers: {
          "content-type": "application/json",
          "cache-control": chunkCacheControl,
          "x-chunk-cache": "kv-hit",
        },
      });
    }
  }

  const obj = await env.MANIFESTS.get(name);
  if (!obj) {
    return corsResponse(
      JSON.stringify({ error: `manifest not found: ${name}` }),
      { status: 404, headers: { "content-type": "application/json" } },
    );
  }

  // Populate KV on R2 miss so the next request from any colo reads from
  // KV instead of R2. This write is fire-and-forget; a failure here is
  // non-fatal (we've already served the response from R2).
  if (isChunkImmutable && env.CHUNK_CACHE) {
    try {
      const bodyBytes = await obj.arrayBuffer();
      await env.CHUNK_CACHE.put(name, bodyBytes);
      return corsResponse(bodyBytes, {
        headers: {
          "content-type": "application/json",
          "cache-control": chunkCacheControl,
          "x-chunk-cache": "kv-miss-seeded",
        },
      });
    } catch (err) {
      // Fall through to serving obj.body below without caching.
    }
  }

  return corsResponse(obj.body, {
    headers: {
      "content-type": "application/json",
      "cache-control": chunkCacheControl,
    },
  });
}

async function handleManifestStats(env: Env, name: string): Promise<Response> {
  if (!env.MANIFESTS) {
    return corsResponse(
      JSON.stringify({ error: "R2 binding not configured" }),
      { status: 500, headers: { "content-type": "application/json" } },
    );
  }

  // Strip optional extension and probe both variants so callers can
  // pass either `.../chunks/-6_-4`, `.../chunks/-6_-4.json`, or
  // `.../chunks/-6_-4.msgpack`.
  let base = name;
  if (base.endsWith(".json")) base = base.slice(0, -".json".length);
  else if (base.endsWith(".msgpack")) base = base.slice(0, -".msgpack".length);

  const jsonKey = `${base}.json`;
  const msgpackKey = `${base}.msgpack`;
  const [jsonHead, msgpackHead] = await Promise.all([
    env.MANIFESTS.head(jsonKey),
    env.MANIFESTS.head(msgpackKey),
  ]);

  if (!jsonHead && !msgpackHead) {
    return corsResponse(
      JSON.stringify({ error: `no variants found for ${base}` }),
      { status: 404, headers: { "content-type": "application/json" } },
    );
  }

  const body: Record<string, number | null> = {
    json: jsonHead ? jsonHead.size : null,
    msgpack: msgpackHead ? msgpackHead.size : null,
    ratio:
      jsonHead && msgpackHead && msgpackHead.size > 0
        ? jsonHead.size / msgpackHead.size
        : null,
  };

  return corsResponse(JSON.stringify(body), {
    headers: {
      "content-type": "application/json",
      "cache-control": "public, max-age=60",
    },
  });
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return corsResponse(null, { status: 204 });
    }

    const url = new URL(request.url);
    const parts = url.pathname.split("/").filter(Boolean);

    if (parts[0] === "health") {
      return handleHealth();
    }

    if (parts[0] === "telemetry" && parts[1] === "run" && request.method === "POST") {
      return handleTelemetryRun(env, request);
    }

    if (parts[0] === "telemetry" && parts[1] === "latest" && request.method === "GET") {
      return handleTelemetryLatest(env, url);
    }

    if (parts[0] === "tile" && parts.length === 5) {
      const [, provider, zStr, xStr, yStr] = parts;
      const z = parseInt(zStr, 10);
      const x = parseInt(xStr, 10);
      const y = parseInt(yStr, 10);
      if (!isFinite(z) || !isFinite(x) || !isFinite(y)) {
        return corsResponse(JSON.stringify({ error: "invalid tile coords" }), {
          status: 400,
          headers: { "content-type": "application/json" },
        });
      }
      return handleTile(env, provider, z, x, y);
    }

    if (parts[0] === "manifests" && parts.length >= 2) {
      // Support nested paths: /manifests/austin.json, /manifests/austin/index.json,
      // /manifests/austin/chunks/0_0.json — all map to R2 key after "manifests/"
      // Trailing /stats segment returns a size/ratio summary across json
      // and msgpack variants of a single chunk.
      const rest = parts.slice(1);
      if (rest.length >= 2 && rest[rest.length - 1] === "stats") {
        const r2Key = rest.slice(0, -1).join("/");
        return handleManifestStats(env, r2Key);
      }
      const r2Key = rest.join("/");
      return handleManifest(env, r2Key, request);
    }

    // Admin manifest upload endpoint. Path shape:
    //   PUT /admin/manifest/{city}/index.json
    //   PUT /admin/manifest/{city}/chunks/{id}.json
    // Body: raw JSON. Auth: X-Admin-Token header must match env.ADMIN_TOKEN.
    // Writes directly via the R2 binding (not the public management API)
    // so it bypasses the 429 rate limits that `wrangler r2 object put`
    // hits on bulk uploads. Also purges CHUNK_CACHE for the uploaded key
    // so the next GET through the normal serving path sees the new bytes.
    if (parts[0] === "admin" && parts[1] === "manifest" && request.method === "PUT") {
      if (!env.MANIFESTS) {
        return corsResponse(
          JSON.stringify({ error: "R2 binding not configured" }),
          { status: 503, headers: { "content-type": "application/json" } },
        );
      }
      if (!env.ADMIN_TOKEN) {
        return corsResponse(
          JSON.stringify({ error: "admin upload disabled (ADMIN_TOKEN unset)" }),
          { status: 503, headers: { "content-type": "application/json" } },
        );
      }
      const token = request.headers.get("x-admin-token") || "";
      // Timing-safe comparison to prevent token oracle attacks.
      const encoder = new TextEncoder();
      const tokenBytes = encoder.encode(token);
      const secretBytes = encoder.encode(env.ADMIN_TOKEN);
      // Pad to equal length (crypto.subtle.timingSafeEqual requires same length).
      const maxLen = Math.max(tokenBytes.length, secretBytes.length, 1);
      const a = new Uint8Array(maxLen);
      const b = new Uint8Array(maxLen);
      a.set(tokenBytes);
      b.set(secretBytes);
      const tokensMatch = crypto.subtle.timingSafeEqual(a, b)
        && tokenBytes.length === secretBytes.length;
      if (!tokensMatch) {
        return corsResponse(
          JSON.stringify({ error: "unauthorized" }),
          { status: 401, headers: { "content-type": "application/json" } },
        );
      }
      const rest = parts.slice(2);
      if (rest.length < 2) {
        return corsResponse(
          JSON.stringify({ error: "usage: PUT /admin/manifest/{city}/{path}" }),
          { status: 400, headers: { "content-type": "application/json" } },
        );
      }
      // R2 key matches handleManifest's read path: the `manifests/`
      // URL prefix is stripped when constructing the R2 key, so chunks
      // live at "austin/chunks/0_0.json" not "manifests/austin/chunks/0_0.json".
      // CHUNK_CACHE uses the same key, so the purge below targets the
      // same entry the read path will look up on the next GET.
      const r2Key = rest.join("/");
      const contentType = request.headers.get("content-type") || "application/json";
      const body = await request.arrayBuffer();
      await env.MANIFESTS.put(r2Key, body, {
        httpMetadata: { contentType },
      });
      // Invalidate the corresponding CHUNK_CACHE entry. The cache key
      // mirrors the R2 key exactly (see handleManifest), so a direct
      // KV delete is enough — next GET will miss, fall through to R2,
      // and repopulate from the bytes we just wrote.
      if (env.CHUNK_CACHE) {
        try {
          await env.CHUNK_CACHE.delete(r2Key);
        } catch {
          // Non-fatal: stale cache entries will eventually expire via TTL.
        }
      }
      return corsResponse(
        JSON.stringify({ ok: true, key: r2Key, bytes: body.byteLength }),
        { status: 200, headers: { "content-type": "application/json" } },
      );
    }

    return corsResponse(
      JSON.stringify({
        error: "not found",
        endpoints: [
          "GET /health",
          "GET /tile/{provider}/{z}/{x}/{y}",
          "GET /manifests/{name}.json",
          "POST /telemetry/run",
          "GET /telemetry/latest?limit=N",
        ],
      }),
      { status: 404, headers: { "content-type": "application/json" } },
    );
  },
};
