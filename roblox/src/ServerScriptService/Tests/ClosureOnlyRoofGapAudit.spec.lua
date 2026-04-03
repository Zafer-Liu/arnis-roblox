return function()
    local Workspace = game:GetService("Workspace")

    local ImportService = require(script.Parent.Parent.ImportService)
    local SceneAudit = require(script.Parent.Parent.ImportService.SceneAudit)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "ClosureOnlyRoofGapAudit",
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
                        id = "closure_only_gabled",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 32, z = 0 },
                            { x = 32, z = 16 },
                            { x = 16, z = 16 },
                            { x = 16, z = 32 },
                            { x = 0, z = 32 },
                        },
                        baseY = 0,
                        height = 18,
                        roof = "gabled",
                        usage = "residential",
                        material = "Brick",
                    },
                },
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }

    local worldRootName = "GeneratedWorld_ClosureOnlyRoofGapAudit"
    ImportService.ImportManifest(manifest, {
        clearFirst = true,
        worldRootName = worldRootName,
        config = {
            BuildingMode = "shellParts",
            TerrainMode = "none",
            RoadMode = "none",
            WaterMode = "none",
            LanduseMode = "none",
        },
    })

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    Assert.truthy(worldRoot, "expected closure-only roof audit world root")

    local summary = SceneAudit.summarizeWorld(worldRoot)
    Assert.equal(
        summary.buildingModelsWithClosureOnlyRoofGap,
        1,
        "expected closure-only shaped roof fallback to be surfaced as a dedicated roof gap"
    )
    Assert.equal(
        summary.buildingModelsWithNoRoofEvidence,
        1,
        "expected closure-only shaped roof fallback to remain visible in legacy no-roof counts"
    )
    Assert.equal(#summary.buildingClosureOnlyRoofGapDetails, 1, "expected one closure-only roof gap detail row")
    Assert.equal(
        summary.buildingClosureOnlyRoofGapDetails[1].sourceId,
        "closure_only_gabled",
        "expected closure-only roof gap detail to preserve the building source id"
    )

    worldRoot:Destroy()
end
