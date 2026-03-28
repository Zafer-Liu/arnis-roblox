return function()
    local Workspace = game:GetService("Workspace")
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    Workspace.Terrain:Clear()

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "RoomInteriorShellFillTruth",
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
                        id = "room_overlap_building",
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
                        rooms = {
                            {
                                id = "main_room",
                                name = "Main Room",
                                footprint = {
                                    { x = 0, z = 0 },
                                    { x = 32, z = 0 },
                                    { x = 32, z = 24 },
                                    { x = 0, z = 24 },
                                },
                                floorY = 0,
                                height = 0.2,
                                floorMaterial = "WoodPlanks",
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

    local worldRootName = "GeneratedWorld_RoomInteriorShellFillTruth"
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
                EnableRoomInteriors = true,
            },
        })

        local roomSampleCenter = Vector3.new(16, 10.1, 12)
        local region = Region3.new(
            roomSampleCenter - Vector3.new(2, 2, 2),
            roomSampleCenter + Vector3.new(2, 2, 2)
        ):ExpandToGrid(4)
        local materials, occupancies = Workspace.Terrain:ReadVoxels(region, 4)
        local material = materials[1][1][1]
        local occupancy = occupancies[1][1][1]

        Assert.truthy(occupancy <= 0.01, "expected authored room interior to stay free of shell terrain fill")
        Assert.equal(material, Enum.Material.Air, "expected authored room interior voxel to remain air")

        local worldRoot = Workspace:FindFirstChild(worldRootName)
        Assert.truthy(worldRoot, "expected imported world root")

        local chunkFolder = worldRoot:FindFirstChild("0_0")
        Assert.truthy(chunkFolder, "expected imported chunk folder")

        local buildingsFolder = chunkFolder:FindFirstChild("Buildings")
        Assert.truthy(buildingsFolder, "expected Buildings folder")

        local buildingModel = buildingsFolder:FindFirstChild("room_overlap_building")
        Assert.truthy(buildingModel, "expected imported building model")

        local buildingTopY = buildingModel:GetAttribute("ArnisImportBuildingTopY")
        Assert.truthy(type(buildingTopY) == "number", "expected building top attribute on imported model")

        local roomsFolder = buildingModel:FindFirstChild("Rooms")
        Assert.truthy(roomsFolder, "expected Rooms folder under imported building")

        local ceilingsFolder = roomsFolder:FindFirstChild("Ceilings")
        Assert.truthy(ceilingsFolder, "expected Ceilings folder under imported building")

        local highestCeilingTopY = nil
        for _, descendant in ipairs(ceilingsFolder:GetDescendants()) do
            if descendant:IsA("BasePart") then
                local partTopY = descendant.Position.Y + descendant.Size.Y * 0.5
                if highestCeilingTopY == nil or partTopY > highestCeilingTopY then
                    highestCeilingTopY = partTopY
                end
            end
        end

        Assert.truthy(highestCeilingTopY ~= nil, "expected at least one authored ceiling part")
        Assert.truthy(
            highestCeilingTopY <= buildingTopY + 1e-4,
            "expected top-floor ceiling top to stay at or below the imported building top"
        )

        local shellFolder = buildingModel:FindFirstChild("Shell")
        Assert.truthy(shellFolder, "expected Shell folder under imported building")

        local lowestRoofBottomY = nil
        for _, descendant in ipairs(shellFolder:GetDescendants()) do
            if descendant:IsA("BasePart") then
                local nameLower = string.lower(descendant.Name)
                if string.find(nameLower, "roof", 1, true) then
                    local partBottomY = descendant.Position.Y - descendant.Size.Y * 0.5
                    if lowestRoofBottomY == nil or partBottomY < lowestRoofBottomY then
                        lowestRoofBottomY = partBottomY
                    end
                end
            end
        end

        Assert.truthy(lowestRoofBottomY ~= nil, "expected at least one roof or roof-closure part")
        Assert.truthy(
            highestCeilingTopY <= lowestRoofBottomY + 1e-4,
            "expected top-floor ceiling top to stay below the lowest roof or roof-closure bottom"
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
