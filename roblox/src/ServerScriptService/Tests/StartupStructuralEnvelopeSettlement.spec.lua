return function()
    local Workspace = game:GetService("Workspace")

    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)
    local Assert = require(script.Parent.Assert)

    local worldRootName = "GeneratedWorld_StartupStructuralEnvelopeSettlement"
    local spawnPoint = Vector3.new(128, 0, 128)

    local function makeAnchoredPart(name, size, position, parent)
        local part = Instance.new("Part")
        part.Name = name
        part.Anchored = true
        part.CanCollide = true
        part.Size = size
        part.CFrame = CFrame.new(position)
        part.Parent = parent
        return part
    end

    local worldRoot = Instance.new("Folder")
    worldRoot.Name = worldRootName
    worldRoot.Parent = Workspace

    local chunkFolder = Instance.new("Folder")
    chunkFolder.Name = "0_0"
    chunkFolder.Parent = worldRoot
    chunkFolder:SetAttribute("ArnisChunkId", "0_0")

    local buildingsFolder = Instance.new("Folder")
    buildingsFolder.Name = "Buildings"
    buildingsFolder.Parent = chunkFolder
    buildingsFolder:SetAttribute("ArnisChunkId", "0_0")

    Workspace:SetAttribute("ArnisStreamingRingNearResidentChunkCount", 1)
    Workspace:SetAttribute("ArnisStreamingRingNearDesiredChunkCount", 1)
    Workspace:SetAttribute("ArnisStreamingQueuedWorkItemCount", 0)
    Workspace:SetAttribute("ArnisStreamingSchedulerState", "steady_state")

    local ok, err = xpcall(function()
        local emptySnapshot = StreamingService.GetStartupResidencySnapshot(spawnPoint, worldRootName)
        Assert.falsy(
            emptySnapshot.ready,
            "expected startup residency to stay gated until the nearby chunk is registered"
        )
        Assert.equal(emptySnapshot.nearbyBuildingModels, 0, "expected no nearby building evidence before shells load")
        Assert.equal(emptySnapshot.nearbyWallParts, 0, "expected no nearby wall evidence before shells load")
        Assert.equal(emptySnapshot.nearbyRoofParts, 0, "expected no nearby roof evidence before shells load")

        local nearbyBuilding = Instance.new("Model")
        nearbyBuilding.Name = "nearby_shell_building"
        nearbyBuilding:SetAttribute("ArnisImportBuildingHeight", 20)
        nearbyBuilding:SetAttribute("ArnisImportSourceId", "nearby_shell_building")
        nearbyBuilding:SetAttribute("ArnisImportRoofShape", "flat")
        nearbyBuilding:SetAttribute("ArnisChunkId", "0_0")
        nearbyBuilding.Parent = buildingsFolder

        local nearbyShell = Instance.new("Folder")
        nearbyShell.Name = "Shell"
        nearbyShell.Parent = nearbyBuilding

        makeAnchoredPart("west_wall", Vector3.new(1, 16, 18), Vector3.new(120, 8, 128), nearbyShell)
        makeAnchoredPart("flat_roof", Vector3.new(18, 1, 18), Vector3.new(128, 18.5, 128), nearbyShell)

        local roofClosureDeck =
            makeAnchoredPart("flat_roof_closure", Vector3.new(4, 0.5, 4), Vector3.new(128, 17.5, 128), nearbyShell)
        roofClosureDeck:SetAttribute("ArnisRoofClosureDeck", true)

        local unregisteredChunk = Instance.new("Folder")
        unregisteredChunk.Name = "late_chunk"
        unregisteredChunk.Parent = worldRoot

        local unregisteredBuildings = Instance.new("Folder")
        unregisteredBuildings.Name = "Buildings"
        unregisteredBuildings.Parent = unregisteredChunk

        local lateBuilding = Instance.new("Model")
        lateBuilding.Name = "late_shell_building"
        lateBuilding:SetAttribute("ArnisImportBuildingHeight", 20)
        lateBuilding:SetAttribute("ArnisImportSourceId", "late_shell_building")
        lateBuilding:SetAttribute("ArnisImportRoofShape", "flat")
        lateBuilding:SetAttribute("ArnisChunkId", "late_chunk")
        lateBuilding.Parent = unregisteredBuildings

        local lateShell = Instance.new("Folder")
        lateShell.Name = "Shell"
        lateShell.Parent = lateBuilding
        makeAnchoredPart("late_wall", Vector3.new(1, 16, 18), Vector3.new(132, 8, 128), lateShell)
        makeAnchoredPart("late_roof", Vector3.new(18, 1, 18), Vector3.new(128, 18.5, 128), lateShell)

        local distantBuilding = Instance.new("Model")
        distantBuilding.Name = "distant_shell_building"
        distantBuilding:SetAttribute("ArnisImportBuildingHeight", 20)
        distantBuilding:SetAttribute("ArnisImportSourceId", "distant_shell_building")
        distantBuilding:SetAttribute("ArnisImportRoofShape", "flat")
        distantBuilding:SetAttribute("ArnisChunkId", "0_0")
        distantBuilding.Parent = buildingsFolder

        local distantShell = Instance.new("Folder")
        distantShell.Name = "Shell"
        distantShell.Parent = distantBuilding
        makeAnchoredPart("far_wall", Vector3.new(1, 16, 18), Vector3.new(520, 8, 520), distantShell)
        makeAnchoredPart("far_roof", Vector3.new(18, 1, 18), Vector3.new(528, 18.5, 520), distantShell)

        nearbyBuilding:PivotTo(CFrame.new(128, 0, 128))
        lateBuilding:PivotTo(CFrame.new(128, 0, 128))
        distantBuilding:PivotTo(CFrame.new(520, 0, 520))

        local unresolvedSnapshot = StreamingService.GetStartupResidencySnapshot(spawnPoint, worldRootName)
        Assert.falsy(
            unresolvedSnapshot.ready,
            "expected unregistered nearby shell geometry to stay out of startup residency"
        )
        Assert.equal(
            unresolvedSnapshot.nearbyBuildingModels,
            0,
            "expected unregistered nearby chunk shell evidence to stay invisible"
        )
        Assert.equal(
            unresolvedSnapshot.nearbyWallParts,
            0,
            "expected unregistered nearby chunk walls to stay invisible"
        )
        Assert.equal(
            unresolvedSnapshot.nearbyRoofParts,
            0,
            "expected unregistered nearby chunk roofs to stay invisible"
        )

        ChunkLoader.RegisterChunk("0_0", chunkFolder, {
            id = "0_0",
            originStuds = { x = 0, y = 0, z = 0 },
        })

        chunkFolder:SetAttribute("ArnisChunkId", nil)
        local unresolvedOwnedSnapshot = StreamingService.GetStartupResidencySnapshot(spawnPoint, worldRootName)
        Assert.falsy(
            unresolvedOwnedSnapshot.ready,
            "expected startup residency to ignore nearby shell evidence when the registered chunk folder loses ownership"
        )
        Assert.equal(
            unresolvedOwnedSnapshot.nearbyBuildingModels,
            0,
            "expected nearby buildings to stay hidden when the registered chunk folder loses ownership"
        )

        chunkFolder:SetAttribute("ArnisChunkId", "0_0")
        local snapshot = StreamingService.GetStartupResidencySnapshot(spawnPoint, worldRootName)

        Assert.truthy(snapshot.ready, "expected startup residency to wait for the nearby structural envelope")
        Assert.equal(snapshot.nearbyBuildingModels, 1, "expected one nearby building model")
        Assert.truthy(snapshot.nearbyWallParts >= 1, "expected nearby shell wall evidence")
        Assert.truthy(snapshot.collidableWallPartsNearby >= 1, "expected nearby collidable shell wall evidence")
        Assert.equal(snapshot.nearbyRoofParts, 1, "expected closure decks to stay out of roof evidence")
        Assert.truthy(snapshot.overheadRoofParts >= 1, "expected overhead roof evidence near the spawn point")
    end, debug.traceback)

    worldRoot:Destroy()
    ChunkLoader.Clear()
    Workspace:SetAttribute("ArnisStreamingRingNearResidentChunkCount", nil)
    Workspace:SetAttribute("ArnisStreamingRingNearDesiredChunkCount", nil)
    Workspace:SetAttribute("ArnisStreamingQueuedWorkItemCount", nil)
    Workspace:SetAttribute("ArnisStreamingSchedulerState", nil)

    if not ok then
        error(err, 0)
    end
end
