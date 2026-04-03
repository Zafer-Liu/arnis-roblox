return function()
    local Workspace = game:GetService("Workspace")

    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)
    local Assert = require(script.Parent.Assert)

    local worldRootName = "GeneratedWorld_StartupMergedShellReadableEnvelope"
    local spawnPoint = Vector3.new(128, 0, 128)

    local function makeAnchoredPart(className, name, size, position, parent)
        local part = Instance.new(className)
        part.Name = name
        part.Anchored = true
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
        local building = Instance.new("Model")
        building.Name = "merged_shell_readable_envelope"
        building:SetAttribute("ArnisImportBuildingHeight", 24)
        building:SetAttribute("ArnisImportSourceId", "merged_shell_readable_envelope")
        building:SetAttribute("ArnisImportRoofShape", "flat")
        building:SetAttribute("ArnisChunkId", "0_0")
        building.Parent = buildingsFolder

        local shellFolder = Instance.new("Folder")
        shellFolder.Name = "Shell"
        shellFolder.Parent = building

        local detailFolder = Instance.new("Folder")
        detailFolder.Name = "Detail"
        detailFolder.Parent = building

        local mergedWall = makeAnchoredPart(
            "MeshPart",
            "merged_shell_wall",
            Vector3.new(18, 18, 1),
            Vector3.new(128, 9, 120),
            shellFolder
        )
        mergedWall.CanCollide = false

        makeAnchoredPart("Part", "flat_roof", Vector3.new(18, 1, 18), Vector3.new(128, 24.5, 128), shellFolder)
        makeAnchoredPart(
            "Part",
            "MergedShellWallPresenceCue",
            Vector3.new(16, 3, 0.5),
            Vector3.new(128, 3, 119.4),
            detailFolder
        )
        makeAnchoredPart(
            "Part",
            "MergedShellStreetFacadeCue",
            Vector3.new(12, 2.5, 0.5),
            Vector3.new(128, 2.5, 119.2),
            detailFolder
        )

        building:PivotTo(CFrame.new(128, 0, 128))
        ChunkLoader.RegisterChunk("0_0", chunkFolder, {
            id = "0_0",
            originStuds = { x = 0, y = 0, z = 0 },
        })

        local snapshot = StreamingService.GetStartupResidencySnapshot(spawnPoint, worldRootName)
        Assert.truthy(snapshot.ready, "expected merged shell readable cues to satisfy startup envelope readiness")
        Assert.truthy(snapshot.nearbyMergedBuildingMeshParts >= 1, "expected merged shell mesh evidence near spawn")
        Assert.truthy(snapshot.nearbyReadableFacadeCueParts >= 2, "expected readable facade cues near spawn")
        Assert.equal(
            snapshot.collidableWallPartsNearby,
            0,
            "expected merged readable envelope path not to require collidable explicit walls"
        )
        Assert.truthy(
            snapshot.coherentEnvelopeNearbyReadableFacadeCueParts >= 2,
            "expected coherent envelope to keep readable facade cue evidence"
        )
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
