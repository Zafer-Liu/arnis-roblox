# planetary tile worker

Cloudflare Worker for arnis-roblox planetary streaming. Serves:

- **Satellite tile proxy** — `/tile/{provider}/{z}/{x}/{y}` proxies ESRI/Mapbox/OSM/MapTiler tiles with KV caching
- **Manifest CDN** — `/manifests/{name}.json` serves compiled city manifests from R2

## First-time setup (you do this once)

```bash
cd cloudflare/planetary-tile-worker
npm install
npx wrangler login   # opens browser, authorizes with adpena Cloudflare account

# Create the KV namespace for tile cache
npx wrangler kv:namespace create planetary_tile_cache
# Paste the returned ID into wrangler.toml under [[kv_namespaces]]

# Create the R2 bucket for manifests
npx wrangler r2 bucket create planetary-manifests
# Uncomment the [[r2_buckets]] block in wrangler.toml

# Deploy
npx wrangler deploy
```

After deploy the worker is live at:
- `https://planetary.adpena.workers.dev/health`
- `https://planetary.adpena.workers.dev/tile/esri/14/3823/6707`
- `https://planetary.adpena.workers.dev/manifests/austin.json`

## Upload a manifest to R2

```bash
npx wrangler r2 object put planetary-manifests/austin.json --file=out/austin-publish.json
```

Then in WorldConfig.lua:
```lua
ManifestSource = {
    mode = "external_url",
    externalUrl = "https://planetary.adpena.workers.dev/manifests/austin.json",
},
```

## Endpoints

### `GET /health`
Returns `{"status": "ok", "service": "planetary"}`. Liveness probe.

### `GET /tile/{provider}/{z}/{x}/{y}`
Providers: `esri`, `mapbox`, `osm`, `maptiler`.

Returns:
```json
{
  "provider": "esri",
  "z": 14, "x": 3823, "y": 6707,
  "width": 256, "height": 256,
  "format": "jpeg",
  "bytesBase64": "..."
}
```

The Roblox runtime currently can't decode JPEG natively in Lua. Until
EditableImage gets a `LoadFromBytes` method, the Lua side falls back
to a placeholder colored Part for each tile. The worker is ready for
when the decode pipeline lands.

### `GET /manifests/{name}.json`
Returns the requested manifest JSON from R2 with permissive CORS.

## Local dev

```bash
npx wrangler dev
# → http://localhost:8787
curl http://localhost:8787/health
curl http://localhost:8787/tile/esri/14/3823/6707
```

## Cost

Cloudflare Workers free tier: 100,000 requests/day. R2 storage: 10 GB
free. KV: 100,000 reads/day, 1,000 writes/day, 1 GB storage. Should
cover early playtesting at zero cost.
