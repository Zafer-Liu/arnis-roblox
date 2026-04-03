return function()
    local Workspace = game:GetService("Workspace")

    local ImportService = require(script.Parent.Parent.ImportService)
    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local SceneAudit = require(script.Parent.Parent.ImportService.SceneAudit)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)
    local Assert = require(script.Parent.Assert)

    local worldRootName = "GeneratedWorld_PlayStreamingRoomShellParity"
    local interiorSampleCenter = Vector3.new(16, 12, 12)

    local terrainHeights = table.create(16 * 16, 0)
    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "PlayStreamingRoomShellParity",
            generator = "test",
            source = "unit",
            metersPerStud = 1.0,
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
                    heights = terrainHeights,
                    material = "Grass",
                },
                roads = {},
                rails = {},
                buildings = {
                    {
                        id = "roomed_shell_building",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 32, z = 0 },
                            { x = 32, z = 24 },
                            { x = 0, z = 24 },
                        },
                        baseY = 12,
                        height = 20,
                        levels = 2,
                        roof = "flat",
                        material = "Concrete",
                        rooms = {
                            {
                                id = "room_1",
                                name = "Room 1",
                                footprint = {
                                    { x = 0, z = 0 },
                                    { x = 32, z = 0 },
                                    { x = 32, z = 24 },
                                    { x = 0, z = 24 },
                                },
                                floorY = 0,
                                height = 10,
                            },
                        },
                    },
                },
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }

    local config = {
        BuildingMode = "shellMesh",
        TerrainMode = "voxel",
        RoadMode = "none",
        WaterMode = "none",
        LanduseMode = "none",
        EnableRoomInteriors = true,
        StreamingEnabled = true,
        StreamingTargetRadius = 512,
        HighDetailRadius = 256,
        ChunkSizeStuds = 256,
    }

    local function readVoxel(center)
        local region = Region3.new(center - Vector3.new(2, 2, 2), center + Vector3.new(2, 2, 2)):ExpandToGrid(4)
        local materials, occupancies = Workspace.Terrain:ReadVoxels(region, 4)
        return materials[1][1][1], occupancies[1][1][1]
    end

    local function summarizeChunkWorld()
        local worldRoot = Workspace:FindFirstChild(worldRootName)
        Assert.truthy(worldRoot, "expected generated world root")

        local summary = SceneAudit.summarizeWorld(worldRoot)
        local chunkFolder = worldRoot:FindFirstChild("0_0")
        Assert.truthy(chunkFolder, "expected chunk folder")

        local building = chunkFolder:FindFirstChild("Buildings") and chunkFolder.Buildings:FindFirstChild("roomed_shell_building")
        Assert.truthy(building, "expected streamed building model")

        return {
            summary = summary,
            building = building,
        }
    end

    Workspace.Terrain:Clear()

    local ok, err = xpcall(function()
        ImportService.ImportManifest(manifest, {
            clearFirst = true,
            worldRootName = worldRootName,
            config = config,
        })

        local startupState = summarizeChunkWorld()
        local startupMaterial, startupOccupancy = readVoxel(interiorSampleCenter)

        Assert.truthy(
            ChunkLoader.GetChunkEntry("0_0", worldRootName) ~= nil,
            "expected startup import to register the chunk for streaming reconciliation"
        )
        Assert.equal(
            startupState.summary.buildingModelsWithVisibleShellWalls,
            1,
            "expected startup import to keep visible shell wall evidence for the roomed shell mesh building"
        )
        Assert.equal(
            startupState.summary.buildingModelsWithoutVisibleShellWalls,
            0,
            "expected startup import to avoid visible-wall gaps for the roomed shell mesh building"
        )
        Assert.truthy(
            startupOccupancy <= 0.01,
            "expected startup import to keep the roomed shell interior free of floating terrain fill"
        )
        Assert.equal(
            startupMaterial,
            Enum.Material.Air,
            "expected startup import to leave the roomed shell interior voxel as air"
        )

        StreamingService.Start(manifest, {
            worldRootName = worldRootName,
            config = config,
        })
        StreamingService.Update(Vector3.new(16, 0, 16))

        local streamedState = summarizeChunkWorld()
        local streamedMaterial, streamedOccupancy = readVoxel(interiorSampleCenter)

        Assert.equal(
            streamedState.summary.buildingModelsWithVisibleShellWalls,
            startupState.summary.buildingModelsWithVisibleShellWalls,
            "expected streaming startup to preserve visible shell wall evidence for the roomed shell mesh building"
        )
        Assert.equal(
            streamedState.summary.buildingModelsWithoutVisibleShellWalls,
            startupState.summary.buildingModelsWithoutVisibleShellWalls,
            "expected streaming startup not to introduce new visible-wall gaps"
        )
        Assert.truthy(
            streamedOccupancy <= 0.01,
            "expected streaming startup not to reintroduce floating terrain fill inside the roomed shell"
        )
        Assert.equal(
            streamedMaterial,
            Enum.Material.Air,
            "expected streaming startup to preserve an air interior voxel inside the roomed shell"
        )
    end, debug.traceback)

    StreamingService.Stop()

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    if worldRoot then
        worldRoot:Destroy()
    end
    Workspace.Terrain:Clear()

    if not ok then
        error(err, 0)
    end
end
