local AustinPreviewRequest = {}

AustinPreviewRequest.MODE_PREVIEW = "preview"
AustinPreviewRequest.MODE_FULL_BAKE = "full_bake"

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

function AustinPreviewRequest.SelectChunkIds(handle, focusPoint, request, defaultLoadRadius)
    local normalizedRequest = AustinPreviewRequest.Normalize(request)
    if normalizedRequest.routeLane ~= nil and type(handle.LoadLaneSummary) == "function" then
        local laneSummary = handle:LoadLaneSummary(normalizedRequest.routeStepIndex or 0, normalizedRequest.routeLane)
        return laneSummary.chunk_ids or {}, nil
    end

    local loadRadius = AustinPreviewRequest.ResolveLoadRadius(normalizedRequest, defaultLoadRadius)
    return handle:GetChunkIdsWithinRadius(focusPoint, loadRadius), loadRadius
end

return AustinPreviewRequest
