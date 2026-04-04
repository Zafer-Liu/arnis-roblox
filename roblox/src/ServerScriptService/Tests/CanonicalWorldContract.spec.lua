return function()
    local Assert = require(script.Parent.Assert)
    local AustinSpawn = require(script.Parent.Parent.ImportService.AustinSpawn)
    local CanonicalWorldContract = require(script.Parent.Parent.ImportService.CanonicalWorldContract)
    local ManifestLoader = require(script.Parent.Parent.ImportService.ManifestLoader)

    local function makeManifestSource()
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "ExportedWorld",
                chunkSizeStuds = 256,
                canonicalAnchor = {
                    positionStuds = {
                        x = -6.0854,
                        y = -0.4639,
                        z = -208.371,
                    },
                    lookDirectionStuds = {
                        x = 0,
                        y = 0,
                        z = 1,
                    },
                },
            },
            chunks = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    roads = {},
                    buildings = {},
                    props = {},
                },
                {
                    id = "1_0",
                    originStuds = { x = 256, y = 0, z = 0 },
                    roads = {},
                    buildings = {},
                    props = {},
                },
                {
                    id = "2_0",
                    originStuds = { x = 512, y = 0, z = 0 },
                    roads = {},
                    buildings = {},
                    props = {},
                },
            },
        }
    end

    local canonicalFamily = CanonicalWorldContract.resolveCanonicalManifestFamily("preview")
    Assert.equal(
        canonicalFamily,
        "AustinManifestIndex",
        "expected the canonical Austin world family to stay locked to the full-bake manifest"
    )
    Assert.equal(
        CanonicalWorldContract.resolveCanonicalManifestFamily("play"),
        canonicalFamily,
        "expected preview and play to resolve the same canonical family"
    )
    Assert.equal(
        CanonicalWorldContract.resolveCanonicalManifestFamily("full_bake"),
        canonicalFamily,
        "expected full-bake requests to resolve the same canonical family"
    )
    local materializationCandidates = CanonicalWorldContract.resolveCanonicalMaterializationCandidates("full_bake")
    Assert.truthy(
        #materializationCandidates >= 1,
        "expected the canonical contract to expose at least one materialization candidate"
    )
    Assert.equal(
        materializationCandidates[#materializationCandidates],
        canonicalFamily,
        "expected the canonical full-bake family to remain the last-resort materialization candidate"
    )
    Assert.equal(
        CanonicalWorldContract.resolveCanonicalMaterializationFamily("full_bake"),
        materializationCandidates[1],
        "expected canonical materialization resolution to return the highest-priority available candidate"
    )

    local manifestSource = makeManifestSource()
    local canonicalAnchor = AustinSpawn.resolveCanonicalAnchorValues(manifestSource, 500)
    local boundedEnvelope = CanonicalWorldContract.resolveBoundedEnvelope(manifestSource, 500)

    Assert.equal(
        boundedEnvelope.manifestFamily,
        canonicalFamily,
        "expected bounded envelopes to retain the canonical manifest family"
    )
    Assert.truthy(type(boundedEnvelope.anchor) == "table", "expected bounded envelopes to carry a resolved anchor")
    Assert.equal(
        boundedEnvelope.focusPoint,
        canonicalAnchor.focusPoint,
        "expected bounded envelopes to reuse the canonical anchor focus point"
    )
    Assert.equal(
        boundedEnvelope.spawnPoint,
        canonicalAnchor.spawnPoint,
        "expected bounded envelopes to reuse the canonical anchor spawn point"
    )
    Assert.equal(
        boundedEnvelope.lookTarget,
        canonicalAnchor.lookTarget,
        "expected bounded envelopes to reuse the canonical anchor look target"
    )
    Assert.truthy(
        type(boundedEnvelope.chunkIds) == "table" and #boundedEnvelope.chunkIds > 0,
        "expected bounded envelopes to derive a chunk slice from the canonical artifact family"
    )
    Assert.equal(
        boundedEnvelope.manifestSourceKind,
        "canonical_manifest",
        "expected bounded envelopes to preserve canonical source kind"
    )

    local chunkSelectionCalls = 0
    local handleBackedManifest = {
        schemaVersion = "0.4.0",
        meta = manifestSource.meta,
        chunkRefs = manifestSource.chunks,
        GetChunkIdsWithinRadius = function(_self, focusPoint, radius)
            chunkSelectionCalls += 1
            Assert.truthy(focusPoint ~= nil, "expected canonical bounded envelopes to pass a focus point")
            Assert.truthy(radius ~= nil, "expected canonical bounded envelopes to pass a radius")
            return { "0_0", "1_0" }
        end,
        GetChunk = function(_self, chunkId)
            for _, chunk in ipairs(manifestSource.chunks) do
                if chunk.id == chunkId then
                    return chunk
                end
            end
            return nil
        end,
    }

    CanonicalWorldContract.resolveBoundedEnvelope(handleBackedManifest, 500)
    CanonicalWorldContract.resolveBoundedEnvelope(handleBackedManifest, 500)
    Assert.equal(
        chunkSelectionCalls,
        2,
        "expected canonical bounded envelopes to re-resolve chunk selection for each build on handle-backed manifests"
    )

    local originalLoadNamedRouteCatalogHandle = ManifestLoader.LoadNamedRouteCatalogHandle
    local routeCatalogCalls = {}
    local laneHandle = {
        chunkRefs = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
            },
        },
        GetChunkIdsWithinRadius = function()
            return { "0_0" }
        end,
        GetChunk = function(_self, chunkId)
            return {
                id = chunkId,
                originStuds = { x = 0, y = 0, z = 0 },
                roads = {},
                buildings = {},
                props = {},
            }
        end,
    }
    ManifestLoader.LoadNamedRouteCatalogHandle = function(name, timeoutSeconds, options)
        routeCatalogCalls[#routeCatalogCalls + 1] = {
            name = name,
            timeoutSeconds = timeoutSeconds,
            routeLane = options and options.routeLane or nil,
            routeStepIndex = options and options.routeStepIndex or nil,
        }
        return {
            LoadLaneRuntimeHandle = function(_self, stepIndex, laneName)
                routeCatalogCalls[#routeCatalogCalls + 1] = {
                    laneName = laneName,
                    stepIndex = stepIndex,
                }
                return laneHandle
            end,
        }
    end

    local routeManifestSource, resolvedRouteName = CanonicalWorldContract.loadCanonicalManifestSource("preview", 0, {
        routeCatalogName = "PlanetaryRouteBundle.route-catalog",
        routeLane = "active",
        routeStepIndex = 2,
    })
    Assert.equal(
        routeManifestSource,
        laneHandle,
        "expected canonical world contract to return the selected route lane runtime handle"
    )
    Assert.equal(
        resolvedRouteName,
        "PlanetaryRouteBundle.route-catalog",
        "expected route catalog loads to report the resolved route catalog name"
    )
    Assert.equal(
        routeManifestSource.manifestSourceKind,
        "route_catalog",
        "expected route catalog materialization to annotate the manifest source kind"
    )
    Assert.equal(
        routeManifestSource.manifestSourceName,
        "PlanetaryRouteBundle.route-catalog",
        "expected route catalog materialization to annotate the resolved source name"
    )
    Assert.equal(
        routeManifestSource.routeCatalogName,
        "PlanetaryRouteBundle.route-catalog",
        "expected route catalog materialization to preserve route catalog identity on the handle"
    )
    Assert.equal(
        routeManifestSource.routeLane,
        "active",
        "expected route catalog materialization to preserve route lane on the handle"
    )
    Assert.equal(
        routeManifestSource.routeStepIndex,
        2,
        "expected route catalog materialization to preserve route step index on the handle"
    )
    Assert.equal(routeCatalogCalls[1].name, "PlanetaryRouteBundle.route-catalog", "expected route catalog load name")
    Assert.equal(routeCatalogCalls[1].routeLane, "active", "expected route lane to flow into route catalog loading")
    Assert.equal(routeCatalogCalls[1].routeStepIndex, 2, "expected route step index to flow into route catalog loading")
    Assert.equal(
        routeCatalogCalls[2].laneName,
        "active",
        "expected canonical world contract to request the desired route lane"
    )
    Assert.equal(
        routeCatalogCalls[2].stepIndex,
        2,
        "expected canonical world contract to request the desired route step"
    )
    local routeEnvelope = CanonicalWorldContract.resolveBoundedEnvelope(routeManifestSource, 500)
    Assert.equal(
        routeEnvelope.manifestSourceKind,
        "route_catalog",
        "expected route envelopes to preserve route source kind"
    )
    Assert.equal(
        routeEnvelope.manifestSourceName,
        "PlanetaryRouteBundle.route-catalog",
        "expected route envelopes to preserve route source name"
    )
    Assert.equal(routeEnvelope.routeLane, "active", "expected route envelopes to preserve route lane")
    Assert.equal(routeEnvelope.routeStepIndex, 2, "expected route envelopes to preserve route step index")

    ManifestLoader.LoadNamedRouteCatalogHandle = originalLoadNamedRouteCatalogHandle
end
