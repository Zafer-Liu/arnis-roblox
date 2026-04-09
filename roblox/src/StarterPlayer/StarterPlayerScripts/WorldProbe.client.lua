local HttpService = game:GetService("HttpService")
local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local RunService = game:GetService("RunService")
local Workspace = game:GetService("Workspace")
local WorldProbeGeometry = require(ReplicatedStorage.Shared.WorldProbeGeometry)
local WorldProbeSupport = require(ReplicatedStorage.Shared.WorldProbeSupport)
local WorldProbeTelemetryFlags = require(ReplicatedStorage.Shared.WorldProbeTelemetryFlags)
local WorldProbeTerrain = require(ReplicatedStorage.Shared.WorldProbeTerrain)

local player = Players.LocalPlayer

local WORLD_ROOT_ATTR = "ArnisWorldRootName"
local CHUNK_BUILDING_BOUNDS_MIN_X_ATTR = "ArnisMinimapChunkBuildingBoundsMinX"
local CHUNK_BUILDING_BOUNDS_MAX_X_ATTR = "ArnisMinimapChunkBuildingBoundsMaxX"
local CHUNK_BUILDING_BOUNDS_MIN_Z_ATTR = "ArnisMinimapChunkBuildingBoundsMinZ"
local CHUNK_BUILDING_BOUNDS_MAX_Z_ATTR = "ArnisMinimapChunkBuildingBoundsMaxZ"
local BUILDING_BOUNDS_MIN_X_ATTR = "ArnisImportBoundsMinX"
local BUILDING_BOUNDS_MAX_X_ATTR = "ArnisImportBoundsMaxX"
local BUILDING_BOUNDS_MIN_Z_ATTR = "ArnisImportBoundsMinZ"
local BUILDING_BOUNDS_MAX_Z_ATTR = "ArnisImportBoundsMaxZ"
local IDLE_SAMPLE_INTERVAL = 1.5
local MOVING_SAMPLE_INTERVAL = 0.5
local NEARBY_BUILDING_RADIUS = 260
local NEARBY_BUILDING_RADIUS_SQ = NEARBY_BUILDING_RADIUS * NEARBY_BUILDING_RADIUS
local NEARBY_NAMED_BUILDING_RADIUS = 640
local NEARBY_NAMED_BUILDING_RADIUS_SQ = NEARBY_NAMED_BUILDING_RADIUS * NEARBY_NAMED_BUILDING_RADIUS
local OVERHEAD_ROOF_RADIUS = 220
local OVERHEAD_ROOF_RADIUS_SQ = OVERHEAD_ROOF_RADIUS * OVERHEAD_ROOF_RADIUS
local OVERHEAD_MIN_DELTA_Y = 12
local NEARBY_WALL_RADIUS = 180
local NEARBY_WALL_RADIUS_SQ = NEARBY_WALL_RADIUS * NEARBY_WALL_RADIUS
local IDLE_RESAMPLE_DISTANCE = 24
local MOVING_RESAMPLE_DISTANCE = 8
local MOVING_SPEED_THRESHOLD = 4
local MAX_BUILDING_IDS = 6
local MAX_NAMED_BUILDINGS = 6
local MAX_OVERHEAD_IDS = 6
local GROUND_SAMPLE_HEIGHT = 24
local GROUND_SAMPLE_DEPTH = 256
local LOCAL_TERRAIN_SAMPLE_RADIUS = 12
local LOCAL_TERRAIN_SAMPLE_PATTERN = "cross_5"
local LOCAL_TERRAIN_NEIGHBOR_PAIRS = {
    { 3, 1 },
    { 3, 2 },
    { 3, 4 },
    { 3, 5 },
}
local LOCAL_TERRAIN_EDGE_INDICES = { 1, 2, 4, 5 }
local LOCAL_TERRAIN_OFFSETS = {
    Vector3.new(-LOCAL_TERRAIN_SAMPLE_RADIUS, 0, 0),
    Vector3.new(0, 0, -LOCAL_TERRAIN_SAMPLE_RADIUS),
    Vector3.new(0, 0, 0),
    Vector3.new(0, 0, LOCAL_TERRAIN_SAMPLE_RADIUS),
    Vector3.new(LOCAL_TERRAIN_SAMPLE_RADIUS, 0, 0),
}

local lastPayloadJson = nil
local lastBootstrapPayloadJson = nil
local lastCompactPayloadJson = nil
local lastLocalExperiencePayloadJson = nil
local lastPerfPayloadJson = nil
local lastSampleAt = 0
local lastSamplePosition = nil
local lastSampleWorldRootName = nil
local lastSampleWasMoving = false

-- Performance counters: zero per-frame allocations.
-- Pre-allocated ring buffer for frame times (5-second window at 60 fps = 300 slots).
local PERF_RING_CAPACITY = 300
local PERF_WINDOW_SECONDS = 5
local perfRing = table.create(PERF_RING_CAPACITY, 0)
local perfSortBuf = table.create(PERF_RING_CAPACITY, 0)
local perfRingHead = 0
local perfRingCount = 0
local perfRingTimestampStart = 0
local perfCachedPartCount = 0
local perfCachedMeshPartCount = 0
local perfLastEmitAt = 0
local PERF_EMIT_INTERVAL = 5
local perfWatchedWorldRoot = nil
local perfWatchedConnections = {}

-- ---------------------------------------------------------------------------
-- Flicker detector state
-- ---------------------------------------------------------------------------
-- Runs at FLICKER_SAMPLE_HZ on Heartbeat, emits every FLICKER_EMIT_INTERVAL s.
-- Budget goal: < ~500 us per heartbeat sample. We use a single OverlapParams
-- GetPartBoundsInBox for the near-part count (O(near-part-count)) and a
-- bounded incremental DescendantAdded/Removing watcher for chunk churn — no
-- full GetDescendants sweeps.
local FLICKER_SAMPLE_HZ = 10
local FLICKER_SAMPLE_INTERVAL = 1 / FLICKER_SAMPLE_HZ
local FLICKER_WINDOW_SECONDS = 6
local FLICKER_WINDOW_CAPACITY = FLICKER_WINDOW_SECONDS * FLICKER_SAMPLE_HZ
local FLICKER_EMIT_INTERVAL = 1
local FLICKER_NEAR_RADIUS_STUDS = 50
local FLICKER_NEAR_MAX_PARTS = 2048
local FLICKER_NEAR_BOX_SIZE = Vector3.new(
    FLICKER_NEAR_RADIUS_STUDS * 2,
    FLICKER_NEAR_RADIUS_STUDS * 2,
    FLICKER_NEAR_RADIUS_STUDS * 2
)
local FLICKER_RING_BOUNCE_DELTA_BYTES = 64 * 1024 -- 64 KiB minimum swing to count as a bounce

-- Chunk folder names look like "0_-2", "12_3" — two signed integers joined
-- by underscore. Avoid matching unrelated folders under the world root.
local function isChunkFolderName(name)
    if type(name) ~= "string" then
        return false
    end
    return string.match(name, "^%-?%d+_%-?%d+$") ~= nil
end

local nearPartRing = table.create(FLICKER_WINDOW_CAPACITY, 0)
local ringResidentNearRing = table.create(FLICKER_WINDOW_CAPACITY, 0)
local ringResidentMidRing = table.create(FLICKER_WINDOW_CAPACITY, 0)
local ringResidentFarRing = table.create(FLICKER_WINDOW_CAPACITY, 0)
local nearPartRingHead = 0
local nearPartRingCount = 0
local flickerLastSampleAt = 0
local flickerLastEmitAt = 0
local flickerLastFetchFailures = nil

-- Stationary-vs-moving classification. We stamp the HRP position on each
-- flicker sample tick and accumulate per-window displacement so the server
-- aggregate can split thrash into stationary vs moving buckets. "Stationary"
-- is defined as <4 studs of cumulative movement across the emit window — a
-- threshold tuned to the humanoid's natural anchoring jitter while idle.
local FLICKER_STATIONARY_STUDS = 4
local flickerLastSamplePosition = nil
local flickerWindowMovementStuds = 0
local flickerWindowMovementPeak = 0

-- Chunk id -> timestamp maps for thrash detection. We keep the last
-- add/remove wall-clock time for each chunk id seen, and on every emit we
-- count ids that had BOTH an add and a remove within the active window.
local flickerChunkAddTimes = {}
local flickerChunkRemoveTimes = {}
local flickerChunkThrashCounts = {} -- chunk id -> number of thrash cycles in window
local flickerWatchedWorldRoot = nil
local flickerWatchedConnections = {}
local flickerOverlapParams = OverlapParams.new()
flickerOverlapParams.MaxParts = FLICKER_NEAR_MAX_PARTS
flickerOverlapParams.FilterType = Enum.RaycastFilterType.Exclude
flickerOverlapParams.FilterDescendantsInstances = {}
flickerOverlapParams.RespectCanCollide = false

-- Mirror of the last published flicker sample, for server-side ingestion.
local flickerClientRemote = nil
local function resolveFlickerClientRemote()
    if flickerClientRemote and flickerClientRemote.Parent ~= nil then
        return flickerClientRemote
    end
    local existing = ReplicatedStorage:FindFirstChild("ArnisClientFlickerRemote")
    if existing and (existing:IsA("RemoteEvent") or existing:IsA("UnreliableRemoteEvent")) then
        flickerClientRemote = existing
    end
    return flickerClientRemote
end

local function resolveTelemetryFamilies()
    local playerTelemetryFamilies = player:GetAttribute(WorldProbeTelemetryFlags.PLAYER_ATTR)
    if type(playerTelemetryFamilies) == "string" and playerTelemetryFamilies ~= "" then
        return playerTelemetryFamilies
    end
    local replicatedTelemetryFamilies = ReplicatedStorage:GetAttribute(WorldProbeTelemetryFlags.REPLICATED_STORAGE_ATTR)
    if type(replicatedTelemetryFamilies) == "string" and replicatedTelemetryFamilies ~= "" then
        return replicatedTelemetryFamilies
    end
    return Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)
end

local telemetryFamilies = resolveTelemetryFamilies()
local telemetryFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(telemetryFamilies)

local function refreshTelemetryFlags()
    local nextTelemetryFamilies = resolveTelemetryFamilies()
    if nextTelemetryFamilies == telemetryFamilies then
        return
    end

    telemetryFamilies = nextTelemetryFamilies
    telemetryFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(telemetryFamilies)
    lastPayloadJson = nil
    lastBootstrapPayloadJson = nil
    lastCompactPayloadJson = nil
    lastLocalExperiencePayloadJson = nil
end

local function setPlayerAttributeIfChanged(name, nextValue)
    if player:GetAttribute(name) == nextValue then
        return
    end
    player:SetAttribute(name, nextValue)
end

local function getCharacterRootPart()
    local character = player.Character
    if not character then
        return nil
    end
    return character:FindFirstChild("HumanoidRootPart")
end

local function getWorldRoot()
    local worldRootName = Workspace:GetAttribute(WORLD_ROOT_ATTR)
    if type(worldRootName) ~= "string" or worldRootName == "" then
        return nil, nil
    end
    local worldRoot = Workspace:FindFirstChild(worldRootName)
    return worldRoot, worldRootName
end

local function resolveMovementAwareSampleCadence(rootPart)
    if rootPart and rootPart:IsA("BasePart") then
        local speed = rootPart.AssemblyLinearVelocity.Magnitude
        if speed >= MOVING_SPEED_THRESHOLD then
            return MOVING_SAMPLE_INTERVAL, MOVING_RESAMPLE_DISTANCE, true
        end
    end

    return IDLE_SAMPLE_INTERVAL, IDLE_RESAMPLE_DISTANCE, false
end

local function appendLimited(list, value, limit)
    if #list >= limit then
        return
    end
    list[#list + 1] = value
end

local function appendNearestNamedBuilding(list, entry, limit)
    if type(entry) ~= "table" then
        return
    end
    local sourceId = entry.sourceId
    if type(sourceId) ~= "string" or sourceId == "" then
        return
    end
    for _, existing in ipairs(list) do
        if existing.sourceId == sourceId then
            return
        end
    end
    list[#list + 1] = entry
    table.sort(list, function(a, b)
        return (a.distanceStuds or math.huge) < (b.distanceStuds or math.huge)
    end)
    while #list > limit do
        table.remove(list)
    end
end

local function roundTenths(value)
    if type(value) ~= "number" then
        return nil
    end
    return math.round(value * 10) / 10
end

local function chunkIntersectsNearbyBuildingRadius(chunkFolder, rootPosition)
    if chunkFolder == nil or rootPosition == nil then
        return true
    end

    local minX = tonumber(chunkFolder:GetAttribute(CHUNK_BUILDING_BOUNDS_MIN_X_ATTR))
    local maxX = tonumber(chunkFolder:GetAttribute(CHUNK_BUILDING_BOUNDS_MAX_X_ATTR))
    local minZ = tonumber(chunkFolder:GetAttribute(CHUNK_BUILDING_BOUNDS_MIN_Z_ATTR))
    local maxZ = tonumber(chunkFolder:GetAttribute(CHUNK_BUILDING_BOUNDS_MAX_Z_ATTR))
    if minX == nil or maxX == nil or minZ == nil or maxZ == nil then
        return true
    end

    local closestX = math.clamp(rootPosition.X, minX, maxX)
    local closestZ = math.clamp(rootPosition.Z, minZ, maxZ)
    local deltaX = rootPosition.X - closestX
    local deltaZ = rootPosition.Z - closestZ
    local distanceSq = deltaX * deltaX + deltaZ * deltaZ
    if distanceSq <= NEARBY_NAMED_BUILDING_RADIUS_SQ then
        return true
    end
    return false
end

local function modelIntersectsNearbyNamedBuildingRadius(model, rootPosition)
    if model == nil or rootPosition == nil then
        return true
    end

    local minX = tonumber(model:GetAttribute(BUILDING_BOUNDS_MIN_X_ATTR))
    local maxX = tonumber(model:GetAttribute(BUILDING_BOUNDS_MAX_X_ATTR))
    local minZ = tonumber(model:GetAttribute(BUILDING_BOUNDS_MIN_Z_ATTR))
    local maxZ = tonumber(model:GetAttribute(BUILDING_BOUNDS_MAX_Z_ATTR))
    if minX == nil or maxX == nil or minZ == nil or maxZ == nil then
        return true
    end

    local closestX = math.clamp(rootPosition.X, minX, maxX)
    local closestZ = math.clamp(rootPosition.Z, minZ, maxZ)
    local deltaX = rootPosition.X - closestX
    local deltaZ = rootPosition.Z - closestZ
    local distanceSq = deltaX * deltaX + deltaZ * deltaZ
    if distanceSq <= NEARBY_NAMED_BUILDING_RADIUS_SQ then
        return true
    end
    return false
end

local function isRoofClosureDeckPart(part)
    if part == nil then
        return false
    end
    if part:GetAttribute("ArnisRoofClosureDeck") == true then
        return true
    end

    return string.find(part.Name, "_roof_closure", 1, true) ~= nil
end

local function isRoofCuePart(part)
    if part == nil or not part:IsA("BasePart") then
        return false
    end

    local name = part.Name
    return name == "MergedShellRooflineCue" or name == "MergedShellPerimeterCue"
end

local function isReadableFacadeCuePart(part)
    if part == nil or not part:IsA("BasePart") then
        return false
    end

    local name = part.Name
    return name == "MergedShellWallPresenceCue"
        or name == "MergedShellStreetFacadeCue"
        or name == "MergedShellWindowPaneCue"
        or name == "MergedShellDoorCue"
        or name == "FacadeBeltline"
        or name == "CornerAccent"
end

local function findNearestSourceId(hitInstance)
    local node = hitInstance
    while node and node.Parent do
        if node:IsA("Model") then
            local sourceId = node:GetAttribute("ArnisImportSourceId")
            if type(sourceId) == "string" and sourceId ~= "" then
                return sourceId
            end
        end
        node = node.Parent
    end
    return nil
end

local function raycastGroundSupport(rootPart, worldRoot, ignore)
    local character = player.Character
    ignore = ignore or {}
    if character then
        table.insert(ignore, 1, character)
    end
    local raycastParams = RaycastParams.new()
    raycastParams.FilterType = Enum.RaycastFilterType.Exclude
    raycastParams.FilterDescendantsInstances = ignore

    for _ = 1, 8 do
        local origin = rootPart.Position + Vector3.new(0, GROUND_SAMPLE_HEIGHT, 0)
        local direction = Vector3.new(0, -(GROUND_SAMPLE_HEIGHT + GROUND_SAMPLE_DEPTH), 0)
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

local function sampleGroundSupport(rootPart, worldRoot)
    local rayResult = raycastGroundSupport(rootPart, worldRoot)
    if not rayResult then
        return {
            groundMaterial = nil,
            groundInstance = nil,
            supportSurfaceRole = "unknown",
            supportY = nil,
            terrainY = nil,
            supportMinusTerrainYStuds = nil,
            supportSourceIds = {},
        }
    end

    local supportInstance = rayResult.Instance
    local supportSourceId = findNearestSourceId(supportInstance)
    local supportY = roundTenths(rayResult.Position.Y)
    local supportSurfaceRole = WorldProbeSupport.classifySupportSurfaceRole(supportInstance)
    local terrainY = nil
    if supportSurfaceRole ~= "terrain" and supportSurfaceRole ~= "water" then
        local beneathResult = raycastGroundSupport(rootPart, worldRoot, { supportInstance })
        if beneathResult and WorldProbeSupport.classifySupportSurfaceRole(beneathResult.Instance) == "terrain" then
            terrainY = roundTenths(beneathResult.Position.Y)
        end
    else
        terrainY = supportY
    end

    return {
        groundMaterial = tostring(rayResult.Material),
        groundInstance = supportInstance and supportInstance:GetFullName() or nil,
        supportSurfaceRole = supportSurfaceRole,
        supportY = supportY,
        terrainY = terrainY,
        supportMinusTerrainYStuds = if supportY ~= nil and terrainY ~= nil
            then roundTenths(supportY - terrainY)
            else nil,
        supportSourceIds = if supportSourceId ~= nil then { supportSourceId } else {},
    }
end

local function raycastTerrainAtPosition(samplePosition, worldRoot)
    local character = player.Character
    local ignore = {}
    if character then
        table.insert(ignore, 1, character)
    end

    local raycastParams = RaycastParams.new()
    raycastParams.FilterType = Enum.RaycastFilterType.Exclude
    raycastParams.FilterDescendantsInstances = ignore

    for _ = 1, 8 do
        local origin = samplePosition + Vector3.new(0, GROUND_SAMPLE_HEIGHT, 0)
        local direction = Vector3.new(0, -(GROUND_SAMPLE_HEIGHT + GROUND_SAMPLE_DEPTH), 0)
        local rayResult = Workspace:Raycast(origin, direction, raycastParams)
        if not rayResult then
            return nil
        end
        if WorldProbeSupport.classifySupportSurfaceRole(rayResult.Instance) == "terrain" then
            return rayResult
        end

        ignore[#ignore + 1] = rayResult.Instance
        raycastParams.FilterDescendantsInstances = ignore
    end

    return nil
end

local function sampleLocalTerrain(rootPart, worldRoot)
    local rootPosition = rootPart.Position
    local samples = table.create(#LOCAL_TERRAIN_OFFSETS)

    for index, offset in ipairs(LOCAL_TERRAIN_OFFSETS) do
        local terrainResult = raycastTerrainAtPosition(rootPosition + offset, worldRoot)
        samples[index] = {
            terrainY = if terrainResult then roundTenths(terrainResult.Position.Y) else nil,
            terrainMaterial = if terrainResult then terrainResult.Material.Name else nil,
        }
    end

    return WorldProbeTerrain.summarizeTerrainSamples(samples, {
        centerIndex = 3,
        samplePattern = LOCAL_TERRAIN_SAMPLE_PATTERN,
        sampleRadiusStuds = LOCAL_TERRAIN_SAMPLE_RADIUS,
        neighborPairs = LOCAL_TERRAIN_NEIGHBOR_PAIRS,
        edgeIndices = LOCAL_TERRAIN_EDGE_INDICES,
    })
end

local function summarizeWorld(rootPart, worldRoot, worldRootName, telemetryFlags)
    local rootPosition = rootPart.Position
    local nearbyBuildingModels = 0
    local nearbyMergedBuildingMeshParts = 0
    local nearbyRoofParts = 0
    local overheadRoofParts = 0
    local overheadRoofMinClearanceStuds = nil
    local nearbyWallParts = 0
    local nearbyReadableFacadeCueParts = 0
    local collidableWallPartsNearby = 0
    local nearestWallDistanceStuds = nil
    local nearestBuildingSourceIds = {}
    local overheadRoofSourceIds = {}
    local nearestBuildingDetails = nil
    local nearestNamedBuildingDetails = {}
    local groundSupport = sampleGroundSupport(rootPart, worldRoot)
    local localTerrain = nil
    local playerLocalTelemetryEnabled = WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "player_local")

    if WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "terrain") then
        localTerrain = sampleLocalTerrain(rootPart, worldRoot)
    end

    if WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "structures") then
        nearestBuildingDetails = {}
    end

    for _, chunkFolder in ipairs(worldRoot:GetChildren()) do
        local buildingsFolder = chunkFolder:FindFirstChild("Buildings")
        if not buildingsFolder then
            continue
        end
        if not chunkIntersectsNearbyBuildingRadius(chunkFolder, rootPosition) then
            continue
        end

        local mergedMeshes = buildingsFolder:FindFirstChild("MergedMeshes")
        if mergedMeshes then
            for _, descendant in ipairs(mergedMeshes:GetDescendants()) do
                if not descendant:IsA("MeshPart") then
                    continue
                end

                local partOffset = descendant.Position - rootPosition
                local horizontalDistanceSq = partOffset.X * partOffset.X + partOffset.Z * partOffset.Z
                if horizontalDistanceSq <= NEARBY_BUILDING_RADIUS_SQ then
                    nearbyMergedBuildingMeshParts += 1
                end
            end
        end

        for _, model in ipairs(buildingsFolder:GetChildren()) do
            if not model:IsA("Model") or model:GetAttribute("ArnisImportBuildingHeight") == nil then
                continue
            end
            if not modelIntersectsNearbyNamedBuildingRadius(model, rootPosition) then
                continue
            end

            local sourceId = model:GetAttribute("ArnisImportSourceId")
            if type(sourceId) ~= "string" or sourceId == "" then
                continue
            end
            local roofShape = model:GetAttribute("ArnisImportRoofShape")
            local buildingTopY = model:GetAttribute("ArnisImportBuildingTopY")
            local buildingUsage = model:GetAttribute("ArnisImportBuildingUsage")
            local shellFolder = model:FindFirstChild("Shell")

            local pivotPosition = model:GetPivot().Position
            local offset = pivotPosition - rootPosition
            local horizontalDistanceSq = offset.X * offset.X + offset.Z * offset.Z
            if horizontalDistanceSq > NEARBY_NAMED_BUILDING_RADIUS_SQ then
                continue
            end
            local horizontalDistance = math.sqrt(horizontalDistanceSq)

            local buildingName = model:GetAttribute("ArnisImportBuildingName")
            if
                type(buildingName) == "string"
                and buildingName ~= ""
                and horizontalDistance <= NEARBY_NAMED_BUILDING_RADIUS
            then
                appendNearestNamedBuilding(nearestNamedBuildingDetails, {
                    sourceId = sourceId,
                    buildingName = buildingName,
                    usage = buildingUsage,
                    roofShape = roofShape,
                    roofMaterial = model:GetAttribute("ArnisImportRoofMaterial"),
                    wallMaterial = model:GetAttribute("ArnisImportWallMaterial"),
                    distanceStuds = roundTenths(horizontalDistance),
                }, MAX_NAMED_BUILDINGS)
            end
            local isWithinNearbyBuildingRadius = horizontalDistance <= NEARBY_BUILDING_RADIUS
            if not isWithinNearbyBuildingRadius then
                continue
            end

            nearbyBuildingModels += 1
            appendLimited(nearestBuildingSourceIds, sourceId, MAX_BUILDING_IDS)
            if nearestBuildingDetails ~= nil then
                appendLimited(nearestBuildingDetails, {
                    sourceId = sourceId,
                    buildingName = buildingName,
                    roofShape = roofShape,
                    buildingTopY = buildingTopY,
                    usage = buildingUsage,
                }, MAX_BUILDING_IDS)
            end

            for _, descendant in ipairs(model:GetDescendants()) do
                if not descendant:IsA("BasePart") then
                    continue
                end

                local partOffset = descendant.Position - rootPosition
                local nameLower = string.lower(descendant.Name)
                local horizontalDistanceSq = partOffset.X * partOffset.X + partOffset.Z * partOffset.Z
                local isRoofClosureDeck = descendant:GetAttribute("ArnisRoofClosureDeck") == true
                    or isRoofClosureDeckPart(descendant)
                local isRoofPart = string.find(nameLower, "roof", 1, true) ~= nil and not isRoofClosureDeck
                local isRoofCue = isRoofCuePart(descendant)
                local isReadableFacadeCue = isReadableFacadeCuePart(descendant)

                if descendant:IsA("MeshPart") and shellFolder and descendant:IsDescendantOf(shellFolder) then
                    if
                        horizontalDistanceSq <= NEARBY_BUILDING_RADIUS_SQ
                        and not isRoofPart
                        and not isRoofCue
                        and not isRoofClosureDeck
                    then
                        nearbyMergedBuildingMeshParts += 1
                    end
                end

                if
                    shellFolder
                    and descendant:IsDescendantOf(shellFolder)
                    and not isRoofPart
                    and not isRoofCue
                    and not isRoofClosureDeck
                then
                    local isNearbyShellWall, nearestShellWallDistanceStuds =
                        WorldProbeGeometry.isNearbyShellWall(descendant, rootPosition, NEARBY_WALL_RADIUS)
                    if isNearbyShellWall then
                        nearbyWallParts += 1
                        if descendant.CanCollide then
                            collidableWallPartsNearby += 1
                        end
                        if
                            nearestWallDistanceStuds == nil
                            or nearestShellWallDistanceStuds < nearestWallDistanceStuds
                        then
                            nearestWallDistanceStuds = nearestShellWallDistanceStuds
                        end
                    end
                    continue
                end
                if isReadableFacadeCue and horizontalDistanceSq <= NEARBY_WALL_RADIUS_SQ then
                    nearbyReadableFacadeCueParts += 1
                end
                if not isRoofPart and not isRoofCue then
                    continue
                end

                nearbyRoofParts += 1

                local verticalDelta = partOffset.Y
                if horizontalDistanceSq <= OVERHEAD_ROOF_RADIUS_SQ and verticalDelta >= OVERHEAD_MIN_DELTA_Y then
                    overheadRoofParts += 1
                    appendLimited(overheadRoofSourceIds, sourceId, MAX_OVERHEAD_IDS)
                    if overheadRoofMinClearanceStuds == nil or verticalDelta < overheadRoofMinClearanceStuds then
                        overheadRoofMinClearanceStuds = verticalDelta
                    end
                end
            end
        end
    end

    local localSupport = nil
    local localEnclosure = nil
    local localRoofCover = nil
    local characterPosition = nil

    if playerLocalTelemetryEnabled then
        localSupport = {
            surfaceRole = groundSupport.supportSurfaceRole,
            supportY = groundSupport.supportY,
            terrainY = groundSupport.terrainY,
            supportMinusTerrainYStuds = groundSupport.supportMinusTerrainYStuds,
            sourceIds = groundSupport.supportSourceIds,
        }
        localEnclosure = {
            nearbyWallParts = nearbyWallParts,
            readableFacadeCueParts = nearbyReadableFacadeCueParts,
            collidableWallPartsNearby = collidableWallPartsNearby,
            nearestWallDistanceStuds = roundTenths(nearestWallDistanceStuds),
        }
        localRoofCover = {
            nearbyRoofParts = nearbyRoofParts,
            overheadRoofParts = overheadRoofParts,
            overheadRoofMinClearanceStuds = roundTenths(overheadRoofMinClearanceStuds),
            overheadRoofSourceIds = overheadRoofSourceIds,
        }
        characterPosition = {
            x = roundTenths(rootPosition.X),
            y = roundTenths(rootPosition.Y),
            z = roundTenths(rootPosition.Z),
        }
    end

    return {
        worldRootName = worldRootName,
        nearbyBuildingModels = nearbyBuildingModels,
        nearbyMergedBuildingMeshParts = nearbyMergedBuildingMeshParts,
        nearbyRoofParts = nearbyRoofParts,
        overheadRoofParts = overheadRoofParts,
        overheadRoofMinClearanceStuds = roundTenths(overheadRoofMinClearanceStuds),
        nearbyWallParts = nearbyWallParts,
        nearbyReadableFacadeCueParts = nearbyReadableFacadeCueParts,
        collidableWallPartsNearby = collidableWallPartsNearby,
        nearestWallDistanceStuds = roundTenths(nearestWallDistanceStuds),
        nearestBuildingSourceIds = nearestBuildingSourceIds,
        nearestBuildingDetails = nearestBuildingDetails,
        nearestNamedBuildingDetails = nearestNamedBuildingDetails,
        overheadRoofSourceIds = overheadRoofSourceIds,
        groundMaterial = groundSupport.groundMaterial,
        groundInstance = groundSupport.groundInstance,
        supportSurfaceRole = groundSupport.supportSurfaceRole,
        supportY = groundSupport.supportY,
        terrainY = groundSupport.terrainY,
        supportMinusTerrainYStuds = groundSupport.supportMinusTerrainYStuds,
        supportSourceIds = groundSupport.supportSourceIds,
        localSupport = localSupport,
        localTerrain = localTerrain,
        localEnclosure = localEnclosure,
        localRoofCover = localRoofCover,
        characterPosition = characterPosition,
    }
end

local function recordFrameTime(dt)
    perfRingHead = perfRingHead % PERF_RING_CAPACITY + 1
    perfRing[perfRingHead] = dt
    if perfRingCount < PERF_RING_CAPACITY then
        perfRingCount = perfRingCount + 1
    end
end

local function disconnectPerfWatchers()
    for _, conn in ipairs(perfWatchedConnections) do
        if conn and conn.Connected then
            conn:Disconnect()
        end
    end
    perfWatchedConnections = {}
    perfWatchedWorldRoot = nil
end

local function applyPerfInstanceDelta(instance, delta)
    if instance == nil then
        return
    end
    if instance:IsA("MeshPart") then
        perfCachedMeshPartCount = perfCachedMeshPartCount + delta
        perfCachedPartCount = perfCachedPartCount + delta
    elseif instance:IsA("BasePart") then
        perfCachedPartCount = perfCachedPartCount + delta
    end
end

local function handlePerfDescendantAdded(descendant)
    applyPerfInstanceDelta(descendant, 1)
end

local function handlePerfDescendantRemoving(descendant)
    applyPerfInstanceDelta(descendant, -1)
end

local function ensurePerfInstanceWatchers(worldRoot)
    if worldRoot == perfWatchedWorldRoot then
        return
    end

    disconnectPerfWatchers()
    perfCachedPartCount = 0
    perfCachedMeshPartCount = 0
    if worldRoot == nil then
        return
    end

    perfWatchedWorldRoot = worldRoot
    for _, desc in ipairs(worldRoot:GetDescendants()) do
        applyPerfInstanceDelta(desc, 1)
    end
    table.insert(perfWatchedConnections, worldRoot.DescendantAdded:Connect(handlePerfDescendantAdded))
    table.insert(perfWatchedConnections, worldRoot.DescendantRemoving:Connect(handlePerfDescendantRemoving))
end

local function publishPerfTelemetry()
    if not WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "client_perf") then
        return
    end
    local now = os.clock()
    if now - perfLastEmitAt < PERF_EMIT_INTERVAL then
        return
    end
    perfLastEmitAt = now
    if perfRingCount == 0 then
        return
    end

    local sampleCount = perfRingCount
    local sumDt = 0
    local maxDt = 0

    -- Reuse pre-allocated sort buffer (no allocation per emit)
    local ringStart = perfRingHead - sampleCount + 1
    for i = 1, sampleCount do
        local idx = (ringStart + i - 2) % PERF_RING_CAPACITY + 1
        local v = perfRing[idx]
        perfSortBuf[i] = v
        sumDt = sumDt + v
        if v > maxDt then
            maxDt = v
        end
    end
    -- Push stale slots to end so they don't corrupt p99 calculation
    for i = sampleCount + 1, PERF_RING_CAPACITY do
        perfSortBuf[i] = math.huge
    end
    table.sort(perfSortBuf)

    local avgDt = sumDt / sampleCount
    local p99Index = math.ceil(sampleCount * 0.99)
    if p99Index < 1 then
        p99Index = 1
    end
    local p99Dt = perfSortBuf[p99Index]

    local worldRoot = getWorldRoot()
    ensurePerfInstanceWatchers(worldRoot)
    local totalParts = perfCachedPartCount
    local totalMeshParts = perfCachedMeshPartCount

    local perfPayload = {
        avgFrameTimeMs = math.round(avgDt * 100000) / 100,
        p99FrameTimeMs = math.round(p99Dt * 100000) / 100,
        maxFrameTimeMs = math.round(maxDt * 100000) / 100,
        fps = math.round(1 / avgDt * 10) / 10,
        sampleCount = sampleCount,
        windowSeconds = PERF_WINDOW_SECONDS,
        instanceCountParts = totalParts,
        instanceCountMeshParts = totalMeshParts,
    }
    WorldProbeTelemetryFlags.annotateMarkerPayload(perfPayload, telemetryFlags)
    local perfPayloadJson = HttpService:JSONEncode(perfPayload)
    if perfPayloadJson ~= lastPerfPayloadJson then
        lastPerfPayloadJson = perfPayloadJson
        print("ARNIS_CLIENT_PERF " .. perfPayloadJson)
    end
end

local function disconnectFlickerWatchers()
    for _, conn in ipairs(flickerWatchedConnections) do
        if conn and conn.Connected then
            conn:Disconnect()
        end
    end
    flickerWatchedConnections = {}
    flickerWatchedWorldRoot = nil
end

local function handleChunkFolderAdded(child)
    if child == nil or not isChunkFolderName(child.Name) then
        return
    end
    flickerChunkAddTimes[child.Name] = os.clock()
end

local function handleChunkFolderRemoving(child)
    if child == nil or not isChunkFolderName(child.Name) then
        return
    end
    local now = os.clock()
    flickerChunkRemoveTimes[child.Name] = now
end

local function ensureFlickerWatchers(worldRoot)
    if worldRoot == flickerWatchedWorldRoot then
        return
    end
    disconnectFlickerWatchers()
    flickerChunkAddTimes = {}
    flickerChunkRemoveTimes = {}
    flickerChunkThrashCounts = {}
    if worldRoot == nil then
        return
    end
    flickerWatchedWorldRoot = worldRoot
    -- Seed current chunk folders as already-added so the first window has a
    -- baseline. Only scan direct children (one level) — cheap.
    local now = os.clock()
    for _, child in ipairs(worldRoot:GetChildren()) do
        if isChunkFolderName(child.Name) then
            flickerChunkAddTimes[child.Name] = now
        end
    end
    table.insert(flickerWatchedConnections, worldRoot.ChildAdded:Connect(handleChunkFolderAdded))
    table.insert(flickerWatchedConnections, worldRoot.ChildRemoved:Connect(handleChunkFolderRemoving))
end

local function countNearPartsAround(rootPart)
    if rootPart == nil or not rootPart:IsA("BasePart") then
        return 0
    end
    local cframe = CFrame.new(rootPart.Position)
    local okParts, parts = pcall(function()
        return Workspace:GetPartBoundsInBox(cframe, FLICKER_NEAR_BOX_SIZE, flickerOverlapParams)
    end)
    if not okParts or type(parts) ~= "table" then
        return 0
    end
    return #parts
end

local function pushNearPartSample(count, ringNear, ringMid, ringFar)
    nearPartRingHead = nearPartRingHead % FLICKER_WINDOW_CAPACITY + 1
    nearPartRing[nearPartRingHead] = count
    ringResidentNearRing[nearPartRingHead] = ringNear
    ringResidentMidRing[nearPartRingHead] = ringMid
    ringResidentFarRing[nearPartRingHead] = ringFar
    if nearPartRingCount < FLICKER_WINDOW_CAPACITY then
        nearPartRingCount = nearPartRingCount + 1
    end
end

-- Count direction reversals in the near-part-count ring. This is the key
-- signal to distinguish "steady chunk admission" (monotonic growth, all
-- deltas same sign → zero reversals) from "thrash" (adds interleaved
-- with removes → many reversals). stdDev alone can't tell these apart:
-- a ring that reads [10, 20, 30, 40, 50] has the same stdDev ≈ 14 as
-- [10, 50, 10, 50, 10] but the first is healthy import progress and the
-- second is visible flicker. We want both metrics on the wire.
local NEAR_PART_REVERSAL_DELTA = 2  -- ignore noise below this part delta
local function countNearPartReversals()
    if nearPartRingCount < 3 then
        return 0, 0, 0
    end
    local reversals = 0
    local upSteps = 0
    local downSteps = 0
    local prevDir = 0
    local prevValue = nearPartRing[1]
    for i = 2, nearPartRingCount do
        local v = nearPartRing[i]
        local diff = v - prevValue
        if math.abs(diff) >= NEAR_PART_REVERSAL_DELTA then
            local dir = if diff > 0 then 1 else -1
            if dir == 1 then
                upSteps = upSteps + 1
            else
                downSteps = downSteps + 1
            end
            if prevDir ~= 0 and dir ~= prevDir then
                reversals = reversals + 1
            end
            prevDir = dir
            prevValue = v
        end
    end
    return reversals, upSteps, downSteps
end

local function summarizeNearPartRing()
    if nearPartRingCount == 0 then
        return 0, 0, 0, 0, 0
    end
    local sum = 0
    local minV = math.huge
    local maxV = -math.huge
    for i = 1, nearPartRingCount do
        local v = nearPartRing[i]
        sum = sum + v
        if v < minV then
            minV = v
        end
        if v > maxV then
            maxV = v
        end
    end
    local avg = sum / nearPartRingCount
    local varSum = 0
    for i = 1, nearPartRingCount do
        local d = nearPartRing[i] - avg
        varSum = varSum + d * d
    end
    local stdDev = math.sqrt(varSum / nearPartRingCount)
    return avg, stdDev, minV, maxV, nearPartRingCount
end

local function summarizeRingResidentDelta(ring)
    if nearPartRingCount == 0 then
        return 0
    end
    local minV = math.huge
    local maxV = -math.huge
    for i = 1, nearPartRingCount do
        local v = ring[i]
        if v < minV then
            minV = v
        end
        if v > maxV then
            maxV = v
        end
    end
    if minV == math.huge then
        return 0
    end
    return maxV - minV
end

local function countRingBounces(ring)
    if nearPartRingCount < 3 then
        return 0
    end
    local bounces = 0
    local prevDir = 0 -- -1 = descending, +1 = ascending
    local lastExtreme = ring[1]
    for i = 2, nearPartRingCount do
        local v = ring[i]
        local diff = v - lastExtreme
        if math.abs(diff) >= FLICKER_RING_BOUNCE_DELTA_BYTES then
            local dir = if diff > 0 then 1 else -1
            if prevDir ~= 0 and dir ~= prevDir then
                bounces = bounces + 1
            end
            prevDir = dir
            lastExtreme = v
        end
    end
    return bounces
end

local function pruneFlickerChunkMaps(windowStart)
    for id, ts in pairs(flickerChunkAddTimes) do
        if ts < windowStart then
            flickerChunkAddTimes[id] = nil
        end
    end
    for id, ts in pairs(flickerChunkRemoveTimes) do
        if ts < windowStart then
            flickerChunkRemoveTimes[id] = nil
        end
    end
    for id, _ in pairs(flickerChunkThrashCounts) do
        if flickerChunkAddTimes[id] == nil and flickerChunkRemoveTimes[id] == nil then
            flickerChunkThrashCounts[id] = nil
        end
    end
end

local function resolveMemoryPressureLabel()
    local hostLevel = tonumber(Workspace:GetAttribute("ArnisStreamingMemoryGuardrailHostPressureLevel"))
    local nearLevel = tonumber(Workspace:GetAttribute("ArnisStreamingRingNearPressureLevel")) or 0
    local midLevel = tonumber(Workspace:GetAttribute("ArnisStreamingRingMidPressureLevel")) or 0
    local farLevel = tonumber(Workspace:GetAttribute("ArnisStreamingRingFarPressureLevel")) or 0
    local peak = math.max(hostLevel or 0, nearLevel, midLevel, farLevel)
    if peak >= 3 then
        return "critical"
    elseif peak >= 2 then
        return "high"
    elseif peak >= 1 then
        return "elevated"
    end
    return "ok"
end

local function sampleFlickerDetector(now)
    if now - flickerLastSampleAt < FLICKER_SAMPLE_INTERVAL then
        return
    end
    flickerLastSampleAt = now
    local worldRoot = getWorldRoot()
    ensureFlickerWatchers(worldRoot)
    local rootPart = getCharacterRootPart()
    local nearCount = 0
    if rootPart and worldRoot then
        nearCount = countNearPartsAround(rootPart)
    end
    -- Accumulate per-sample HRP displacement. The next emit will classify
    -- the window as stationary or moving based on flickerWindowMovementStuds
    -- and then reset the counter. We track the peak single-sample step so we
    -- can distinguish "teleport" thrash (walk path) from "continuous walk".
    if rootPart then
        local pos = rootPart.Position
        if flickerLastSamplePosition ~= nil then
            local step = (pos - flickerLastSamplePosition).Magnitude
            flickerWindowMovementStuds = flickerWindowMovementStuds + step
            if step > flickerWindowMovementPeak then
                flickerWindowMovementPeak = step
            end
        end
        flickerLastSamplePosition = pos
    end
    local ringNear = tonumber(Workspace:GetAttribute("ArnisStreamingRingNearResidentBytes")) or 0
    local ringMid = tonumber(Workspace:GetAttribute("ArnisStreamingRingMidResidentBytes")) or 0
    local ringFar = tonumber(Workspace:GetAttribute("ArnisStreamingRingFarResidentBytes")) or 0
    pushNearPartSample(nearCount, ringNear, ringMid, ringFar)
end

local function publishFlickerTelemetry()
    if not WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "client_flicker") then
        return
    end
    local now = os.clock()
    sampleFlickerDetector(now)
    if now - flickerLastEmitAt < FLICKER_EMIT_INTERVAL then
        return
    end
    flickerLastEmitAt = now

    local windowStart = now - FLICKER_WINDOW_SECONDS
    pruneFlickerChunkMaps(windowStart)

    -- Count ids with BOTH add AND remove within window; track a rolling
    -- thrash count so the same id thrashing multiple times bumps the signal.
    local thrashyIdsList = {}
    local chunkThrashCount = 0
    for id, addTs in pairs(flickerChunkAddTimes) do
        local removeTs = flickerChunkRemoveTimes[id]
        if removeTs ~= nil and addTs >= windowStart and removeTs >= windowStart then
            chunkThrashCount = chunkThrashCount + 1
            flickerChunkThrashCounts[id] = (flickerChunkThrashCounts[id] or 0) + 1
            thrashyIdsList[#thrashyIdsList + 1] = { id = id, count = flickerChunkThrashCounts[id] or 1 }
        end
    end
    table.sort(thrashyIdsList, function(a, b)
        return a.count > b.count
    end)
    local topThrashy = {}
    for i = 1, math.min(5, #thrashyIdsList) do
        topThrashy[i] = thrashyIdsList[i].id
    end

    local avgNear, stdDevNear, minNear, maxNear, sampleCount = summarizeNearPartRing()
    local nearPartReversals, nearPartUpSteps, nearPartDownSteps = countNearPartReversals()
    local ringDeltaNear = summarizeRingResidentDelta(ringResidentNearRing)
    local ringDeltaMid = summarizeRingResidentDelta(ringResidentMidRing)
    local ringDeltaFar = summarizeRingResidentDelta(ringResidentFarRing)

    local ringNearLive = tonumber(Workspace:GetAttribute("ArnisStreamingRingNearResidentBytes")) or 0
    local ringMidLive = tonumber(Workspace:GetAttribute("ArnisStreamingRingMidResidentBytes")) or 0
    local ringFarLive = tonumber(Workspace:GetAttribute("ArnisStreamingRingFarResidentBytes")) or 0

    local fetchFailuresNow = tonumber(Workspace:GetAttribute("ArnisChunkFetchFailures")) or 0
    local fetchFailuresDelta = 0
    if flickerLastFetchFailures ~= nil then
        fetchFailuresDelta = math.max(0, fetchFailuresNow - flickerLastFetchFailures)
    end
    flickerLastFetchFailures = fetchFailuresNow

    local memoryPressure = resolveMemoryPressureLabel()

    local currentNearCount = 0
    if nearPartRingCount > 0 then
        currentNearCount = nearPartRing[nearPartRingHead] or 0
    end

    local windowMovementStuds = flickerWindowMovementStuds
    local windowMovementPeak = flickerWindowMovementPeak
    flickerWindowMovementStuds = 0
    flickerWindowMovementPeak = 0
    local isStationary = windowMovementStuds < FLICKER_STATIONARY_STUDS

    local flickerPayload = {
        windowSeconds = FLICKER_WINDOW_SECONDS,
        sampleCount = sampleCount,
        chunkThrashCount = chunkThrashCount,
        thrashyChunkIds = topThrashy,
        hrpMovementStuds = math.round(windowMovementStuds * 10) / 10,
        hrpMovementPeakStuds = math.round(windowMovementPeak * 10) / 10,
        isStationary = isStationary,
        nearPartReversals = nearPartReversals,
        nearPartUpSteps = nearPartUpSteps,
        nearPartDownSteps = nearPartDownSteps,
        nearPartCount = currentNearCount,
        nearPartCountAvg = math.round(avgNear * 10) / 10,
        nearPartCountStdDev = math.round(stdDevNear * 10) / 10,
        nearPartCountMin = minNear,
        nearPartCountMax = maxNear,
        ringResidentBytesNear = ringNearLive,
        ringResidentBytesMid = ringMidLive,
        ringResidentBytesFar = ringFarLive,
        ringDeltaNear = ringDeltaNear,
        ringDeltaMid = ringDeltaMid,
        ringDeltaFar = ringDeltaFar,
        ringBouncesNear = countRingBounces(ringResidentNearRing),
        ringBouncesMid = countRingBounces(ringResidentMidRing),
        ringBouncesFar = countRingBounces(ringResidentFarRing),
        chunkFetchFailuresDelta = fetchFailuresDelta,
        memoryPressure = memoryPressure,
    }
    WorldProbeTelemetryFlags.annotateMarkerPayload(flickerPayload, telemetryFlags)
    local flickerPayloadJson = HttpService:JSONEncode(flickerPayload)
    print("ARNIS_CLIENT_FLICKER " .. flickerPayloadJson)

    local remote = resolveFlickerClientRemote()
    if remote then
        pcall(function()
            remote:FireServer(flickerPayload)
        end)
    end
end

local function publishWorldTelemetry()
    refreshTelemetryFlags()
    local rootPart = getCharacterRootPart()
    local worldRoot, worldRootName = getWorldRoot()
    local bootstrapPayload = {
        worldRootName = worldRootName,
        worldRootExists = worldRoot ~= nil,
        bootstrapAttemptId = Workspace:GetAttribute("ArnisAustinBootstrapAttemptId"),
        bootstrapState = Workspace:GetAttribute("ArnisAustinBootstrapState"),
        bootstrapStateTrace = Workspace:GetAttribute("ArnisAustinBootstrapStateTrace"),
        bootstrapDuplicateCount = Workspace:GetAttribute("ArnisAustinBootstrapDuplicateCount"),
        bootstrapLastScriptPath = Workspace:GetAttribute("ArnisAustinBootstrapLastScriptPath"),
    }
    local payload = {
        worldRootName = worldRootName,
        worldRootExists = worldRoot ~= nil,
        nearbyBuildingModels = 0,
        nearbyMergedBuildingMeshParts = 0,
        nearbyRoofParts = 0,
        overheadRoofParts = 0,
        nearestBuildingSourceIds = {},
        nearestBuildingDetails = nil,
        overheadRoofSourceIds = {},
        groundMaterial = nil,
        groundInstance = nil,
        supportSurfaceRole = "unknown",
        supportY = nil,
        terrainY = nil,
        supportMinusTerrainYStuds = nil,
        supportSourceIds = {},
        nearbyWallParts = 0,
        nearbyReadableFacadeCueParts = 0,
        collidableWallPartsNearby = 0,
        nearestWallDistanceStuds = nil,
        overheadRoofMinClearanceStuds = nil,
        localSupport = nil,
        localTerrain = nil,
        localEnclosure = nil,
        localRoofCover = nil,
        characterPosition = nil,
        bootstrapAttemptId = bootstrapPayload.bootstrapAttemptId,
        bootstrapState = bootstrapPayload.bootstrapState,
        bootstrapStateTrace = bootstrapPayload.bootstrapStateTrace,
        bootstrapDuplicateCount = bootstrapPayload.bootstrapDuplicateCount,
        bootstrapLastScriptPath = bootstrapPayload.bootstrapLastScriptPath,
    }
    local compactPayload = {
        worldRootName = worldRootName,
        worldRootExists = worldRoot ~= nil,
        nearbyBuildingModels = 0,
        nearbyMergedBuildingMeshParts = 0,
        nearbyRoofParts = 0,
        overheadRoofParts = 0,
        nearestBuildingSourceIds = {},
        overheadRoofSourceIds = {},
        groundMaterial = nil,
        supportSurfaceRole = "unknown",
        supportY = nil,
        terrainY = nil,
        supportMinusTerrainYStuds = nil,
        nearbyWallParts = 0,
        nearbyReadableFacadeCueParts = 0,
        collidableWallPartsNearby = 0,
        nearestWallDistanceStuds = nil,
        overheadRoofMinClearanceStuds = nil,
        localSupport = nil,
        localTerrain = nil,
        localEnclosure = nil,
        localRoofCover = nil,
        bootstrapAttemptId = bootstrapPayload.bootstrapAttemptId,
        bootstrapState = bootstrapPayload.bootstrapState,
        bootstrapStateTrace = bootstrapPayload.bootstrapStateTrace,
        bootstrapDuplicateCount = bootstrapPayload.bootstrapDuplicateCount,
        bootstrapLastScriptPath = bootstrapPayload.bootstrapLastScriptPath,
    }
    local playerLocalTelemetryEnabled = WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "player_local")
    local localExperiencePayload = {
        worldRootName = worldRootName,
        worldRootExists = worldRoot ~= nil,
        playerLocalTelemetryEnabled = false,
        localSupport = nil,
        localTerrain = nil,
        localEnclosure = nil,
        localRoofCover = nil,
        bootstrapAttemptId = bootstrapPayload.bootstrapAttemptId,
        bootstrapState = bootstrapPayload.bootstrapState,
        bootstrapStateTrace = bootstrapPayload.bootstrapStateTrace,
        bootstrapDuplicateCount = bootstrapPayload.bootstrapDuplicateCount,
        bootstrapLastScriptPath = bootstrapPayload.bootstrapLastScriptPath,
    }

    if rootPart and worldRoot then
        payload = summarizeWorld(rootPart, worldRoot, worldRootName, telemetryFlags)
        payload.worldRootExists = true
        compactPayload.worldRootName = payload.worldRootName
        compactPayload.worldRootExists = payload.worldRootExists
        compactPayload.nearbyBuildingModels = payload.nearbyBuildingModels
        compactPayload.nearbyMergedBuildingMeshParts = payload.nearbyMergedBuildingMeshParts
        compactPayload.nearbyRoofParts = payload.nearbyRoofParts
        compactPayload.overheadRoofParts = payload.overheadRoofParts
        compactPayload.nearestBuildingSourceIds = payload.nearestBuildingSourceIds
        compactPayload.nearestNamedBuildingSourceIds = {}
        compactPayload.nearestNamedBuildingNames = {}
        for _, row in ipairs(payload.nearestNamedBuildingDetails or {}) do
            appendLimited(compactPayload.nearestNamedBuildingSourceIds, row.sourceId, MAX_NAMED_BUILDINGS)
            appendLimited(compactPayload.nearestNamedBuildingNames, row.buildingName, MAX_NAMED_BUILDINGS)
        end
        compactPayload.overheadRoofSourceIds = payload.overheadRoofSourceIds
        compactPayload.groundMaterial = payload.groundMaterial
        compactPayload.supportSurfaceRole = payload.supportSurfaceRole
        compactPayload.supportY = payload.supportY
        compactPayload.terrainY = payload.terrainY
        compactPayload.supportMinusTerrainYStuds = payload.supportMinusTerrainYStuds
        compactPayload.nearbyWallParts = payload.nearbyWallParts
        compactPayload.nearbyReadableFacadeCueParts = payload.nearbyReadableFacadeCueParts
        compactPayload.collidableWallPartsNearby = payload.collidableWallPartsNearby
        compactPayload.nearestWallDistanceStuds = payload.nearestWallDistanceStuds
        compactPayload.overheadRoofMinClearanceStuds = payload.overheadRoofMinClearanceStuds
        compactPayload.localSupport = payload.localSupport
        compactPayload.localTerrain = payload.localTerrain
        compactPayload.localEnclosure = payload.localEnclosure
        compactPayload.localRoofCover = payload.localRoofCover
        if playerLocalTelemetryEnabled then
            localExperiencePayload.worldRootName = payload.worldRootName
            localExperiencePayload.worldRootExists = payload.worldRootExists
            localExperiencePayload.localSupport = payload.localSupport
            localExperiencePayload.localTerrain = payload.localTerrain
            localExperiencePayload.localEnclosure = payload.localEnclosure
            localExperiencePayload.localRoofCover = payload.localRoofCover
        end
    end

    setPlayerAttributeIfChanged("ArnisClientWorldRootName", payload.worldRootName)
    setPlayerAttributeIfChanged("ArnisClientWorldRootExists", payload.worldRootExists)
    setPlayerAttributeIfChanged("ArnisClientNearbyBuildingModels", payload.nearbyBuildingModels)
    setPlayerAttributeIfChanged("ArnisClientNearbyMergedBuildingMeshParts", payload.nearbyMergedBuildingMeshParts)
    setPlayerAttributeIfChanged("ArnisClientNearbyRoofParts", payload.nearbyRoofParts)
    setPlayerAttributeIfChanged("ArnisClientOverheadRoofParts", payload.overheadRoofParts)
    setPlayerAttributeIfChanged("ArnisClientGroundMaterial", payload.groundMaterial)
    setPlayerAttributeIfChanged("ArnisClientSupportSurfaceRole", payload.supportSurfaceRole)
    setPlayerAttributeIfChanged(
        "ArnisClientNearbyNamedBuildingNames",
        HttpService:JSONEncode(compactPayload.nearestNamedBuildingNames or {})
    )
    WorldProbeTelemetryFlags.annotateMarkerPayload(bootstrapPayload, telemetryFlags)
    local bootstrapPayloadJson = HttpService:JSONEncode(bootstrapPayload)
    if bootstrapPayloadJson ~= lastBootstrapPayloadJson then
        lastBootstrapPayloadJson = bootstrapPayloadJson
        print("ARNIS_CLIENT_BOOTSTRAP " .. bootstrapPayloadJson)
    end
    WorldProbeTelemetryFlags.annotateMarkerPayload(compactPayload, telemetryFlags)
    local compactPayloadJson = HttpService:JSONEncode(compactPayload)
    if compactPayloadJson ~= lastCompactPayloadJson then
        lastCompactPayloadJson = compactPayloadJson
        print("ARNIS_CLIENT_WORLD_COMPACT " .. compactPayloadJson)
    end
    WorldProbeTelemetryFlags.shapeLocalExperiencePayload(
        localExperiencePayload,
        telemetryFlags,
        playerLocalTelemetryEnabled
    )
    local localExperiencePayloadJson = HttpService:JSONEncode(localExperiencePayload)
    if localExperiencePayloadJson ~= lastLocalExperiencePayloadJson then
        lastLocalExperiencePayloadJson = localExperiencePayloadJson
        print("ARNIS_CLIENT_LOCAL_EXPERIENCE " .. localExperiencePayloadJson)
    end

    WorldProbeTelemetryFlags.annotateMarkerPayload(payload, telemetryFlags)
    local payloadJson = HttpService:JSONEncode(payload)
    if payloadJson == lastPayloadJson then
        return
    end
    lastPayloadJson = payloadJson
    print("ARNIS_CLIENT_WORLD " .. payloadJson)
end

local function maybeSampleWorldTelemetry()
    local rootPart = getCharacterRootPart()
    local _, worldRootName = getWorldRoot()
    local now = os.clock()
    local sampleInterval, resampleDistance, isMoving = resolveMovementAwareSampleCadence(rootPart)
    if isMoving and not lastSampleWasMoving then
        lastSampleAt = 0
    end
    if now - lastSampleAt < sampleInterval then
        return
    end
    if rootPart and lastSamplePosition and lastSampleWorldRootName == worldRootName then
        local displacement = (rootPart.Position - lastSamplePosition).Magnitude
        if displacement < resampleDistance then
            return
        end
    end
    lastSampleAt = now
    if rootPart then
        lastSamplePosition = rootPart.Position
    end
    lastSampleWorldRootName = worldRootName
    lastSampleWasMoving = isMoving
    publishWorldTelemetry()
end

Workspace:GetAttributeChangedSignal(WorldProbeTelemetryFlags.WORKSPACE_ATTR):Connect(function()
    refreshTelemetryFlags()
    publishWorldTelemetry()
end)
player:GetAttributeChangedSignal(WorldProbeTelemetryFlags.PLAYER_ATTR):Connect(function()
    refreshTelemetryFlags()
    publishWorldTelemetry()
end)

player.CharacterAdded:Connect(function()
    lastPayloadJson = nil
    lastSamplePosition = nil
    lastSampleWasMoving = false
    task.defer(publishWorldTelemetry)
end)

Workspace:GetAttributeChangedSignal(WORLD_ROOT_ATTR):Connect(function()
    lastPayloadJson = nil
    lastSamplePosition = nil
    lastSampleWorldRootName = nil
    lastSampleWasMoving = false
    ensurePerfInstanceWatchers(getWorldRoot())
    publishWorldTelemetry()
end)

-- One-time camera initialization: briefly switch to Scriptable to override
-- the orbit camera, position behind+above the character looking at the horizon,
-- then switch back to Custom so the player can orbit normally. Without this,
-- the Studio test-mode orbit camera starts looking upward at the elevated spawn.
task.defer(function()
    local camera = Workspace.CurrentCamera
    local character = player.Character or player.CharacterAdded:Wait()
    local root = character:WaitForChild("HumanoidRootPart", 10)
    if root and camera then
        -- Keep camera Scriptable for 5 seconds so the initial view is stable
        -- for screenshot capture, then restore Custom for player control
        camera.CameraType = Enum.CameraType.Scriptable
        local pos = root.Position
        -- Cycle through camera angles for automated visual proof.
        -- Each angle holds for 8s (enough for capture), then moves to next.
        -- After all angles, restores Custom camera for player control.
        local angles = {
            { pos + Vector3.new(0, 8, 20), pos + Vector3.new(0, 4, -40), "street_behind" },
            { pos + Vector3.new(20, 15, 20), pos + Vector3.new(-20, 4, -20), "street_diagonal" },
            { pos + Vector3.new(0, 60, 60), pos + Vector3.new(0, 0, -30), "aerial_approach" },
            { pos + Vector3.new(0, 120, 0), pos + Vector3.new(0, 0, -1), "aerial_topdown" },
        }
        for _, angle in ipairs(angles) do
            camera.CFrame = CFrame.lookAt(angle[1], angle[2])
            print("ARNIS_CAMERA_ANGLE " .. angle[3])
            task.wait(8)
        end
        camera.CameraType = Enum.CameraType.Custom
    end
end)

RunService.Heartbeat:Connect(function(dt)
    recordFrameTime(dt)
    publishPerfTelemetry()
    publishFlickerTelemetry()
    maybeSampleWorldTelemetry()
end)

task.defer(publishWorldTelemetry)
