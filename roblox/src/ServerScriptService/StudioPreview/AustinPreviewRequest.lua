local Workspace = game:GetService("Workspace")

local AustinPreviewRequest = {}

AustinPreviewRequest.MODE_PREVIEW = "preview"
AustinPreviewRequest.MODE_FULL_BAKE = "full_bake"
AustinPreviewRequest.ROUTE_CATALOG_ATTR = "VertigoRouteCatalogName"
AustinPreviewRequest.ROUTE_LANE_ATTR = "VertigoRouteLane"
AustinPreviewRequest.ROUTE_STEP_INDEX_ATTR = "VertigoRouteStepIndex"

local FULL_BAKE_MODE_ALIASES = table.freeze({
    export = true,
    full = true,
    authoritative = true,
    full_bake = true,
})

function AustinPreviewRequest.Normalize(request)
    local mode = AustinPreviewRequest.MODE_PREVIEW
    local debugHelpers = false
    local routeCatalogName = nil
    local routeLane = nil
    local routeStepIndex = nil

    if type(request) == "table" then
        local requestedMode = request.mode or request.buildMode
        if type(requestedMode) == "string" and FULL_BAKE_MODE_ALIASES[requestedMode] == true then
            mode = AustinPreviewRequest.MODE_FULL_BAKE
        end

        debugHelpers = request.debugHelpers == true or request.showDebugHelpers == true

        local requestedRouteCatalogName = request.routeCatalogName or request.routeCatalog
        if type(requestedRouteCatalogName) == "string" and requestedRouteCatalogName ~= "" then
            routeCatalogName = requestedRouteCatalogName
        end

        local requestedRouteLane = request.routeLane or request.lane
        if type(requestedRouteLane) == "string" and requestedRouteLane ~= "" then
            routeLane = requestedRouteLane
        end

        local requestedRouteStepIndex = request.routeStepIndex or request.stepIndex
        if type(requestedRouteStepIndex) == "number" then
            routeStepIndex = math.floor(requestedRouteStepIndex)
        end
    end

    if routeCatalogName == nil then
        local workspaceRouteCatalogName = Workspace:GetAttribute(AustinPreviewRequest.ROUTE_CATALOG_ATTR)
        if type(workspaceRouteCatalogName) == "string" and workspaceRouteCatalogName ~= "" then
            routeCatalogName = workspaceRouteCatalogName
        end
    end

    if routeLane == nil then
        local workspaceRouteLane = Workspace:GetAttribute(AustinPreviewRequest.ROUTE_LANE_ATTR)
        if type(workspaceRouteLane) == "string" and workspaceRouteLane ~= "" then
            routeLane = workspaceRouteLane
        end
    end

    if routeStepIndex == nil then
        local workspaceRouteStepIndex = Workspace:GetAttribute(AustinPreviewRequest.ROUTE_STEP_INDEX_ATTR)
        if type(workspaceRouteStepIndex) == "number" then
            routeStepIndex = math.floor(workspaceRouteStepIndex)
        end
    end

    return {
        mode = mode,
        debugHelpers = debugHelpers,
        routeCatalogName = routeCatalogName,
        routeLane = routeLane,
        routeStepIndex = routeStepIndex,
    }
end

function AustinPreviewRequest.ResolveLoadRadius(request, defaultLoadRadius)
    local normalizedRequest = AustinPreviewRequest.Normalize(request)
    if normalizedRequest.mode == AustinPreviewRequest.MODE_FULL_BAKE then
        return nil
    end

    return defaultLoadRadius
end

local function normalizeRouteChunkId(chunkId)
    if type(chunkId) ~= "string" or chunkId == "" then
        return nil
    end
    local _, separatorIndex = string.find(chunkId, ":", 1, true)
    if separatorIndex == nil then
        return chunkId
    end
    local normalized = string.sub(chunkId, separatorIndex + 1)
    if normalized == "" then
        return nil
    end
    return normalized
end

local function normalizeRouteChunkIds(laneSummary)
    if type(laneSummary) ~= "table" then
        return {}
    end

    local normalizedChunkIds = {}
    local seenChunkIds = {}
    local chunkRefs = laneSummary.chunk_refs
    if type(chunkRefs) == "table" then
        for _, chunkRef in ipairs(chunkRefs) do
            local normalizedChunkId = normalizeRouteChunkId(type(chunkRef) == "table" and chunkRef.chunk_id or nil)
            if normalizedChunkId ~= nil and not seenChunkIds[normalizedChunkId] then
                seenChunkIds[normalizedChunkId] = true
                normalizedChunkIds[#normalizedChunkIds + 1] = normalizedChunkId
            end
        end
    end

    if #normalizedChunkIds > 0 then
        return normalizedChunkIds
    end

    local chunkIds = laneSummary.chunk_ids
    if type(chunkIds) ~= "table" then
        return {}
    end
    for _, chunkId in ipairs(chunkIds) do
        local normalizedChunkId = normalizeRouteChunkId(chunkId)
        if normalizedChunkId ~= nil and not seenChunkIds[normalizedChunkId] then
            seenChunkIds[normalizedChunkId] = true
            normalizedChunkIds[#normalizedChunkIds + 1] = normalizedChunkId
        end
    end
    return normalizedChunkIds
end

function AustinPreviewRequest.SelectChunkIds(handle, focusPoint, request, defaultLoadRadius)
    local normalizedRequest = AustinPreviewRequest.Normalize(request)
    if normalizedRequest.routeLane ~= nil and type(handle.LoadLaneSummary) == "function" then
        local laneSummary = handle:LoadLaneSummary(normalizedRequest.routeStepIndex or 0, normalizedRequest.routeLane)
        return normalizeRouteChunkIds(laneSummary), nil
    end

    local loadRadius = AustinPreviewRequest.ResolveLoadRadius(normalizedRequest, defaultLoadRadius)
    return handle:GetChunkIdsWithinRadius(focusPoint, loadRadius), loadRadius
end

return AustinPreviewRequest
