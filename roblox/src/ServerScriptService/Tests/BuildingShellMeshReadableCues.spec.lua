return function()
    local Workspace = game:GetService("Workspace")
    local SceneAudit = require(script.Parent.Parent.ImportService.SceneAudit)
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "ShellMeshReadableCues",
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
                        id = "tall_hipped_annex",
                        name = "Tall Hipped Annex",
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

    local worldRootName = "GeneratedWorld_BuildingShellMeshReadableCues"
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
    Assert.truthy(worldRoot, "expected shell mesh readability test world root")

    local building = worldRoot:FindFirstChild("0_0"):FindFirstChild("Buildings"):FindFirstChild("tall_hipped_annex")
    Assert.truthy(building, "expected tall hipped annex")

    local shellFolder = building:FindFirstChild("Shell")
    local detailFolder = building:FindFirstChild("Detail")
    Assert.truthy(shellFolder, "expected shell folder on tall hipped annex")
    Assert.truthy(detailFolder, "expected detail folder on tall hipped annex")

    local roofMesh = shellFolder:FindFirstChild("tall_hipped_annex_roof_mesh")
    local roofClosureDeck = shellFolder:FindFirstChild("tall_hipped_annex_roof_closure")
    Assert.truthy(roofMesh, "expected shell mesh roof to stay explicit")
    Assert.truthy(roofClosureDeck, "expected shaped roof closure deck")
    Assert.equal(
        roofClosureDeck:GetAttribute("ArnisRoofClosureDeck"),
        true,
        "expected the closure deck to remain internal support"
    )

    local cornerAccents = 0
    local beltlines = 0
    local rooflineCues = 0
    local perimeterCues = 0
    local streetFacadeCues = 0
    local streetFacadeCueYs = {}
    for _, child in ipairs(detailFolder:GetDescendants()) do
        if child:IsA("Part") and child.Name == "CornerAccent" then
            cornerAccents += 1
        elseif child:IsA("Part") and child.Name == "FacadeBeltline" then
            beltlines += 1
        elseif child:IsA("Part") and child.Name == "MergedShellRooflineCue" then
            rooflineCues += 1
        elseif child:IsA("Part") and child.Name == "MergedShellPerimeterCue" then
            perimeterCues += 1
        elseif child:IsA("Part") and child.Name == "MergedShellStreetFacadeCue" then
            streetFacadeCues += 1
            streetFacadeCueYs[#streetFacadeCueYs + 1] = child.Position.Y
        end
    end

    Assert.equal(cornerAccents, 4, "expected one corner accent per footprint corner")
    Assert.equal(beltlines, 4, "expected one facade beltline per footprint edge")
    Assert.equal(rooflineCues, 4, "expected one roofline cue per footprint edge")
    Assert.equal(perimeterCues, 4, "expected one perimeter cue per footprint corner")
    Assert.equal(streetFacadeCues, 4, "expected one street-level facade cue per footprint edge")
    for _, cueY in ipairs(streetFacadeCueYs) do
        Assert.truthy(
            cueY >= 1.5 and cueY <= 5,
            "expected street-level facade cues to sit low enough to read from street level"
        )
    end

    Assert.equal(
        detailFolder:GetAttribute("ArnisMergedShellRooflineCueCount"),
        4,
        "expected roofline cue attribute to reflect emitted roofline geometry"
    )
    Assert.equal(
        detailFolder:GetAttribute("ArnisMergedShellPerimeterCueCount"),
        4,
        "expected perimeter cue attribute to reflect emitted corner geometry"
    )
    Assert.equal(
        detailFolder:GetAttribute("ArnisMergedShellStreetFacadeCueCount"),
        4,
        "expected street-level facade cue attribute to reflect emitted facade geometry"
    )

    local sceneSummary = SceneAudit.summarizeWorld(worldRoot)
    Assert.truthy(
        sceneSummary.buildingVisibleDetailPartCount >= 20,
        "expected merged shell cues to add visible detail parts"
    )

    worldRoot:Destroy()
end
