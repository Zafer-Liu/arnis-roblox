return function()
    local Workspace = game:GetService("Workspace")
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    Workspace.Terrain:Clear()

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "ShellTerrainFillClearance",
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
                        id = "shell_fill_clearance_building",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 32, z = 0 },
                            { x = 32, z = 24 },
                            { x = 0, z = 24 },
                        },
                        baseY = 10,
                        height = 20,
                        levels = 1,
                        roof = "flat",
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

    local worldRootName = "GeneratedWorld_ShellTerrainFillClearance"
    local ok, err = xpcall(function()
        ImportService.ImportManifest(manifest, {
            clearFirst = true,
            worldRootName = worldRootName,
            config = {
                BuildingMode = "shellMesh",
                TerrainMode = "none",
                RoadMode = "none",
                WaterMode = "none",
                LanduseMode = "none",
                EnableRoomInteriors = false,
            },
        })

        local edgeSampleCenter = Vector3.new(2, 10, 12)
        local edgeRegion = Region3.new(
            edgeSampleCenter - Vector3.new(2, 2, 2),
            edgeSampleCenter + Vector3.new(2, 2, 2)
        ):ExpandToGrid(4)
        local edgeMaterials, edgeOccupancies = Workspace.Terrain:ReadVoxels(edgeRegion, 4)

        Assert.truthy(
            edgeOccupancies[1][1][1] <= 0.01,
            "expected shell terrain fill to stay clear of the wall-adjacent boundary voxel"
        )
        Assert.equal(
            edgeMaterials[1][1][1],
            Enum.Material.Air,
            "expected wall-adjacent boundary voxel to remain air"
        )

        local interiorSampleCenter = Vector3.new(6, 10, 12)
        local interiorRegion = Region3.new(
            interiorSampleCenter - Vector3.new(2, 2, 2),
            interiorSampleCenter + Vector3.new(2, 2, 2)
        ):ExpandToGrid(4)
        local interiorMaterials, interiorOccupancies = Workspace.Terrain:ReadVoxels(interiorRegion, 4)

        Assert.truthy(
            interiorOccupancies[1][1][1] > 0.01,
            "expected interior shell terrain fill to remain present away from the shell boundary"
        )
        Assert.equal(
            interiorMaterials[1][1][1],
            Enum.Material.Concrete,
            "expected interior shell terrain fill to keep the building wall material"
        )
    end, debug.traceback)

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    if worldRoot then
        worldRoot:Destroy()
    end
    Workspace.Terrain:Clear()

    if not ok then
        error(err, 0)
    end
end
