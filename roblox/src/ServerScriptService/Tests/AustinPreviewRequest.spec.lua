return function()
    local Assert = require(script.Parent.Assert)
    local AustinPreviewRequest = require(script.Parent.Parent.StudioPreview.AustinPreviewRequest)

    local defaultRequest = AustinPreviewRequest.Normalize(nil)
    Assert.equal(
        defaultRequest.mode,
        AustinPreviewRequest.MODE_PREVIEW,
        "expected nil request to default to preview mode"
    )
    Assert.equal(
        defaultRequest.debugHelpers,
        false,
        "expected nil request to keep preview helper geometry disabled by default"
    )

    local previewRequest = AustinPreviewRequest.Normalize({
        mode = "preview",
    })
    Assert.equal(
        previewRequest.mode,
        AustinPreviewRequest.MODE_PREVIEW,
        "expected explicit preview mode to stay preview"
    )
    Assert.equal(
        previewRequest.debugHelpers,
        false,
        "expected plain preview requests not to enable debug helper geometry"
    )

    local helperRequest = AustinPreviewRequest.Normalize({
        mode = "preview",
        debugHelpers = true,
    })
    Assert.equal(helperRequest.debugHelpers, true, "expected preview requests to opt into helper geometry explicitly")

    local exportRequest = AustinPreviewRequest.Normalize({
        mode = "export",
    })
    Assert.equal(
        exportRequest.mode,
        AustinPreviewRequest.MODE_FULL_BAKE,
        "expected export mode to normalize to authoritative full-bake mode"
    )

    local fullBakeRequest = AustinPreviewRequest.Normalize({
        mode = "full_bake",
    })
    Assert.equal(fullBakeRequest.mode, AustinPreviewRequest.MODE_FULL_BAKE, "expected full_bake mode to stay full_bake")

    local selectionCalls = {}
    local handle = {}

    function handle:GetChunkIdsWithinRadius(_focusPoint, radius)
        selectionCalls[#selectionCalls + 1] = if radius == nil then "full" else tostring(radius)
        if radius == nil then
            return { "0_0", "1_0", "2_0" }
        end
        return { "0_0", "1_0" }
    end

    local previewIds, previewRadius = AustinPreviewRequest.SelectChunkIds(handle, nil, { mode = "preview" }, 1500)
    Assert.equal(previewRadius, 1500, "expected preview requests to keep the default preview radius")
    Assert.equal(#previewIds, 2, "expected preview requests to keep radius-limited chunk selection")
    Assert.equal(selectionCalls[1], "1500", "expected preview requests to pass the preview radius to the handle")

    local fullBakeIds, fullBakeRadius = AustinPreviewRequest.SelectChunkIds(handle, nil, { mode = "full_bake" }, 1500)
    Assert.equal(fullBakeRadius, nil, "expected full-bake requests to clear the preview radius")
    Assert.equal(#fullBakeIds, 3, "expected full-bake requests to select all chunk ids")
    Assert.equal(selectionCalls[2], "full", "expected full-bake requests to use nil radius selection")

    local exportIds, exportRadius = AustinPreviewRequest.SelectChunkIds(handle, nil, { mode = "export" }, 1500)
    Assert.equal(exportRadius, nil, "expected export requests to inherit full-bake radius semantics")
    Assert.equal(#exportIds, 3, "expected export requests to select all chunk ids")
    Assert.equal(selectionCalls[3], "full", "expected export requests to use nil radius selection")

    local routeSelectionCalls = {}
    local routeHandle = {}

    function routeHandle:LoadLaneSummary(stepIndex, laneName)
        routeSelectionCalls[#routeSelectionCalls + 1] = ("lane:%s:%s"):format(tostring(stepIndex), tostring(laneName))
        return {
            chunk_ids = { "austin:1_0", "austin:2_0" },
            chunk_refs = {
                { scene_id = "austin", chunk_id = "1_0" },
                { scene_id = "austin", chunk_id = "2_0" },
            },
        }
    end

    function routeHandle:GetChunkIdsWithinRadius(_focusPoint, radius)
        routeSelectionCalls[#routeSelectionCalls + 1] = if radius == nil then "full" else tostring(radius)
        return { "0_0" }
    end

    local routeRequest = AustinPreviewRequest.Normalize({
        mode = "preview",
        routeCatalogName = "PlanetaryRouteBundle.route-catalog",
        routeLane = "active",
        routeStepIndex = 2,
    })
    Assert.equal(
        routeRequest.routeCatalogName,
        "PlanetaryRouteBundle.route-catalog",
        "expected route catalog name to normalize through preview requests"
    )
    Assert.equal(routeRequest.routeLane, "active", "expected route lane to normalize through preview requests")
    Assert.equal(routeRequest.routeStepIndex, 2, "expected route step index to normalize through preview requests")

    local routeChunkIds, routeRadius = AustinPreviewRequest.SelectChunkIds(routeHandle, nil, routeRequest, 1500)
    Assert.equal(routeRadius, nil, "expected route lane selection to bypass radius-based preview selection")
    Assert.equal(#routeChunkIds, 2, "expected route lane selection to use route catalog chunk ids")
    Assert.equal(routeChunkIds[1], "1_0", "expected route lane selection to strip scene-qualified chunk ids")
    Assert.equal(routeChunkIds[2], "2_0", "expected route lane selection to prefer plain chunk ids for manifest handles")
    Assert.equal(
        routeSelectionCalls[1],
        "lane:2:active",
        "expected route lane selection to prefer route-catalog lane summaries over radius selection"
    )
end
