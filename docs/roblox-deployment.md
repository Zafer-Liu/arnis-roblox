# Roblox Open Cloud deployment

This document describes how to publish Arnis to a live Roblox experience
using the two-part deployment architecture the runtime expects: a small
scripts-only place file that ships through Open Cloud, and a compiled
manifest JSON that the runtime fetches at boot via `HttpService:GetAsync`.

## Why two parts

The authored place file (`roblox/out/arnis-scripts-only.rbxlx`, ~7.7 MiB)
intentionally contains only code, the baseplate, and configuration. The
world data — chunks, buildings, terrain, roads — lives in a separate
compiled manifest JSON (tens to hundreds of MiB) that the runtime streams
in on demand.

Benefits:

- Open Cloud place uploads stay small and fast to iterate on.
- Swapping the hosted manifest URL publishes a new city (or a new bake
  of the same city) without re-uploading the place.
- The same published place can serve multiple manifests for A/B proofs.
- CI can publish scripts-only changes without needing to bake world data.

The runtime side of this contract lives in
`roblox/src/ServerScriptService/ImportService/ManifestLoader.lua`
(`loadFromExternalSource`) and is keyed by
`roblox/src/ReplicatedStorage/Shared/WorldConfig.lua` ->
`WorldConfig.ManifestSource`.

## Part 1: publish the place file

### 1. Get an Open Cloud API key

1. Sign in to <https://create.roblox.com/dashboard/credentials>.
2. Create an API key.
3. Under **Access Permissions**, add the **Place Management API** and
   select the target universe + place.
4. Grant the `place:write` operation (sometimes labelled
   "Place Management -> Write").
5. Add your deploy machine's public IP to the allow list (or `0.0.0.0/0`
   for local testing — tighten later).
6. Save and copy the key once; Roblox will not show it again.

### 2. Find the universe and place IDs

- **Universe ID**: open the experience in the creator dashboard; it is
  visible in the URL and in the **Basic Info** tab as "Universe ID".
- **Place ID**: each experience has one or more places; the start place
  ID is shown on the **Places** tab. For a scripts-only deploy you
  almost always target the start place.

These are distinct numeric IDs. The Open Cloud endpoint requires both.

### 3. Export the API key

```bash
export ROBLOX_OPEN_CLOUD_API_KEY='paste-the-key-here'
```

Never commit this value, never paste it into a script, and never pass
it as a CLI argument. The publish script only reads it from the env.

### 4. Dry-run the publish

```bash
python3 scripts/publish_to_roblox.py \
  --place-file roblox/out/arnis-scripts-only.rbxlx \
  --universe-id 1234567890 \
  --place-id 9876543210 \
  --dry-run
```

Expected output:

```
[publish_to_roblox] dry-run preview:
  place_file      = roblox/out/arnis-scripts-only.rbxlx
  universe_id     = 1234567890
  place_id        = 9876543210
  version_type    = Published
  url             = https://apis.roblox.com/universes/v1/1234567890/places/9876543210/versions?versionType=Published
  content_type    = application/xml
  body_size_bytes = 8087040 (7.71 MiB)
  auth_header     = x-api-key: <redacted>
```

### 5. Publish for real

Drop `--dry-run`:

```bash
python3 scripts/publish_to_roblox.py \
  --place-file roblox/out/arnis-scripts-only.rbxlx \
  --universe-id 1234567890 \
  --place-id 9876543210
```

On success the script prints:

```
[publish_to_roblox] OK place_id=9876543210 universe_id=1234567890 versionNumber=<n>
```

Use `--version-type Saved` to upload without going live; use
`--version-type Published` (the default) to push the new version to
every player immediately.

### Endpoint reference

```
POST https://apis.roblox.com/universes/v1/{universeId}/places/{placeId}/versions
    ?versionType=Published
Headers:
    x-api-key: <ROBLOX_OPEN_CLOUD_API_KEY>
    Content-Type: application/xml        # for .rbxlx
    Content-Type: application/octet-stream  # for .rbxl
Body: raw place file bytes
```

See the official docs at
<https://create.roblox.com/docs/cloud/legacy/open-cloud> for the
authoritative description of scopes, rate limits, and response shape.

## Part 2: host the compiled manifest

The place file on its own will boot into an empty scene. To populate
it, host the compiled manifest JSON somewhere the Roblox game servers
can reach over HTTPS and point `WorldConfig.ManifestSource` at that URL.

### Verify the manifest is HttpService-decodable

Before hosting, run:

```bash
python3 scripts/host_manifest.py roblox/out/austin.json
```

This invokes `verify_manifest_http_payload.py` under the hood to confirm
the file is valid UTF-8 JSON with the required top-level keys and no
`NaN`/`Inf` values (which `HttpService:JSONDecode` rejects). It also
prints concrete hosting commands for each supported lane.

### Hosting lanes

All four options are free-tier friendly and work with Roblox's HTTP
fetcher. Pick whichever fits your operational model.

1. **Amazon S3**

   ```bash
   aws s3 cp roblox/out/austin.json s3://<bucket>/austin.json \
       --acl public-read --content-type application/json
   # URL: https://<bucket>.s3.amazonaws.com/austin.json
   ```

2. **Cloudflare R2**

   ```bash
   wrangler r2 object put <bucket>/austin.json \
       --file roblox/out/austin.json \
       --content-type application/json
   # URL: https://<your-r2-domain>/austin.json
   ```

3. **GitHub Pages** — the simplest option if the manifest is small
   enough to commit (watch the 100 MiB file size cap):

   ```bash
   cp roblox/out/austin.json docs/manifests/austin.json
   git add docs/manifests && git commit -m 'publish manifest' && git push
   # URL: https://<user>.github.io/<repo>/manifests/austin.json
   ```

4. **Any plain HTTPS static host** (Netlify, Fly volumes, a cached
   nginx, etc.) — serve the file with
   `Content-Type: application/json`.

### Local smoke-test server

For iterating from Studio without a public host:

```bash
python3 scripts/host_manifest.py roblox/out/austin.json --serve --port 8787
```

This binds `http://0.0.0.0:8787/austin.json` and serves the file with
the correct `Content-Type`. Point Studio at that URL (or use a tunnel
such as `cloudflared` / `ngrok` for a public HTTPS endpoint).

### Wire it into WorldConfig

Edit `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua`:

```lua
WorldConfig.ManifestSource = {
    mode = "external_url",
    url = "https://<your-host>/austin.json",
    timeoutSeconds = 20,
}
```

`ManifestLoader.loadFromExternalSource` will call
`HttpService:GetAsync(url)` followed by `HttpService:JSONDecode` and
validate the result against `ChunkSchema.validateManifest`. A failed
fetch is fatal — the server refuses to boot rather than serve a
half-loaded world.

Remember to enable **HTTP Requests** for the experience under
**Game Settings -> Security** before the first boot, otherwise
`HttpService:GetAsync` will error out on the live server.

## CI integration

`scripts/publish_to_roblox.py` uses only Python stdlib (`urllib`,
`argparse`, `json`), so any CI image with Python 3.11+ can run it
without a virtualenv. The recommended CI shape is:

1. Build the scripts-only place via vertigo-sync.
2. Run `scripts/publish_to_roblox.py --dry-run ...` on pull requests.
3. Run the real publish (no `--dry-run`) only from `main` with the
   `ROBLOX_OPEN_CLOUD_API_KEY` secret injected from the CI provider's
   secret store.

The accompanying unit tests live at
`scripts/tests/test_publish_to_roblox.py` and can be run with:

```bash
python3 -m unittest scripts.tests.test_publish_to_roblox -q
```
