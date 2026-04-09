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

local AtmosphereConfig = require(script.Parent.ImportService.AtmosphereConfig)
local AustinSpawn = require(script.Parent.ImportService.AustinSpawn)
local BootstrapStateMachine = require(script.Parent.ImportService.BootstrapStateMachine)
local CanonicalWorldContract = require(script.Parent.ImportService.CanonicalWorldContract)
local HarnessRouteConfig = require(script.Parent.ImportService.HarnessRouteConfig)
local LoadingScreen = require(script.Parent.ImportService.LoadingScreen)
local RunAustin = require(script.Parent.ImportService.RunAustin)
local SceneAudit = require(script.Parent.ImportService.SceneAudit)
local SceneMarkerEmitter = require(script.Parent.ImportService.SceneMarkerEmitter)
local StreamingService = require(script.Parent.ImportService.StreamingService)
local SubplanRollout = require(script.Parent.ImportService.SubplanRollout)
local TelemetryReporter = require(script.Parent.ImportService.TelemetryReporter)
local WorldStateApplier = require(script.Parent.ImportService.WorldStateApplier)
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local WorldConfig = require(ReplicatedStorage.Shared.WorldConfig)
local StreamingRuntimeConfig = require(ReplicatedStorage.Shared.StreamingRuntimeConfig)
local BOOTSTRAP_STATE_ATTR = BootstrapStateMachine.STATE_ATTR
local BOOTSTRAP_STATE_TRACE_ATTR = BootstrapStateMachine.STATE_TRACE_ATTR
local BOOTSTRAP_DUPLICATE_COUNT_ATTR = BootstrapStateMachine.DUPLICATE_COUNT_ATTR
local BOOTSTRAP_ENTRY_COUNT_ATTR = BootstrapStateMachine.ENTRY_COUNT_ATTR
local BOOTSTRAP_LAST_SCRIPT_PATH_ATTR = BootstrapStateMachine.LAST_SCRIPT_PATH_ATTR
local BOOTSTRAP_ATTEMPT_ID_ATTR = "ArnisAustinBootstrapAttemptId"
local FLICKER_REMOTE_NAME = "ArnisClientFlickerRemote"

-- Install the client->server flicker sample RemoteEvent. WorldProbe.client.lua
-- fires each ARNIS_CLIENT_FLICKER sample through this remote so the server
-- can aggregate it via TelemetryReporter.RecordFlickerSample and include a
-- `flicker` block in the next TelemetryReporter.Report() POST.
local function ensureFlickerRemote()
    local existing = ReplicatedStorage:FindFirstChild(FLICKER_REMOTE_NAME)
    if existing
        and (existing:IsA("UnreliableRemoteEvent") or existing:IsA("RemoteEvent"))
    then
        return existing
    end
    if existing then
        existing:Destroy()
    end
    local remote
    local okCreate, created = pcall(function()
        return Instance.new("UnreliableRemoteEvent")
    end)
    if okCreate and created then
        remote = created
    else
        remote = Instance.new("RemoteEvent")
    end
    remote.Name = FLICKER_REMOTE_NAME
    remote.Parent = ReplicatedStorage
    return remote
end

local flickerRemote = ensureFlickerRemote()
flickerRemote.OnServerEvent:Connect(function(_player, sample)
    if type(sample) ~= "table" then
        return
    end
    TelemetryReporter.RecordFlickerSample(sample)
end)
-- Post-osm2world: 1017 chunks at 3.3MB avg need more than 10s to settle
-- the near ring. 60s gives the streaming system time to fetch + import
-- the ~48 near-ring chunks at ~2s each.
local STARTUP_STREAMING_TIMEOUT_SECONDS = 60
local STARTUP_STREAMING_POLL_INTERVAL_SECONDS = 0.1
local STARTUP_STREAMING_REQUIRED_READY_POLLS = 3

-- NOTE: Previously gated to Studio-only. Removed to enable live Roblox
-- deployment. The bootstrap now runs on both Studio and production servers.

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

local PHASE_STATUS_TEXT = table.freeze({
    loading_manifest = "Fetching manifest from Cloudflare planetary worker...",
    importing_startup = "Importing startup chunks...",
    world_ready = "World ready — building runtime state...",
    streaming_ready = "Streaming online — staging chunks...",
    minimap_ready = "Minimap ready — finalizing spawn...",
    gameplay_ready = "Gameplay ready",
})

local PHASE_PROGRESS = table.freeze({
    loading_manifest = 0.10,
    importing_startup = 0.40,
    world_ready = 0.65,
    streaming_ready = 0.85,
    minimap_ready = 0.95,
    gameplay_ready = 1.00,
})

local function setBootstrapState(state)
    if type(state) ~= "string" or state == "" then
        return
    end
    if Workspace:GetAttribute(BOOTSTRAP_STATE_ATTR) ~= state then
        BootstrapStateMachine.transition(bootstrapMachine, state)
    end
    TelemetryReporter.RecordPhase(state)
    local statusText = PHASE_STATUS_TEXT[state] or state
    local fraction = PHASE_PROGRESS[state]
    pcall(function()
        LoadingScreen.UpdateProgress(fraction, statusText)
    end)
end

-- Stamp a unique attempt id so background coroutines (the prefetch hint
-- in RunAustin, for one) can observe whether their parent bootstrap run
-- is still the active one. The value is just a monotonic server clock
-- timestamp concatenated with a short random suffix — good enough to
-- distinguish sequential attempts without pulling in HttpService:GenerateGUID.
local bootstrapAttemptId = string.format(
    "%d-%04x",
    math.floor(os.clock() * 1000),
    math.random(0, 0xFFFF)
)
Workspace:SetAttribute(BOOTSTRAP_ATTEMPT_ID_ATTR, bootstrapAttemptId)

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

local function resolveHarnessRouteSelection()
    if type(HarnessRouteConfig) ~= "table" then
        return {}
    end

    local selection = {}
    -- telemetryFamilies is harness-wide configuration and is independent of
    -- the route-catalog `enabled` gate. The harness rewrites just the
    -- telemetryFamilies field when no route is requested, so we must read it
    -- even when HarnessRouteConfig.enabled is false. Without this, perf
    -- telemetry (and any other family-gated marker) silently never emits.
    if type(HarnessRouteConfig.telemetryFamilies) == "string" then
        selection.telemetryFamilies = HarnessRouteConfig.telemetryFamilies
    end

    if HarnessRouteConfig.enabled ~= true then
        return selection
    end

    if type(HarnessRouteConfig.routeCatalogName) == "string" and HarnessRouteConfig.routeCatalogName ~= "" then
        selection.routeCatalogName = HarnessRouteConfig.routeCatalogName
    end
    if type(HarnessRouteConfig.routeLane) == "string" and HarnessRouteConfig.routeLane ~= "" then
        selection.routeLane = HarnessRouteConfig.routeLane
    end
    if type(HarnessRouteConfig.routeStepIndex) == "number" then
        selection.routeStepIndex = math.floor(HarnessRouteConfig.routeStepIndex)
    end
    return selection
end

local harnessRouteSelection = resolveHarnessRouteSelection()

-- Ensure client_flicker is ALWAYS enabled, regardless of what the
-- harness script pre-rewrites into HarnessRouteConfig.telemetryFamilies.
-- The harness writes "client_perf" at build time; live prod starts with
-- "". Neither includes client_flicker by default, which silenced the
-- flicker detector in WorldProbe.client.lua and made the user's real
-- flickering report invisible to the auto-loop.
--
-- We force-merge client_flicker (and guarantee client_perf) into the
-- families list so both harness and live prod emit the signal.
do
    local current = harnessRouteSelection.telemetryFamilies
    if type(current) ~= "string" then
        current = ""
    end
    local hasPerf = string.find(current, "client_perf", 1, true) ~= nil
    local hasFlicker = string.find(current, "client_flicker", 1, true) ~= nil
    local merged = current
    if not hasPerf then
        merged = merged == "" and "client_perf" or (merged .. ",client_perf")
    end
    if not hasFlicker then
        merged = merged == "" and "client_flicker" or (merged .. ",client_flicker")
    end
    harnessRouteSelection.telemetryFamilies = merged
end

local function onPlayer(player)
    player:SetAttribute("ArnisTelemetryFamilies", harnessRouteSelection.telemetryFamilies)
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

-- Show the loading screen on every player so the live (native) Roblox player
-- always has a visible status surface — without it, the user has nothing to
-- look at while the manifest streams from Cloudflare and falls into the void
-- if anything fails. This must run before manifest load so we can also show
-- bootstrap errors via LoadingScreen.ShowError when load fails.
pcall(function()
    LoadingScreen.Show("Austin, TX")
    LoadingScreen.UpdateProgress(0.05, "Connecting to Cloudflare planetary worker...")
end)

-- Note: StreamingEnabled and Terrain.SmoothingEnabled must be configured
-- in Studio settings (File > Game Settings > Streaming) — not scriptable.

holdingPad = Instance.new("Part")
holdingPad.Name = "AustinLoadingPad"
holdingPad.Anchored = true
holdingPad.CanCollide = true -- character stands on pad during load
holdingPad.Transparency = 1
holdingPad.Size = Vector3.new(256, 4, 256) -- wide enough to catch any drift
holdingPad.CFrame = CFrame.new(0, 500, 0) -- high above the world
holdingPad.Parent = Workspace

local runtimeConfigSource = WorldConfig
if not RunService:IsStudio() then
    local configuredProfile = WorldConfig.StreamingProfile
    if type(configuredProfile) ~= "string" or configuredProfile == "" or configuredProfile == "local_dev" then
        runtimeConfigSource = table.clone(WorldConfig)
        runtimeConfigSource.StreamingProfile = "production_server"
    end
end
local runtimeWorldConfig = StreamingRuntimeConfig.Resolve(runtimeConfigSource)
if harnessRouteSelection.routeCatalogName then
    Workspace:SetAttribute("VertigoRouteCatalogName", harnessRouteSelection.routeCatalogName)
    Workspace:SetAttribute("VertigoRouteLane", harnessRouteSelection.routeLane)
    Workspace:SetAttribute("VertigoRouteStepIndex", harnessRouteSelection.routeStepIndex)
end
ReplicatedStorage:SetAttribute("ArnisTelemetryFamilies", harnessRouteSelection.telemetryFamilies)
Workspace:SetAttribute("ArnisTelemetryFamilies", harnessRouteSelection.telemetryFamilies)
for _, player in ipairs(Players:GetPlayers()) do
    player:SetAttribute("ArnisTelemetryFamilies", harnessRouteSelection.telemetryFamilies)
end

-- Run the full bootstrap inside a pcall so ANY uncaught exception (not
-- just the manifest-unavailable case RunAustin.run handles internally)
-- still triggers the structured failure path: error overlay, telemetry
-- POST, and clean holdingPad teardown. Without this wrap, an exception
-- inside ImportService.ImportManifest or a downstream builder crashes
-- the bootstrap script entirely and no failure telemetry is ever
-- emitted, which is the exact symptom that hid a RoadBuilder crash from
-- the remote harness auto-loop.
local result, runError
local runOk, runReturn1, runReturn2 = pcall(function()
    return RunAustin.run({
        config = runtimeWorldConfig,
        phaseReporter = setBootstrapState,
        routeCatalogName = harnessRouteSelection.routeCatalogName,
        routeLane = harnessRouteSelection.routeLane,
        routeStepIndex = harnessRouteSelection.routeStepIndex,
    })
end)
if runOk then
    result = runReturn1
    runError = runReturn2
else
    result = nil
    runError = tostring(runReturn1)
end
if result == nil then
    BootstrapStateMachine.fail(bootstrapMachine)
    local bootstrapFailureKind = if runOk then "manifest_unavailable" else "import_failed"
    local manifestSourceConfig = WorldConfig.ManifestSource or {}
    local sourceDescription
    if manifestSourceConfig.mode == "external_url" then
        sourceDescription = ("external_url=%s"):format(tostring(manifestSourceConfig.externalUrl))
    elseif manifestSourceConfig.mode == "roblox_asset" then
        sourceDescription = ("roblox_asset=%s"):format(tostring(manifestSourceConfig.robloxAssetId))
    else
        sourceDescription = ("mode=%s"):format(tostring(manifestSourceConfig.mode or "embedded"))
    end
    local errMessage = if bootstrapFailureKind == "import_failed"
        then "Austin import failed — bootstrap halted."
        else "Austin manifest unavailable — bootstrap halted."
    local errDetail = ("source: %s\nerror: %s"):format(
        sourceDescription,
        tostring(runError or "(no error returned)")
    )
    warn(("[BootstrapAustin] %s\n%s"):format(errMessage, errDetail))
    pcall(function()
        LoadingScreen.ShowError(errMessage, errDetail)
    end)
    TelemetryReporter.Report({
        status = "failed",
        errorMessage = errMessage,
        errorDetail = errDetail,
        chunksImported = 0,
        totalInstances = 0,
        totalFeatures = 0,
    })
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
AtmosphereConfig.Apply()

print(
    ("[BootstrapAustin] Manifest source kind=%s name=%s"):format(
        result.manifestSourceKind or "canonical_manifest",
        result.resolvedManifestName or RunAustin.getManifestName()
    )
)
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
    -- Server-authoritative multiplayer streaming hooks. These are no-ops on
    -- the single-player path because the streaming Update() loop only consults
    -- the player focus registry when MultiplayerStreaming.enabled is true in
    -- the resolved world config. We still install the player lifecycle and
    -- client position remote unconditionally so a runtime config flip (e.g.
    -- experimental flag) can enable multiplayer streaming without restarting.
    if type(StreamingService.BindPlayerLifecycle) == "function" then
        StreamingService.BindPlayerLifecycle(spawnPoint)
    end
    if type(StreamingService.BindClientPositionRemote) == "function" then
        StreamingService.BindClientPositionRemote()
    end
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

local telemetryStats = result.stats or {}
local telemetryFeatureTotal = (telemetryStats.roadsImported or 0)
    + (telemetryStats.railsImported or 0)
    + (telemetryStats.buildingsImported or 0)
    + (telemetryStats.waterImported or 0)
    + (telemetryStats.propsImported or 0)
    + (telemetryStats.landuseImported or 0)
    + (telemetryStats.barriersImported or 0)
-- Initial success report — fires immediately so the auto-loop audit has a
-- bootstrap record to anchor on. Subsequent reports give us actual flicker
-- data:
--   * stationary_baseline fires at T+20s with a clean 20s window of
--     stand-still samples. This is the diagnosis signal for "the world
--     flickers even when I'm not moving" — if stationaryThrashCount > 0 or
--     stationaryStdDevAvg > epsilon we have a reproducer.
--   * heartbeat fires every 30s thereafter for the rest of the session,
--     letting us watch flicker state evolve as the user plays.
TelemetryReporter.Report({
    status = "success",
    chunksImported = telemetryStats.chunksImported or 0,
    totalInstances = telemetryStats.totalInstances or 0,
    totalFeatures = telemetryFeatureTotal,
})

local BASELINE_WINDOW_SECONDS = 20
local HEARTBEAT_INTERVAL_SECONDS = 30
-- Cap the heartbeat loop so a runaway (or harness session that never
-- naturally exits) can't fire telemetry forever. 240 heartbeats at 30s
-- each = 2 hours of continuous observation — more than enough for any
-- interactive session, and bounded for the harness which runs ~3 minutes
-- of play-wait. After the cap the session falls silent on telemetry but
-- continues normally otherwise.
local HEARTBEAT_MAX_ITERATIONS = 240
task.spawn(function()
    -- Fresh aggregate so the baseline window starts from zero, not
    -- whatever was captured during the first 2-3s post-gameplay_ready.
    TelemetryReporter.ResetFlickerAggregate()
    task.wait(BASELINE_WINDOW_SECONDS)
    TelemetryReporter.Report({
        status = "stationary_baseline",
        chunksImported = telemetryStats.chunksImported or 0,
        totalInstances = telemetryStats.totalInstances or 0,
        totalFeatures = telemetryFeatureTotal,
    })
    TelemetryReporter.ResetFlickerAggregate()
    for _ = 1, HEARTBEAT_MAX_ITERATIONS do
        task.wait(HEARTBEAT_INTERVAL_SECONDS)
        TelemetryReporter.Report({
            status = "heartbeat",
            chunksImported = telemetryStats.chunksImported or 0,
            totalInstances = telemetryStats.totalInstances or 0,
            totalFeatures = telemetryFeatureTotal,
        })
        TelemetryReporter.ResetFlickerAggregate()
    end
end)

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
            manifestSourceKind = result.manifestSourceKind or "canonical_manifest",
            manifestSourceName = result.resolvedManifestName or RunAustin.getManifestName(),
        }
    )
end)
