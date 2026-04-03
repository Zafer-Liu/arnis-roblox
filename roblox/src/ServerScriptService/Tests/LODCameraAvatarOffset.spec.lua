return function()
    local Players = game:GetService("Players")
    local Workspace = game:GetService("Workspace")
    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)
    local Assert = require(script.Parent.Assert)

    local testManifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "LODCameraAvatarOffsetTest",
            generator = "test",
            source = "test",
            metersPerStud = 0.3,
            chunkSizeStuds = 100,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            totalFeatures = 1,
        },
        chunks = {
            {
                id = "lod_chunk",
                originStuds = { x = 0, y = 0, z = 0 },
                buildings = {
                    {
                        id = "b1",
                        footprint = { { x = 10, z = 10 }, { x = 20, z = 10 }, { x = 20, z = 20 } },
                        baseY = 0,
                        height = 10,
                        roof = "flat",
                        rooms = {
                            {
                                id = "room_1",
                                name = "Room 1",
                                footprint = {
                                    { x = 10, z = 10 },
                                    { x = 20, z = 10 },
                                    { x = 20, z = 20 },
                                },
                                floorY = 0,
                                height = 0.2,
                            },
                        },
                    },
                },
                roads = {},
                water = {},
                props = {},
                landuse = {},
            },
        },
    }

    local testOptions = {
        worldRootName = "LODCameraAvatarOffsetWorld",
        config = {
            StreamingEnabled = true,
            StreamingTargetRadius = 1000,
            HighDetailRadius = 500,
            ChunkSizeStuds = 100,
            BuildingMode = "shellMesh",
            RoadMode = "mesh",
            TerrainMode = "none",
            WaterMode = "mesh",
            LanduseMode = "terrain",
        },
    }

    local function getChunkEntry()
        return ChunkLoader.GetChunkEntry("lod_chunk", testOptions.worldRootName)
    end

    local function getPrimaryLodGroup(kind)
        local chunkEntry = getChunkEntry()
        Assert.truthy(chunkEntry, "expected chunk entry")
        Assert.truthy(chunkEntry.lodGroups, "expected chunk lod groups")
        local groups = chunkEntry.lodGroups[kind]
        Assert.truthy(groups and #groups >= 1, "expected chunk lod group for " .. kind)
        return groups[1]
    end

    local rootPart = nil
    local originalRootCFrame = nil

    local function cleanup(camera, originalCamera)
        StreamingService.Stop()
        ChunkLoader.Clear()
        local worldRoot = Workspace:FindFirstChild(testOptions.worldRootName)
        if worldRoot then
            worldRoot:Destroy()
        end
        if rootPart and originalRootCFrame then
            rootPart.CFrame = originalRootCFrame
        end
        if camera then
            camera:Destroy()
        end
        Workspace.CurrentCamera = originalCamera
    end

    ChunkLoader.Clear()
    local originalCamera = Workspace.CurrentCamera
    local camera = Instance.new("Camera")
    camera.Name = "LODCameraAvatarOffsetCamera"
    camera.CFrame = CFrame.new(4000, 100, 4000)
    camera.Parent = Workspace
    Workspace.CurrentCamera = camera

    local player = Players:GetPlayers()[1]
    Assert.truthy(player, "expected a player in play mode")
    local character = player.Character or player.CharacterAdded:Wait()
    rootPart = character:WaitForChild("HumanoidRootPart")
    originalRootCFrame = rootPart.CFrame
    rootPart.CFrame = CFrame.new(0, originalRootCFrame.Position.Y, 0)

    local ok, err = xpcall(function()
        StreamingService.Start(testManifest, testOptions)
        StreamingService.Update(Vector3.new(0, 0, 0))

        Assert.equal(
            getPrimaryLodGroup("detail"):GetAttribute("ArnisLodVisible"),
            true,
            "expected detail visible when avatar focus is nearby"
        )
        Assert.equal(
            getPrimaryLodGroup("interior"):GetAttribute("ArnisLodVisible"),
            true,
            "expected interior visible when avatar focus is nearby"
        )

        camera.CFrame = CFrame.new(5000, 100, 5000)
        task.wait(2.2)

        Assert.equal(
            getPrimaryLodGroup("detail"):GetAttribute("ArnisLodVisible"),
            true,
            "expected detail to stay visible when camera moves away from the avatar focus"
        )
        Assert.equal(
            getPrimaryLodGroup("interior"):GetAttribute("ArnisLodVisible"),
            true,
            "expected interior to stay visible when camera moves away from the avatar focus"
        )

        rootPart.CFrame = CFrame.new(750, originalRootCFrame.Position.Y, 750)
        camera.CFrame = CFrame.new(16, 20, 16)
        StreamingService.Update(Vector3.new(750, 0, 750))
        Assert.equal(
            getPrimaryLodGroup("detail"):GetAttribute("ArnisLodVisible"),
            true,
            "expected immediate streaming refresh to honor nearby camera detail visibility even when avatar focus moved away"
        )
        Assert.equal(
            getPrimaryLodGroup("interior"):GetAttribute("ArnisLodVisible"),
            false,
            "expected immediate streaming refresh to keep interiors gated when only the camera is nearby"
        )

        camera.CFrame = CFrame.new(5000, 100, 5000)
        task.wait(2.2)

        Assert.equal(
            getPrimaryLodGroup("detail"):GetAttribute("ArnisLodVisible"),
            false,
            "expected detail to hide once the live player root moves away"
        )
        Assert.equal(
            getPrimaryLodGroup("interior"):GetAttribute("ArnisLodVisible"),
            false,
            "expected interior to hide once the live player root moves away"
        )

        StreamingService.Update(Vector3.new(750, 0, 750))
        task.wait(2.2)

        Assert.equal(
            getPrimaryLodGroup("detail"):GetAttribute("ArnisLodVisible"),
            false,
            "expected detail to hide once avatar focus also moves away"
        )
        Assert.equal(
            getPrimaryLodGroup("interior"):GetAttribute("ArnisLodVisible"),
            false,
            "expected interior to hide once avatar focus also moves away"
        )
    end, debug.traceback)

    cleanup(camera, originalCamera)

    if not ok then
        error(err, 0)
    end
end
