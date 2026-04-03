return function()
    local Workspace = game:GetService("Workspace")
    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "LODGroupFootprintVisibility",
            generator = "test",
            source = "test",
            metersPerStud = 1,
            chunkSizeStuds = 1200,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            totalFeatures = 1,
        },
        chunks = {
            {
                id = "lod_chunk",
                originStuds = { x = 0, y = 0, z = 0 },
                roads = {
                    {
                        id = "edge_road",
                        kind = "secondary",
                        material = "Asphalt",
                        widthStuds = 18,
                        lit = true,
                        oneway = true,
                        hasSidewalk = false,
                        points = {
                            { x = 0, y = 0, z = 20 },
                            { x = 1000, y = 0, z = 20 },
                        },
                    },
                },
                buildings = {},
                water = {},
                props = {},
                landuse = {},
            },
        },
    }

    local testOptions = {
        worldRootName = "LODGroupFootprintVisibilityWorld",
        config = {
            StreamingEnabled = true,
            StreamingTargetRadius = 1000,
            HighDetailRadius = 300,
            ChunkSizeStuds = 1200,
            BuildingMode = "none",
            RoadMode = "mesh",
            TerrainMode = "none",
            WaterMode = "none",
            LanduseMode = "none",
        },
    }

    local function getDetailGroup()
        local chunkEntry = ChunkLoader.GetChunkEntry("lod_chunk", testOptions.worldRootName)
        Assert.truthy(chunkEntry, "expected chunk entry")
        Assert.truthy(chunkEntry.lodGroups and #chunkEntry.lodGroups.detail >= 1, "expected road detail LOD group")
        return chunkEntry.lodGroups.detail[1]
    end

    local function cleanup(camera, originalCamera)
        StreamingService.Stop()
        ChunkLoader.Clear()
        local worldRoot = Workspace:FindFirstChild(testOptions.worldRootName)
        if worldRoot then
            worldRoot:Destroy()
        end
        if camera then
            camera:Destroy()
        end
        Workspace.CurrentCamera = originalCamera
    end

    ChunkLoader.Clear()
    local originalCamera = Workspace.CurrentCamera
    local camera = Instance.new("Camera")
    camera.Name = "LODGroupFootprintVisibilityCamera"
    camera.CFrame = CFrame.new(4000, 100, 4000)
    camera.Parent = Workspace
    Workspace.CurrentCamera = camera

    local ok, err = xpcall(function()
        StreamingService.Start(manifest, testOptions)
        StreamingService.Update(Vector3.new(0, 0, 0))
        task.wait(2.2)

        Assert.equal(
            getDetailGroup():GetAttribute("ArnisLodVisible"),
            true,
            "expected edge-visible detail groups to stay visible when the player is near the footprint edge"
        )
    end, debug.traceback)

    cleanup(camera, originalCamera)

    if not ok then
        error(err, 0)
    end
end
