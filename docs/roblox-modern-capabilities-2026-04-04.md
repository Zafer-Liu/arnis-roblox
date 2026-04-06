# Modern Roblox Capability Map

Date: 2026-04-04

This is a compact research surface for modern Roblox capabilities that matter to `arnis-roblox`, especially fidelity, detail preservation, diversification, and external tooling. Links are official Roblox sources.

## Geometry and Runtime Surface Generation

- `AssetService`
  https://create.roblox.com/docs/reference/engine/classes/AssetService
  Relevant APIs:
  - `CreateEditableMesh()`
  - `CreateEditableMeshAsync()`
  - `CreateMeshPartAsync()`
  - `CreateEditableImage()`
  - `CreateEditableImageAsync()`
  - `CreateSurfaceAppearanceAsync()`
  Why it matters:
  - Roblox exposes a modern runtime path for procedural mesh generation, mesh cloning, image-backed map generation, and PBR surface authoring.
  - We are currently using only a slice of this. There is room to push distinct facade/roof/road materials, decals, and local detail without collapsing everything into the same baked surface.

- `EditableMesh`
  https://create.roblox.com/docs/reference/engine/classes/EditableMesh
  Why it matters:
  - Roblox exposes direct mesh topology/attribute operations plus local raycasting.
  - This is relevant for preserving non-standard rooflines, façade breakups, curb geometry, path edge cuts, and local hit testing against generated geometry.

## Material and Appearance Fidelity

- Texture specifications and `SurfaceAppearance`
  https://create.roblox.com/docs/art/modeling/texture-specifications
  Why it matters:
  - Roblox supports PBR textures on meshes through `SurfaceAppearance`.
  - The docs explicitly cover texture sizing and texel expectations, which gives us a defensible way to add material richness without blowing memory.

- `BasePart` and `MaterialVariant`
  https://create.roblox.com/docs/reference/engine/classes/BasePart
  Why it matters:
  - Roblox exposes `MaterialVariant` on parts.
  - That gives us a path to diversify repeated materials without replacing every asset with a unique mesh or texture set.
  - High-payoff targets for us: sidewalks, footways, service alleys, curb families, wall finish variation, roof finish variation.

- Importer
  https://create.roblox.com/docs/studio/importer
  Why it matters:
  - Roblox’s modern importer supports richer mesh/PBR workflows than our current runtime lane assumes.
  - Even when we stay procedural, this is a signal that the engine expects higher-fidelity mesh and texture content than “single standardized material per class.”

## Streaming and LOD

- Instance streaming techniques
  https://create.roblox.com/docs/workspace/streaming/techniques
  Why it matters:
  - Roblox exposes more than raw `StreamingEnabled`; the docs call out `Model.LevelOfDetail`, `SLIM`, and `StreamingMesh`.
  - For us, this is a concrete route to preserve distant identity instead of deleting it:
    - shell models that keep their silhouette farther away
    - district-specific landmark retention
    - less aggressive pop-in for buildings that currently “look like nothing” until close range

## Atmosphere, Lighting, and Environmental Readability

- Atmosphere
  https://create.roblox.com/docs/environment/atmosphere
  Why it matters:
  - Roblox’s environment stack is capable of much more scene shaping than we currently exploit.
  - This is relevant for depth separation, skyline readability, haze, long-view composition, and making neighborhoods feel different instead of uniformly lit.

- Environment overview
  https://create.roblox.com/docs/environment
  Why it matters:
  - Lighting, atmosphere, and post-processing are a legitimate fidelity lane, not decoration.
  - We should treat them as part of world identity, especially if we want places to “look like themselves.”

## Performance and Diagnosis

- MicroProfiler
  https://create.roblox.com/docs/optimization/microprofiler
  Why it matters:
  - Roblox exposes first-class runtime profiling in Studio/client.
  - We should use it more aggressively before assuming a fidelity feature is too expensive.

## External and Online Surfaces

- Open Cloud reference
  https://create.roblox.com/docs/cloud/reference
  Why it matters:
  - Roblox exposes a modern authenticated REST surface for external tools and web apps.
  - This is the official online path for companion services, automation, orchestration, publishing workflows, inventory/assets, server operations, and data tooling.

- `apis.roblox.com` domain reference
  https://create.roblox.com/docs/cloud/reference/domains/apis
  Why it matters:
  - Useful as the concrete entry surface when we need stable modern endpoints instead of legacy cookie-auth paths.

- `HttpService` / in-experience HTTP
  https://create.roblox.com/docs/cloud-services/http-service
  Why it matters:
  - Roblox explicitly documents in-experience access to a subset of Open Cloud and gives current request-limit guidance.
  - That is relevant if we want online detail enrichment, runtime lookups, or a hybrid external fidelity service.

- Memory stores
  https://create.roblox.com/docs/cloud-services/memory-stores
  Why it matters:
  - If we build any online companion for live route/session state, hot caches, or cross-server orchestration, MemoryStore is the low-latency in-platform primitive.

## Immediate Implications for `arnis-roblox`

- We should stop treating Roblox as if it only supports standardized `Part` + `Terrain` fidelity. Officially, it exposes:
  - procedural mesh generation
  - runtime-editable images
  - runtime PBR surface creation
  - material variants
  - explicit model LOD controls
  - modern external cloud APIs

- The highest-payoff near-term upgrades for our world fidelity lane are:
  - preserve route/building identity with less aggressive standardization of path, sidewalk, wall, and roof surfaces
  - use `MaterialVariant` / `SurfaceAppearance` strategically for repeated-but-distinct surfaces
  - use model `LevelOfDetail` intentionally for distant shell preservation
  - use `EditableMesh` more deliberately for non-standard geometry instead of flattening edge cases

- The biggest architectural point:
  - Roblox is no longer the constraint we should be designing around.
  - Our current bottleneck is mostly our own simplification policy, merge policy, and runtime visibility policy.

## Recommended Next Research-to-Implementation Tranches

- Tranche 1:
  Road/path/sidewalk identity preservation in play.
  Use current route-proof lane to reduce over-standardization and improve pedestrian-surface readability.

- Tranche 2:
  Building surface diversification.
  Introduce source-faithful wall/roof material families using `MaterialVariant` first, then selective `SurfaceAppearance` where payoff is high.

- Tranche 3:
  Distant silhouette parity.
  Audit `Model.LevelOfDetail` / streaming techniques against our building shells and landmarks so far-distance identity survives.

- Tranche 4:
  Online companion surface.
  If we want “available to us online” in the strong sense, build a small Open Cloud-backed companion that can inspect route sessions, proof artifacts, and exported fidelity summaries outside Studio.
