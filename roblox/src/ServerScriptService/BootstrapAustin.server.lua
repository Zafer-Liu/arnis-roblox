--[[
  BootstrapAustin.server.lua
  Automatically imports the Austin, TX manifest when the game starts.
  This runs on Play (server-side) so you can open Studio, hit Play, and see Austin.

  To disable: set ENABLED = false below.
--]]

local ENABLED = true

if not ENABLED then
    return
end

local Players = game:GetService("Players")
local Workspace = game:GetService("Workspace")
local RunService = game:GetService("RunService")

local AustinSpawn = require(script.Parent.ImportService.AustinSpawn)
local BootstrapStateMachine = require(script.Parent.ImportService.BootstrapStateMachine)
local CanonicalWorldContract = require(script.Parent.ImportService.CanonicalWorldContract)
local RunAustin = require(script.Parent.ImportService.RunAustin)
local SceneAudit = require(script.Parent.ImportService.SceneAudit)
local SceneMarkerEmitter = require(script.Parent.ImportService.SceneMarkerEmitter)
local StreamingService = require(script.Parent.ImportService.StreamingService)
local SubplanRollout = require(script.Parent.ImportService.SubplanRollout)
local WorldStateApplier = require(script.Parent.ImportService.WorldStateApplier)
local WorldConfig = require(game:GetService("ReplicatedStorage").Shared.WorldConfig)
local StreamingRuntimeConfig = require(game:GetService("ReplicatedStorage").Shared.StreamingRuntimeConfig)
local BOOTSTRAP_STATE_ATTR = BootstrapStateMachine.STATE_ATTR
local BOOTSTRAP_STATE_TRACE_ATTR = BootstrapStateMachine.STATE_TRACE_ATTR
local BOOTSTRAP_DUPLICATE_COUNT_ATTR = BootstrapStateMachine.DUPLICATE_COUNT_ATTR
local BOOTSTRAP_ENTRY_COUNT_ATTR = BootstrapStateMachine.ENTRY_COUNT_ATTR
local BOOTSTRAP_LAST_SCRIPT_PATH_ATTR = BootstrapStateMachine.LAST_SCRIPT_PATH_ATTR
local BOOTSTRAP_ATTEMPT_ID_ATTR = "ArnisAustinBootstrapAttemptId"
local STARTUP_STREAMING_TIMEOUT_SECONDS = 10
local STARTUP_STREAMING_POLL_INTERVAL_SECONDS = 0.1
local STARTUP_STREAMING_REQUIRED_READY_POLLS = 3

if not RunService:IsStudio() then
    warn("[BootstrapAustin] Refusing to auto-import Austin outside Studio.")
    return
end

local bootstrapMachine, duplicateAttempt = BootstrapStateMachine.begin(Workspace, script:GetFullName())
if duplicateAttempt then
    warn(
        "[BootstrapAustin] Duplicate bootstrap attempt ignored. state=",
        duplicateAttempt.state,
        " entries=",
        duplicateAttempt.entryCount,
        " script=",
        script:GetFullName()
    )
    return
end

local function setBootstrapState(state)
    if type(state) ~= "string" or state == "" then
        return
    end
    if Workspace:GetAttribute(BOOTSTRAP_STATE_ATTR) == state then
        return
    end
    BootstrapStateMachine.transition(bootstrapMachine, state)
end

Workspace:GetAttribute(BOOTSTRAP_ATTEMPT_ID_ATTR)

Players.CharacterAutoLoads = false

local importReady = false
local spawnCFrame
local holdingPad

local WALKABLE_WORLD_GROUPS = table.freeze({
    Terrain = true,
    Roads = true,
    Landuse = true,
    Rails = true,
})

local function isWalkableWorldDescendant(hitInstance, worldRoot)
    if not hitInstance or not worldRoot then
        return false
    end
    if not hitInstance:IsDescendantOf(worldRoot) then
        return false
    end

    local node = hitInstance
    while node and node.Parent and node.Parent ~= worldRoot do
        node = node.Parent
    end

    return node ~= nil and WALKABLE_WORLD_GROUPS[node.Name] == true
end

local function isDecorativeRoadDetailDescendant(hitInstance, worldRoot)
    if not hitInstance or not worldRoot then
        return false
    end
    if not hitInstance:IsDescendantOf(worldRoot) then
        return false
    end

    local node = hitInstance
    while node and node.Parent and node.Parent ~= worldRoot do
        if node.Name == "Detail" and node.Parent and node.Parent.Name == "Roads" then
            return true
        end
        node = node.Parent
    end

    return false
end

local function isValidGroundHit(hitInstance, worldRoot, loadingPad, spawn)
    if not hitInstance then
        return false
    end
    if loadingPad and hitInstance:IsDescendantOf(loadingPad) then
        return false
    end
    if spawn and hitInstance:IsDescendantOf(spawn) then
        return false
    end
    if hitInstance == Workspace.Terrain then
        return true
    end
    if isDecorativeRoadDetailDescendant(hitInstance, worldRoot) then
        return false
    end
    if isWalkableWorldDescendant(hitInstance, worldRoot) then
        return true
    end
    return false
end

local function findGroundYNear(worldRoot, point, loadingPad, spawn)
    local ignore = {}
    if loadingPad then
        table.insert(ignore, loadingPad)
    end
    if spawn then
        table.insert(ignore, spawn)
    end

    local params = RaycastParams.new()
    params.FilterType = Enum.RaycastFilterType.Exclude
    params.FilterDescendantsInstances = ignore

    local rayOrigin = Vector3.new(point.X, point.Y + 2000, point.Z)
    local rayDirection = Vector3.new(0, -4000, 0)

    for _ = 1, 8 do
        local hit = Workspace:Raycast(rayOrigin, rayDirection, params)
        if not hit then
            break
        end
        if isValidGroundHit(hit.Instance, worldRoot, loadingPad, spawn) then
            return hit.Position.Y
        end
        table.insert(ignore, hit.Instance)
        params.FilterDescendantsInstances = ignore
    end

    warn(
        string.format(
            "[BootstrapAustin] No valid ground hit near spawn anchor (x=%.1f, y=%.1f, z=%.1f); falling back to manifest Y",
            point.X,
            point.Y,
            point.Z
        )
    )
    return point.Y
end

local function getCharacterSpawnCFrame(character)
    local extents = character:GetExtentsSize()
    local spawnLift = math.max(6, extents.Y * 0.5 + 0.5)
    local basePosition = spawnCFrame.Position
    local lookVector = spawnCFrame.LookVector
    local elevatedPosition = basePosition + Vector3.new(0, spawnLift, 0)
    return CFrame.lookAt(elevatedPosition, elevatedPosition + lookVector)
end

local function moveCharacterToSpawn(character)
    local root = character:FindFirstChild("HumanoidRootPart") or character:WaitForChild("HumanoidRootPart", 10)
    if root and spawnCFrame then
        local characterSpawnCFrame = getCharacterSpawnCFrame(character)
        root.AssemblyLinearVelocity = Vector3.zero
        root.AssemblyAngularVelocity = Vector3.zero
        character:PivotTo(characterSpawnCFrame)
    end
end

local function removeCharacterUntilImportReady(player, character)
    if importReady then
        return
    end

    task.defer(function()
        if player.Character == character and character.Parent then
            character:Destroy()
        end
    end)
end

local function waitForStartupStreamingReady(spawnPoint)
    if typeof(spawnPoint) ~= "Vector3" then
        return true
    end

    local deadline = os.clock() + STARTUP_STREAMING_TIMEOUT_SECONDS
    local readyPollCount = 0
    while os.clock() < deadline do
        StreamingService.Update(spawnPoint)
        local startupResidency = StreamingService.GetStartupResidencySnapshot(spawnPoint, "GeneratedWorld_Austin")
        if startupResidency.ready then
            readyPollCount += 1
            if readyPollCount >= STARTUP_STREAMING_REQUIRED_READY_POLLS then
                return true
            end
        else
            readyPollCount = 0
        end
        task.wait(STARTUP_STREAMING_POLL_INTERVAL_SECONDS)
    end

    StreamingService.Update(spawnPoint)
    local finalResidency = StreamingService.GetStartupResidencySnapshot(spawnPoint, "GeneratedWorld_Austin")
    return finalResidency.ready and readyPollCount + 1 >= STARTUP_STREAMING_REQUIRED_READY_POLLS
end

local function onPlayer(player)
    player.CharacterAdded:Connect(function(character)
        if importReady and spawnCFrame then
            task.defer(function()
                moveCharacterToSpawn(character)
            end)
        else
            removeCharacterUntilImportReady(player, character)
        end
    end)

    if player.Character and not importReady then
        removeCharacterUntilImportReady(player, player.Character)
    end

    if importReady and not player.Character then
        player:LoadCharacter()
    end
end

for _, player in ipairs(Players:GetPlayers()) do
    onPlayer(player)
end
Players.PlayerAdded:Connect(onPlayer)

print("[BootstrapAustin] Starting Austin, TX import...")

-- Note: StreamingEnabled and Terrain.SmoothingEnabled must be configured
-- in Studio settings (File > Game Settings > Streaming) — not scriptable.

holdingPad = Instance.new("Part")
holdingPad.Name = "AustinLoadingPad"
holdingPad.Anchored = true
holdingPad.CanCollide = false
holdingPad.Transparency = 1
holdingPad.Size = Vector3.new(64, 1, 64)
holdingPad.CFrame = CFrame.new(0, 300, 0)
holdingPad.Parent = Workspace

local runtimeWorldConfig = StreamingRuntimeConfig.Resolve(WorldConfig)

local result = RunAustin.run({
    config = runtimeWorldConfig,
    phaseReporter = setBootstrapState,
})
if result == nil then
    BootstrapStateMachine.fail(bootstrapMachine)
    warn("[BootstrapAustin] Austin manifest unavailable; skipping bootstrap.")
    if holdingPad then
        holdingPad:Destroy()
    end
    Players.CharacterAutoLoads = true
    for _, player in ipairs(Players:GetPlayers()) do
        if not player.Character then
            player:LoadCharacter()
        end
    end
    return
end

local manifest = result.manifest
local manifestSource = result.manifestSource or manifest
local worldRoot = Workspace:FindFirstChild("GeneratedWorld_Austin")
setBootstrapState("world_ready")

print("[BootstrapAustin] Done.")

local anchor = AustinSpawn.resolveRuntimeAnchor(manifestSource, RunAustin.LOAD_RADIUS, result.focusPoint)
local spawnPoint = result.spawnPoint or anchor.spawnPoint
local spawn = Instance.new("SpawnLocation")
spawn.Name = "CongressAveSpawn"
spawn.Size = Vector3.new(6, 1, 6)
spawn.Anchored = true
spawn.CanCollide = false
spawn.Neutral = true
spawn.Material = Enum.Material.Concrete
spawn.BrickColor = BrickColor.new("Medium stone grey")
spawn.Transparency = 1
spawn.Parent = Workspace

for _, player in ipairs(Players:GetPlayers()) do
    player.RespawnLocation = spawn
end
Players.PlayerAdded:Connect(function(player)
    player.RespawnLocation = spawn
end)

local spawnSurfaceY = findGroundYNear(worldRoot, spawnPoint, holdingPad, spawn)
local spawnCenterY = spawnSurfaceY + spawn.Size.Y * 0.5
local preferredLookTarget = result.lookTarget or anchor.lookTarget
local lookTarget = Vector3.new(preferredLookTarget.X, spawnSurfaceY, preferredLookTarget.Z)
if (lookTarget - Vector3.new(spawnPoint.X, spawnSurfaceY, spawnPoint.Z)).Magnitude < 1 then
    lookTarget = Vector3.new(spawnPoint.X, spawnSurfaceY, spawnPoint.Z - 1)
end
spawnCFrame = CFrame.lookAt(Vector3.new(spawnPoint.X, spawnSurfaceY, spawnPoint.Z), lookTarget)
spawn.CFrame = CFrame.new(spawnPoint.X, spawnCenterY, spawnPoint.Z)

WorldStateApplier.Apply(manifestSource, runtimeWorldConfig, {
    worldRootName = "GeneratedWorld_Austin",
    startMinimap = true,
    hideLoadingScreen = true,
})

if runtimeWorldConfig.StreamingEnabled then
    local rolloutDescription = SubplanRollout.Describe(runtimeWorldConfig)
    print(
        ("[BootstrapAustin] Streaming profile=%s rollout enabled=%s mode=%s layers=%d chunks=%d"):format(
            tostring(runtimeWorldConfig.StreamingProfile),
            tostring(rolloutDescription.enabled),
            tostring(rolloutDescription.mode),
            rolloutDescription.allowedLayerCount,
            rolloutDescription.allowlistedChunkCount
        )
    )
    StreamingService.Start(manifestSource, {
        worldRootName = "GeneratedWorld_Austin",
        config = runtimeWorldConfig,
        nonBlocking = true,
        frameBudgetSeconds = runtimeWorldConfig.StreamingImportFrameBudgetSeconds,
        preferredLookVector = lookTarget - Vector3.new(spawnPoint.X, spawnSurfaceY, spawnPoint.Z),
    })
    local streamingStartupReady = waitForStartupStreamingReady(spawnPoint)
    if not streamingStartupReady then
        warn("[BootstrapAustin] Startup streaming did not settle the near ring before gameplay readiness.")
    end
    setBootstrapState("streaming_ready")
    print("[BootstrapAustin] StreamingService started.")
else
    setBootstrapState("streaming_ready")
end

importReady = true
Players.CharacterAutoLoads = true

for _, player in ipairs(Players:GetPlayers()) do
    if player.Character then
        moveCharacterToSpawn(player.Character)
    else
        player:LoadCharacter()
    end
end

if runtimeWorldConfig.EnableMinimap ~= false and Workspace:GetAttribute("ArnisMinimapEnabled") ~= true then
    warn("[BootstrapAustin] Minimap expected to be enabled before gameplay readiness, but readiness marker was absent.")
end
setBootstrapState("minimap_ready")

holdingPad:Destroy()

print("[BootstrapAustin] Spawn and shared world state configured.")
setBootstrapState("gameplay_ready")

task.defer(function()
    if not worldRoot or worldRoot.Parent ~= Workspace then
        return
    end
    local sceneSummary = SceneAudit.summarizeWorld(worldRoot)
    SceneMarkerEmitter.emitSceneMarkers(
        "ARNIS_SCENE_PLAY",
        "play",
        worldRoot.Name,
        RunAustin.LOAD_RADIUS,
        sceneSummary,
        {
            worldIdentity = CanonicalWorldContract.resolveCanonicalManifestFamily("play"),
            chunkEnvelopeKind = "runtime_resident",
        }
    )
end)
