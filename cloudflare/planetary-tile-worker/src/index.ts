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
  DEFAULT_TILE_PROVIDER: string;
  TILE_CACHE_TTL_SECONDS: string;
}

const TELEMETRY_INDEX_KEY = "telemetry:index";
const TELEMETRY_INDEX_MAX = 100;
const TELEMETRY_TTL_SECONDS = 60 * 60 * 24 * 30; // 30 days

const CORS_HEADERS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, OPTIONS",
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
  const base64 = btoa(String.fromCharCode(...jpegBytes));

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

async function handleManifest(env: Env, name: string): Promise<Response> {
  if (!env.MANIFESTS) {
    return corsResponse(
      JSON.stringify({ error: "R2 binding not configured" }),
      { status: 500, headers: { "content-type": "application/json" } },
    );
  }

  const obj = await env.MANIFESTS.get(name);
  if (!obj) {
    return corsResponse(
      JSON.stringify({ error: `manifest not found: ${name}` }),
      { status: 404, headers: { "content-type": "application/json" } },
    );
  }

  return corsResponse(obj.body, {
    headers: {
      "content-type": "application/json",
      "cache-control": "public, max-age=300",
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
      const r2Key = parts.slice(1).join("/");
      return handleManifest(env, r2Key);
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
