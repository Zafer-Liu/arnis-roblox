return function()
    local Workspace = game:GetService("Workspace")
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "PlayVisibleShellReadableCues",
            generator = "test",
            source = "unit",
            metersPerStud = 0.3,
            chunkSizeStuds = 256,
            totalFeatures = 1,
        },
        chunks = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                terrain = {
                    cellSizeStuds = 16,
                    width = 16,
                    depth = 16,
                    heights = table.create(16 * 16, 0),
                    material = "Grass",
                },
                roads = {},
                rails = {},
                buildings = {
                    {
                        id = "medium_play_visible_office",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 28, z = 0 },
                            { x = 28, z = 18 },
                            { x = 0, z = 18 },
                        },
                        baseY = 0,
                        height = 24,
                        levels = 5,
                        roof = "hipped",
                        usage = "office",
                        material = "Concrete",
                    },
                },
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }

    local worldRootName = "GeneratedWorld_PlayVisibleShellReadableCues"
    ImportService.ImportManifest(manifest, {
        clearFirst = true,
        worldRootName = worldRootName,
        config = {
            BuildingMode = "shellMesh",
            TerrainMode = "none",
            RoadMode = "none",
            WaterMode = "none",
            LanduseMode = "none",
        },
    })

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    Assert.truthy(worldRoot, "expected play-visible shell readability world root")

    local building =
        worldRoot:FindFirstChild("0_0"):FindFirstChild("Buildings"):FindFirstChild("medium_play_visible_office")
    Assert.truthy(building, "expected play-visible shell office")

    local detailFolder = building:FindFirstChild("Detail")
    Assert.truthy(detailFolder, "expected detail folder")

    Assert.truthy(
        (detailFolder:GetAttribute("ArnisFacadeBeltlineCount") or 0) >= 4,
        "expected play-visible shell path to keep facade beltline readability cues"
    )
    Assert.truthy(
        (detailFolder:GetAttribute("ArnisMergedShellRooflineCueCount") or 0) >= 4,
        "expected play-visible shell path to keep roofline readability cues"
    )
    Assert.truthy(
        (detailFolder:GetAttribute("ArnisCornerAccentCount") or 0) >= 4,
        "expected play-visible shell path to keep corner accent readability cues"
    )
    Assert.truthy(
        (detailFolder:GetAttribute("ArnisMergedShellDoorCueCount") or 0) >= 1,
        "expected play-visible shell path to keep a bounded street-facing door cue"
    )
    Assert.truthy(
        (detailFolder:GetAttribute("ArnisMergedShellStreetFacadeCueCount") or 0) >= 4,
        "expected play-visible shell path to keep bounded street facade readability cues"
    )
    Assert.truthy(
        (detailFolder:GetAttribute("ArnisMergedShellWindowPaneCueCount") or 0) >= 1,
        "expected play-visible shell path to keep bounded window pane readability cues"
    )

    worldRoot:Destroy()
end
