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
local SAMPLE_INTERVAL = 1.5
local NEARBY_BUILDING_RADIUS = 260
local OVERHEAD_ROOF_RADIUS = 220
local OVERHEAD_MIN_DELTA_Y = 12
local NEARBY_WALL_RADIUS = 180
local RESAMPLE_DISTANCE = 24
local MAX_BUILDING_IDS = 6
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
local lastSampleAt = 0
local lastSamplePosition = nil
local lastSampleWorldRootName = nil
local telemetryFamilies = Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)
local telemetryFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(telemetryFamilies)

local function refreshTelemetryFlags()
    local nextTelemetryFamilies = Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)
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

local function appendLimited(list, value, limit)
    if #list >= limit then
        return
    end
    list[#list + 1] = value
end

local function roundTenths(value)
    if type(value) ~= "number" then
        return nil
    end
    return math.round(value * 10) / 10
end

local function isRoofClosureDeckPart(part)
    if part == nil then
        return false
    end
    if part:GetAttribute("ArnisRoofClosureDeck") == true then
        return true
    end

    return string.find(string.lower(part.Name), "roof_closure", 1, true) ~= nil
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
        supportMinusTerrainYStuds = if supportY ~= nil and terrainY ~= nil then roundTenths(supportY - terrainY) else nil,
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
        }
    end

    return WorldProbeTerrain.summarizeTerrainSamples(samples, {
        centerIndex = 3,
        samplePattern = LOCAL_TERRAIN_SAMPLE_PATTERN,
        sampleRadiusStuds = LOCAL_TERRAIN_SAMPLE_RADIUS,
        neighborPairs = LOCAL_TERRAIN_NEIGHBOR_PAIRS,
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
    local collidableWallPartsNearby = 0
    local nearestWallDistanceStuds = nil
    local nearestBuildingSourceIds = {}
    local overheadRoofSourceIds = {}
    local nearestBuildingDetails = nil
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

        local mergedMeshes = buildingsFolder:FindFirstChild("MergedMeshes")
        if mergedMeshes then
            for _, descendant in ipairs(mergedMeshes:GetDescendants()) do
                if not descendant:IsA("MeshPart") then
                    continue
                end

                local partOffset = descendant.Position - rootPosition
                local horizontalDistance = Vector2.new(partOffset.X, partOffset.Z).Magnitude
                if horizontalDistance <= NEARBY_BUILDING_RADIUS then
                    nearbyMergedBuildingMeshParts += 1
                end
            end
        end

        for _, model in ipairs(buildingsFolder:GetDescendants()) do
            if not model:IsA("Model") or model:GetAttribute("ArnisImportBuildingHeight") == nil then
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
            local horizontalDistance = Vector2.new(offset.X, offset.Z).Magnitude
            if horizontalDistance > NEARBY_BUILDING_RADIUS then
                continue
            end

            nearbyBuildingModels += 1
            appendLimited(nearestBuildingSourceIds, sourceId, MAX_BUILDING_IDS)
            if nearestBuildingDetails ~= nil then
                appendLimited(nearestBuildingDetails, {
                    sourceId = sourceId,
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
                local horizontalPartDistance = Vector2.new(partOffset.X, partOffset.Z).Magnitude
                local isRoofClosureDeck = descendant:GetAttribute("ArnisRoofClosureDeck") == true
                    or isRoofClosureDeckPart(descendant)
                local isRoofPart = string.find(nameLower, "roof", 1, true) ~= nil and not isRoofClosureDeck

                if descendant:IsA("MeshPart") and shellFolder and descendant:IsDescendantOf(shellFolder) then
                    if horizontalPartDistance <= NEARBY_BUILDING_RADIUS and not isRoofPart and not isRoofClosureDeck then
                        nearbyMergedBuildingMeshParts += 1
                    end
                end

                if shellFolder and descendant:IsDescendantOf(shellFolder) and not isRoofPart and not isRoofClosureDeck then
                    local isNearbyShellWall, nearestShellWallDistanceStuds =
                        WorldProbeGeometry.isNearbyShellWall(descendant, rootPosition, NEARBY_WALL_RADIUS)
                    if isNearbyShellWall then
                        nearbyWallParts += 1
                        if descendant.CanCollide then
                            collidableWallPartsNearby += 1
                        end
                        if nearestWallDistanceStuds == nil or nearestShellWallDistanceStuds < nearestWallDistanceStuds then
                            nearestWallDistanceStuds = nearestShellWallDistanceStuds
                        end
                    end
                    continue
                end
                if not isRoofPart then
                    continue
                end

                nearbyRoofParts += 1

                local verticalDelta = partOffset.Y
                if horizontalPartDistance <= OVERHEAD_ROOF_RADIUS and verticalDelta >= OVERHEAD_MIN_DELTA_Y then
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
        collidableWallPartsNearby = collidableWallPartsNearby,
        nearestWallDistanceStuds = roundTenths(nearestWallDistanceStuds),
        nearestBuildingSourceIds = nearestBuildingSourceIds,
        nearestBuildingDetails = nearestBuildingDetails,
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
        compactPayload.overheadRoofSourceIds = payload.overheadRoofSourceIds
        compactPayload.groundMaterial = payload.groundMaterial
        compactPayload.supportSurfaceRole = payload.supportSurfaceRole
        compactPayload.supportY = payload.supportY
        compactPayload.terrainY = payload.terrainY
        compactPayload.supportMinusTerrainYStuds = payload.supportMinusTerrainYStuds
        compactPayload.nearbyWallParts = payload.nearbyWallParts
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
    if now - lastSampleAt < SAMPLE_INTERVAL then
        return
    end
    if rootPart and lastSamplePosition and lastSampleWorldRootName == worldRootName then
        local displacement = (rootPart.Position - lastSamplePosition).Magnitude
        if displacement < RESAMPLE_DISTANCE then
            return
        end
    end
    lastSampleAt = now
    if rootPart then
        lastSamplePosition = rootPart.Position
    end
    lastSampleWorldRootName = worldRootName
    publishWorldTelemetry()
end

Workspace:GetAttributeChangedSignal(WorldProbeTelemetryFlags.WORKSPACE_ATTR):Connect(function()
    refreshTelemetryFlags()
    publishWorldTelemetry()
end)

player.CharacterAdded:Connect(function()
    lastPayloadJson = nil
    lastSamplePosition = nil
    task.defer(publishWorldTelemetry)
end)

Workspace:GetAttributeChangedSignal(WORLD_ROOT_ATTR):Connect(function()
    lastPayloadJson = nil
    lastSamplePosition = nil
    lastSampleWorldRootName = nil
    publishWorldTelemetry()
end)

RunService.Heartbeat:Connect(function()
    maybeSampleWorldTelemetry()
end)

task.defer(publishWorldTelemetry)
