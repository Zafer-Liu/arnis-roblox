local ServerStorage = game:GetService("ServerStorage")

local AustinSpawn = require(script.Parent.AustinSpawn)
local ManifestLoader = require(script.Parent.ManifestLoader)

local CanonicalWorldContract = {}

CanonicalWorldContract.CANONICAL_MANIFEST_INDEX_NAME = "AustinManifestIndex"
CanonicalWorldContract.LOCAL_DEV_FULL_BAKE_MATERIALIZATION_INDEX_NAMES = {
    "AustinCanonicalManifestIndex",
}
CanonicalWorldContract.LOCAL_DEV_PLAY_MATERIALIZATION_INDEX_NAMES = {
    "AustinCanonicalManifestIndex",
}

-- Planetary streaming: multi-city manifest registry.
-- Each entry maps a world-space region (origin + radius in studs) to a manifest.
-- When the player crosses a region boundary, the streaming system loads the
-- corresponding manifest. Regions can overlap; the nearest center wins.
-- For single-city mode, this table has one entry covering the whole world.
CanonicalWorldContract.PlanetaryManifestRegistry = {
    {
        worldName = "Austin",
        manifestIndexName = "AustinManifestIndex",
        originStuds = Vector3.new(0, 0, 0), -- world origin for this city
        radiusStuds = 50000, -- covers the compiled area
    },
    -- Future entries:
    -- { worldName = "Amsterdam", manifestIndexName = "AmsterdamManifestIndex", originStuds = ..., radiusStuds = ... },
    -- { worldName = "Tokyo", manifestIndexName = "TokyoManifestIndex", originStuds = ..., radiusStuds = ... },
}

-- Resolve which manifest to use based on world-space position.
-- Returns the manifest index name and the region entry.
function CanonicalWorldContract.resolveManifestForPosition(worldPosition)
    local bestEntry = nil
    local bestDistSq = math.huge
    for _, entry in ipairs(CanonicalWorldContract.PlanetaryManifestRegistry) do
        local delta = worldPosition - entry.originStuds
        local distSq = delta.X * delta.X + delta.Z * delta.Z
        if distSq < entry.radiusStuds * entry.radiusStuds and distSq < bestDistSq then
            bestEntry = entry
            bestDistSq = distSq
        end
    end
    if bestEntry then
        return bestEntry.manifestIndexName, bestEntry
    end
    -- Fallback to canonical
    return CanonicalWorldContract.CANONICAL_MANIFEST_INDEX_NAME, nil
end

function CanonicalWorldContract.resolveCanonicalManifestFamily(_policyMode)
    return CanonicalWorldContract.CANONICAL_MANIFEST_INDEX_NAME
end

local function getSampleDataFolder()
    return ServerStorage:FindFirstChild("SampleData")
end

function CanonicalWorldContract.resolveCanonicalMaterializationCandidates(policyMode)
    local candidates = {}
    local sampleData = getSampleDataFolder()
    if sampleData then
        local preferredIndexNames = CanonicalWorldContract.LOCAL_DEV_FULL_BAKE_MATERIALIZATION_INDEX_NAMES
        if policyMode == "play" then
            preferredIndexNames = CanonicalWorldContract.LOCAL_DEV_PLAY_MATERIALIZATION_INDEX_NAMES
        end
        for _, manifestIndexName in ipairs(preferredIndexNames) do
            if sampleData:FindFirstChild(manifestIndexName) ~= nil then
                candidates[#candidates + 1] = manifestIndexName
            end
        end
    end

    local canonicalFamily = CanonicalWorldContract.resolveCanonicalManifestFamily(policyMode)
    candidates[#candidates + 1] = canonicalFamily
    return candidates
end

function CanonicalWorldContract.resolveCanonicalMaterializationFamily(policyMode)
    return CanonicalWorldContract.resolveCanonicalMaterializationCandidates(policyMode)[1]
end

local function annotateManifestSource(manifestSource, metadata)
    if type(manifestSource) ~= "table" or type(metadata) ~= "table" then
        return manifestSource
    end
    for key, value in pairs(metadata) do
        manifestSource[key] = value
    end
    return manifestSource
end

function CanonicalWorldContract.loadCanonicalManifestSource(policyMode, timeoutSeconds, options)
    local canonicalFamily = CanonicalWorldContract.resolveCanonicalManifestFamily(policyMode)
    if type(options) == "table" and type(options.routeCatalogName) == "string" then
        local routeLane = options.routeLane or "active"
        local routeStepIndex = options.routeStepIndex or 0
        local routeCatalogHandle =
            ManifestLoader.LoadNamedRouteCatalogHandle(options.routeCatalogName, timeoutSeconds, options)
        local manifestSource =
            routeCatalogHandle:LoadLaneRuntimeHandle(routeStepIndex, routeLane, timeoutSeconds, options)
        if type(manifestSource) == "table" and type(routeCatalogHandle.LoadLaneSummary) == "function" then
            function manifestSource:LoadLaneSummary(stepIndex, laneName)
                return routeCatalogHandle:LoadLaneSummary(stepIndex, laneName)
            end
        end
        annotateManifestSource(manifestSource, {
            manifestSourceKind = "route_catalog",
            manifestSourceName = options.routeCatalogName,
            manifestFamily = canonicalFamily,
            routeCatalogName = options.routeCatalogName,
            routeLane = routeLane,
            routeStepIndex = routeStepIndex,
        })
        return manifestSource, options.routeCatalogName, canonicalFamily
    end

    local candidates = CanonicalWorldContract.resolveCanonicalMaterializationCandidates(policyMode)
    local lastError = nil
    for _, manifestIndexName in ipairs(candidates) do
        local ok, handle =
            pcall(ManifestLoader.LoadNamedShardedSampleHandle, manifestIndexName, timeoutSeconds, options)
        if ok then
            annotateManifestSource(handle, {
                manifestSourceKind = "canonical_manifest",
                manifestSourceName = manifestIndexName,
                manifestFamily = canonicalFamily,
            })
            return handle, manifestIndexName, canonicalFamily
        end
        lastError = handle
    end

    if lastError ~= nil then
        error(lastError)
    end

    error("Canonical world contract did not resolve any manifest materialization candidates")
end

function CanonicalWorldContract.resolveCanonicalAnchor(manifestSource, loadRadius, loadCenter)
    return AustinSpawn.resolveCanonicalAnchorValues(manifestSource, loadRadius, loadCenter)
end

function CanonicalWorldContract.resolveBoundedEnvelope(manifestSource, loadRadius, loadCenter)
    local anchor = CanonicalWorldContract.resolveCanonicalAnchor(manifestSource, loadRadius, loadCenter)
    local selectedChunks = anchor.selectedChunks or {}
    local chunkIds = table.create(#selectedChunks)
    for index, chunk in ipairs(selectedChunks) do
        chunkIds[index] = chunk.id
    end

    return {
        manifestFamily = CanonicalWorldContract.resolveCanonicalManifestFamily(),
        manifestSourceKind = manifestSource.manifestSourceKind or "canonical_manifest",
        manifestSourceName = manifestSource.manifestSourceName,
        routeCatalogName = manifestSource.routeCatalogName,
        routeLane = manifestSource.routeLane,
        routeStepIndex = manifestSource.routeStepIndex,
        anchor = anchor,
        focusPoint = anchor.focusPoint,
        spawnPoint = anchor.spawnPoint,
        lookTarget = anchor.lookTarget,
        selectedChunks = selectedChunks,
        chunkIds = chunkIds,
    }
end

return CanonicalWorldContract
