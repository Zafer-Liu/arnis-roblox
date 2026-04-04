return function()
    local CanonicalWorldContract = require(script.Parent.Parent.ImportService.CanonicalWorldContract)
    local ManifestLoader = require(script.Parent.Parent.ImportService.ManifestLoader)
    local RunAustin = require(script.Parent.Parent.ImportService.RunAustin)
    local Assert = require(script.Parent.Assert)

    Assert.equal(
        RunAustin.getManifestName(),
        CanonicalWorldContract.resolveCanonicalManifestFamily("play"),
        "expected play mode to use the canonical full-bake Austin manifest family"
    )
    Assert.equal(
        RunAustin.CANONICAL_MANIFEST_INDEX_NAME,
        "AustinManifestIndex",
        "expected the runtime canonical manifest constant to stay locked to the full-bake Austin family"
    )

    local candidates = RunAustin.getRuntimeManifestCandidates()
    Assert.truthy(#candidates >= 1, "expected at least one runtime manifest candidate")
    Assert.equal(
        candidates[#candidates],
        "AustinManifestIndex",
        "expected runtime candidates to keep the canonical Austin family as the final fallback"
    )
    Assert.equal(
        candidates[1],
        CanonicalWorldContract.resolveCanonicalMaterializationFamily("play"),
        "expected runtime selection to resolve through the canonical materialization contract"
    )

    local originalLoadNamedRouteCatalogHandle = ManifestLoader.LoadNamedRouteCatalogHandle
    local routeCalls = {}
    ManifestLoader.LoadNamedRouteCatalogHandle = function(name, timeoutSeconds, options)
        routeCalls[#routeCalls + 1] = {
            name = name,
            timeoutSeconds = timeoutSeconds,
            routeLane = options.routeLane,
            routeStepIndex = options.routeStepIndex,
        }
        return {
            LoadLaneRuntimeHandle = function(_self, stepIndex, laneName)
                routeCalls[#routeCalls + 1] = {
                    laneName = laneName,
                    stepIndex = stepIndex,
                }
                return {
                    chunkRefs = {},
                    GetChunkIdsWithinRadius = function()
                        return {}
                    end,
                    GetChunk = function()
                        return nil
                    end,
                }
            end,
        }
    end

    local _routeHandle, resolvedRouteName = RunAustin.loadManifestSource({
        routeCatalogName = "PlanetaryRouteBundle.route-catalog",
        routeLane = "retain",
        routeStepIndex = 3,
    })
    Assert.equal(
        resolvedRouteName,
        "PlanetaryRouteBundle.route-catalog",
        "expected runtime route selection to report the route catalog as the resolved manifest source"
    )
    Assert.equal(
        routeCalls[1].name,
        "PlanetaryRouteBundle.route-catalog",
        "expected runtime route selection to load the route catalog"
    )
    Assert.equal(routeCalls[1].routeLane, "retain", "expected runtime route selection to forward the desired lane")
    Assert.equal(routeCalls[1].routeStepIndex, 3, "expected runtime route selection to forward the desired step")
    Assert.equal(
        routeCalls[2].laneName,
        "retain",
        "expected runtime route selection to request the desired runtime lane handle"
    )
    Assert.equal(routeCalls[2].stepIndex, 3, "expected runtime route selection to request the desired route step")

    ManifestLoader.LoadNamedRouteCatalogHandle = originalLoadNamedRouteCatalogHandle
end
