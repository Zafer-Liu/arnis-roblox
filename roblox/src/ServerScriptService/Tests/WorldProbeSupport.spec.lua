return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Workspace = game:GetService("Workspace")
    local Assert = require(script.Parent.Assert)
    local WorldProbeSupport = require(ReplicatedStorage.Shared.WorldProbeSupport)

    local worldRoot = Instance.new("Folder")
    worldRoot.Name = "GeneratedWorld_WorldProbeSupport"
    worldRoot.Parent = Workspace

    local chunkFolder = Instance.new("Folder")
    chunkFolder.Name = "0_0"
    chunkFolder.Parent = worldRoot

    local roadsFolder = Instance.new("Folder")
    roadsFolder.Name = "Roads"
    roadsFolder.Parent = chunkFolder

    local roadSurface = Instance.new("Part")
    roadSurface.Name = "RuntimeRoadSurface"
    roadSurface.Anchored = true
    roadSurface.Size = Vector3.new(16, 1, 16)
    roadSurface.Material = Enum.Material.Concrete
    roadSurface:SetAttribute("ArnisRoadSurfaceRole", "road")
    roadSurface.Parent = roadsFolder

    local spawn = Instance.new("SpawnLocation")
    spawn.Name = "CongressAveSpawn"
    spawn.Anchored = true
    spawn.Size = Vector3.new(6, 1, 6)
    spawn.Material = Enum.Material.Concrete
    spawn.Transparency = 1
    spawn.Parent = Workspace

    local detailFolder = Instance.new("Folder")
    detailFolder.Name = "Detail"
    detailFolder.Parent = roadsFolder

    local laneDecal = Instance.new("Part")
    laneDecal.Name = "LaneStripe"
    laneDecal.Anchored = true
    laneDecal.CanCollide = true
    laneDecal.Size = Vector3.new(10, 0.2, 10)
    laneDecal.Position = Vector3.new(0, 4, 0)
    laneDecal.Parent = detailFolder

    roadSurface.Position = Vector3.new(0, 2, 0)

    local terrain = Workspace.Terrain
    terrain:Clear()
    terrain:FillBlock(CFrame.new(0, 0, 0), Vector3.new(24, 4, 24), Enum.Material.Grass)

    local character = Instance.new("Model")
    character.Name = "WorldProbeSupportCharacter"
    character.Parent = Workspace

    local rootPart = Instance.new("Part")
    rootPart.Name = "HumanoidRootPart"
    rootPart.Anchored = true
    rootPart.CanCollide = false
    rootPart.Size = Vector3.new(2, 2, 1)
    rootPart.Position = Vector3.new(0, 12, 0)
    rootPart.Parent = character

    local function raycastGroundSupport(ignore)
        ignore = ignore or {}
        table.insert(ignore, 1, character)

        local raycastParams = RaycastParams.new()
        raycastParams.FilterType = Enum.RaycastFilterType.Exclude
        raycastParams.FilterDescendantsInstances = ignore

        for _ = 1, 8 do
            local origin = rootPart.Position + Vector3.new(0, 24, 0)
            local direction = Vector3.new(0, -(24 + 256), 0)
            local rayResult = Workspace:Raycast(origin, direction, raycastParams)
            if not rayResult then
                return nil
            end
            if WorldProbeSupport.shouldIgnoreGroundHit(rayResult.Instance, worldRoot, ignore) then
                ignore[#ignore + 1] = rayResult.Instance
                raycastParams.FilterDescendantsInstances = ignore
            else
                return rayResult
            end
        end

        return nil
    end

    local ok, err = xpcall(function()
        Assert.equal(
            WorldProbeSupport.classifySupportSurfaceRole(roadSurface),
            "road",
            "expected support role classification to trust explicit road surface attributes"
        )
        Assert.equal(
            WorldProbeSupport.shouldIgnoreGroundHit(spawn, worldRoot, {}),
            true,
            "expected hidden runtime spawn to be excluded from ground support sampling"
        )
        Assert.equal(
            WorldProbeSupport.shouldIgnoreGroundHit(laneDecal, worldRoot, {}),
            true,
            "expected decorative road detail to be excluded from ground support sampling"
        )
        Assert.equal(
            WorldProbeSupport.shouldIgnoreGroundHit(roadSurface, worldRoot, {}),
            false,
            "expected world-root road surfaces to remain valid ground support hits"
        )

        local roadResult = raycastGroundSupport()
        Assert.truthy(roadResult ~= nil, "expected retrying ground support raycast to find a world-root support surface")
        Assert.equal(
            roadResult.Instance,
            roadSurface,
            "expected ground support raycast to skip spawn/detail hits and land on the explicit road surface"
        )
        Assert.equal(
            WorldProbeSupport.classifySupportSurfaceRole(roadResult.Instance),
            "road",
            "expected accepted ground support hit to classify as road"
        )

        roadSurface:Destroy()
        laneDecal:Destroy()

        local terrainResult = raycastGroundSupport()
        Assert.truthy(terrainResult ~= nil, "expected terrain fallback when no world-root support surfaces remain")
        Assert.equal(
            terrainResult.Instance,
            terrain,
            "expected retrying ground support raycast to fall through to terrain after skipping spawn/detail"
        )
        Assert.equal(
            WorldProbeSupport.classifySupportSurfaceRole(terrainResult.Instance),
            "terrain",
            "expected terrain fallback hit to classify as terrain"
        )
    end, debug.traceback)

    terrain:Clear()
    character:Destroy()
    spawn:Destroy()
    worldRoot:Destroy()

    if not ok then
        error(err, 0)
    end
end
