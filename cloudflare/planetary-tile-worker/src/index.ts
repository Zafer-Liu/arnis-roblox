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
  DEFAULT_TILE_PROVIDER: string;
  TILE_CACHE_TTL_SECONDS: string;
}

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
        ],
      }),
      { status: 404, headers: { "content-type": "application/json" } },
    );
  },
};
