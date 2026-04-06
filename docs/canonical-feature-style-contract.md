# Canonical Feature Style Contract

Date: 2026-04-04

## Goal

Unify source features from the repo's data union into one canonical feature layer, then map that layer onto Roblox rendering/runtime capabilities without hard-coding style decisions inside every builder.

The source union currently includes:

- OSM
- Overpass-derived data
- Overture-derived data
- Other future adapters

The renderer target must cover:

- terrain
- materials
- textures
- meshes
- decals / markings
- props
- lighting / atmospheric accents
- sound / ambient identity
- streaming / LOD behavior

All of that must be:

- optional
- overrideable
- defaulted sensibly
- source-traceable
- future-proof for regional style localization

## Core Principle

Split the problem into four layers:

1. Source canonicalization
2. Canonical feature semantics
3. Style resolution
4. Roblox realization

We should not let OSM tags, Overture fields, or ad hoc builder heuristics directly decide final Roblox materials/meshes in most places.

## Layer 1: Source Canonicalization

Purpose:
Take the union of raw source records and normalize them into one canonical source feature record before render mapping.

Each canonical source feature should carry:

- `canonical_feature_id`
- `feature_family`
- `feature_kind`
- `geometry_kind`
- `source_lineage`
- `retained_semantics`
- `collapsed_semantics`
- `confidence`
- `regional_context`

Examples:

- A road from OSM and a matching transportation segment from Overture should collapse into one canonical road feature with preserved lineage.
- A building footprint from OSM plus enriched usage/material metadata from another source should remain one canonical structure feature, not multiple renderer decisions scattered across builders.

This is where we should continue improving truth-pack canonicalization.

## Layer 2: Canonical Feature Semantics

Purpose:
Define what a thing is independent of how Roblox renders it.

Every canonical feature should resolve to a stable semantic identity such as:

- `transport.road.primary`
- `transport.path.footway`
- `transport.path.sidewalk`
- `structure.building.residential.rowhouse`
- `structure.building.commercial.midrise`
- `landcover.grass.park`
- `water.channel.stream`

It should also expose render-relevant semantic traits such as:

- `surface_family`
- `wall_family`
- `roof_family`
- `edge_family`
- `structural_form`
- `pedestrian_priority`
- `vehicle_priority`
- `material_age`
- `urban_density`
- `style_region`

These traits are what style resolution should consume.

## Layer 3: Style Resolution

Purpose:
Map canonical semantics to a style package, not directly to one concrete Roblox asset/material.

This layer should resolve:

- material family
- texture family
- mesh family
- detail/decal family
- prop set family
- weathering family
- region/style overrides
- user/custom overrides

This should produce stable style selectors, for example:

- `road.surface.asphalt.city-standard`
- `sidewalk.surface.light-concrete.us-sunbelt`
- `building.wall.brick.red-lowrise.us-texas`
- `roof.shingle.dark-hip.us-suburban`

Then the Roblox layer can map those selectors to actual assets/material variants/surface appearances/mesh parts.

## Layer 4: Roblox Realization

Purpose:
Resolve style selectors into actual Roblox engine constructs.

Target surfaces should include:

- `Terrain` material writes
- `Part` / `MeshPart`
- `EditableMesh`
- `SurfaceAppearance`
- `MaterialVariant`
- `Texture` / decals / markings
- props / prefabs / attachments
- audio emitters
- particles / atmosphere hooks
- model `LevelOfDetail`

The important rule:

Builders should consume resolved render instructions, not invent local style policy.

## Required Contract Objects

We should add three explicit contracts.

### 1. Canonical Feature Descriptor

Stable semantic description of the thing.

Suggested fields:

- `canonicalFeatureId`
- `featureFamily`
- `featureKind`
- `geometryKind`
- `usage`
- `form`
- `sourceLineage`
- `semanticTraits`
- `regionalContext`

### 2. Style Selector

Stable render intent, still asset-agnostic.

Suggested fields:

- `styleProfile`
- `styleRegion`
- `materialFamily`
- `textureFamily`
- `meshFamily`
- `detailFamily`
- `lodFamily`
- `overrideKeys`

### 3. Roblox Render Recipe

Final resolved Roblox-facing instruction bundle.

Suggested fields:

- `terrainMaterial`
- `materialVariant`
- `surfaceAppearanceKey`
- `meshAssetKey`
- `prefabKey`
- `decalSetKey`
- `detailPolicy`
- `lodPolicy`
- `streamingPolicy`

## Optional Overrides With Sensible Defaults

Defaults:

- If no custom mesh pack exists, use stock/default generated geometry.
- If no regional style pack exists, use the canonical default profile.
- If no texture family is provided, use material-only rendering.
- If no custom prop family exists, use the current default prefabs.

Overrides:

- custom mesh packs
- custom material packs
- custom texture packs
- custom decal/marking packs
- regional style packs
- per-feature manual overrides

This means custom content becomes additive, not required.

## Custom Mesh Plug-In Surface

We should make mesh substitution a first-class registry, not builder-local special cases.

Needed behavior:

- register mesh families by semantic/style selector
- allow fallback to generated geometry if a mesh is unavailable
- allow per-style-region mesh bundles
- allow per-project custom packs without forking core builders

Examples:

- `transport.path.sidewalk.us-sunbelt` -> custom sidewalk curb mesh kit
- `structure.building.residential.rowhouse.uk-brick` -> rowhouse mesh family
- `prop.streetlight.modern-us` -> custom prefab family

## Regional Style Localization

This is not text localization first. It is visual/regional localization.

We should support future region packs such as:

- `us-sunbelt`
- `us-northeast`
- `uk-terraced`
- `jp-urban`
- `de-suburban`

Each region pack should be able to override:

- wall materials
- roof families
- sidewalk conventions
- curb profiles
- road marking styles
- street furniture families
- vegetation defaults
- ambient detail sets

All optional. The default profile must still work with no region pack.

## Manifest and Audit Implications

The manifest should carry enough canonical/style identity to audit what was intended versus what Roblox realized.

Recommended additions or stronger guarantees:

- canonical feature family/kind on retained renderable features
- style selector or style profile key
- render recipe key / asset key
- source lineage preserved into audits
- reason codes when a richer render recipe fell back to a default

Audit surfaces should be able to answer:

- Was this thing canonicalized correctly?
- Which style selector did it resolve to?
- Which Roblox render recipe was chosen?
- Did it fall back?
- Did the realized world actually expose the expected visible result?

## Immediate Implementation Direction

We should not try to solve everything in one jump.

### Tranche A

Create the style contract and resolver surface.

Minimal initial target:

- roads
- sidewalks
- footways
- building wall families
- building roof families

### Tranche B

Move current builder-local material selection into the resolver.

High-value first:

- `RoadBuilder`
- `BuildingBuilder`
- preview/runtime shared contract

### Tranche C

Add optional mesh pack registry.

This should allow:

- default generated geometry
- custom mesh override by selector
- region-pack override by selector

### Tranche D

Extend audits and proof tooling so parity checks understand intended style/render identity, not just raw counts.

## What This Changes About Current Work

The current parity work should be treated as the first consumer of this contract, not the final architecture.

Specifically:

- road/path/sidewalk parity
- remaining building wall parity
- local visual distinctiveness

should migrate toward:

- canonical feature identity
- canonical style resolution
- Roblox render recipes

not one-off per-builder heuristics.

## Practical Rule

When we see a builder deciding something like:

- “all sidewalks are pavement”
- “all pedestrian paths use one standard material”
- “all buildings of class X use one fallback shell style”

that is a signal the decision belongs in the style resolver, not in the builder.
