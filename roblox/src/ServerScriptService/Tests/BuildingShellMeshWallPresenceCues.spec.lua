return function()
    local Workspace = game:GetService("Workspace")
    local SceneAudit = require(script.Parent.Parent.ImportService.SceneAudit)
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "ShellMeshWallPresenceCues",
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
                        id = "tall_shell_wall_presence",
                        name = "Tall Shell Wall Presence",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 28, z = 0 },
                            { x = 28, z = 18 },
                            { x = 0, z = 18 },
                        },
                        baseY = 0,
                        height = 36,
                        levels = 8,
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

    local worldRootName = "GeneratedWorld_BuildingShellMeshWallPresenceCues"
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
    Assert.truthy(worldRoot, "expected shell mesh wall-presence world root")

    local building =
        worldRoot:FindFirstChild("0_0"):FindFirstChild("Buildings"):FindFirstChild("tall_shell_wall_presence")
    Assert.truthy(building, "expected tall shell wall presence building")

    local detailFolder = building:FindFirstChild("Detail")
    Assert.truthy(detailFolder, "expected detail folder on tall shell wall presence building")

    local wallPresenceCues = 0
    local rooflineCues = 0
    local perimeterCues = 0
    for _, child in ipairs(detailFolder:GetChildren()) do
        if child:IsA("Part") and child.Name == "MergedShellWallPresenceCue" then
            wallPresenceCues += 1
        elseif child:IsA("Part") and child.Name == "MergedShellRooflineCue" then
            rooflineCues += 1
        elseif child:IsA("Part") and child.Name == "MergedShellPerimeterCue" then
            perimeterCues += 1
        end
    end

    Assert.equal(wallPresenceCues, 4, "expected one wall-presence cue per outer footprint edge")
    Assert.equal(rooflineCues, 4, "expected roofline cues to stay edge-bounded")
    Assert.equal(perimeterCues, 4, "expected perimeter cues to stay corner-bounded")
    Assert.equal(
        detailFolder:GetAttribute("ArnisMergedShellWallPresenceCueCount"),
        4,
        "expected merged shell wall-presence cue count to match emitted geometry"
    )
    Assert.equal(
        detailFolder:GetAttribute("ArnisMergedShellWallStripCount"),
        4,
        "expected merged shell wall strip count to match emitted geometry"
    )

    local sceneSummary = SceneAudit.summarizeWorld(worldRoot)
    Assert.truthy(
        sceneSummary.buildingVisibleDetailPartCount >= 20,
        "expected merged shell wall-presence cues to raise visible detail part count"
    )

    worldRoot:Destroy()
end
