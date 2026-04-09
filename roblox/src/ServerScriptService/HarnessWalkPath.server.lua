--[[
    HarnessWalkPath.server.lua

    Scripted character walk path for the Studio harness, used to exercise
    StreamingService against chunk-boundary crossings that a stationary
    harness run never triggers. Teleports the first player's character
    through a 10-waypoint spiral spanning ~512 studs (2 chunks) in every
    direction, pausing between hops so the streaming update tick can
    admit/evict chunks and the flicker detector in WorldProbe.client.lua
    can accumulate real signal.

    Gated on `RunService:IsStudio()` so live production (native Roblox
    player) never runs this. Dev Studio sessions also run it by default —
    set the Workspace attribute `ArnisHarnessWalkPathDisabled=true`
    before BootstrapAustin reaches gameplay_ready to opt out.

    Output:
      - [HarnessWalkPath] starting scripted walk from (X,Y,Z)
      - [HarnessWalkPath] waypoint K/N → (X,Y,Z)
      - [HarnessWalkPath] walk complete — duration S
      - Workspace attribute `ArnisHarnessWalkPathStatus` = "idle" |
        "running" | "complete" | "failed" for audit pickup.
]]

local RunService = game:GetService("RunService")
local Players = game:GetService("Players")
local Workspace = game:GetService("Workspace")
local ServerScriptService = game:GetService("ServerScriptService")

-- Optional require of TelemetryReporter for post-walk re-report. We
-- soft-require so this script survives in any context where the
-- ImportService tree isn't present (edge cases in isolated specs).
local TelemetryReporter
do
    local importService = ServerScriptService:FindFirstChild("ImportService")
    if importService then
        local reporterModule = importService:FindFirstChild("TelemetryReporter")
        if reporterModule then
            local ok, mod = pcall(require, reporterModule)
            if ok then
                TelemetryReporter = mod
            end
        end
    end
end

local BOOTSTRAP_STATE_ATTR = "ArnisAustinBootstrapState"
local WALK_STATUS_ATTR = "ArnisHarnessWalkPathStatus"
local WALK_DISABLED_ATTR = "ArnisHarnessWalkPathDisabled"

if not RunService:IsStudio() then
    Workspace:SetAttribute(WALK_STATUS_ATTR, "skipped_live")
    return
end

Workspace:SetAttribute(WALK_STATUS_ATTR, "idle")

-- Spiral of 10 waypoints in XZ around the spawn point. Each hop crosses
-- one to two chunk boundaries (chunkSizeStuds=256) and the full path
-- visits all 8 compass octants plus two further-out jumps. Pauses of
-- WAYPOINT_PAUSE_SECONDS between hops give StreamingService time to
-- admit/evict and the flicker detector a full rolling-window sample.
local WAYPOINTS_RELATIVE = table.freeze({
    Vector3.new(256, 0, 0),     -- E
    Vector3.new(256, 0, 256),   -- SE
    Vector3.new(0, 0, 256),     -- S
    Vector3.new(-256, 0, 256),  -- SW
    Vector3.new(-256, 0, 0),    -- W
    Vector3.new(-256, 0, -256), -- NW
    Vector3.new(0, 0, -256),    -- N
    Vector3.new(256, 0, -256),  -- NE
    Vector3.new(512, 0, 0),     -- E x2 (further out)
    Vector3.new(0, 0, 0),       -- back to spawn
})

local WAYPOINT_PAUSE_SECONDS = 2.5
local GAMEPLAY_READY_TIMEOUT_SECONDS = 90
local CHARACTER_TIMEOUT_SECONDS = 30

local function waitForGameplayReady(timeoutSeconds)
    local deadline = os.clock() + timeoutSeconds
    while os.clock() < deadline do
        local state = Workspace:GetAttribute(BOOTSTRAP_STATE_ATTR)
        if state == "gameplay_ready" then
            return true
        end
        if state == "failed" then
            return false
        end
        task.wait(0.5)
    end
    return false
end

local function waitForPlayerCharacter(timeoutSeconds)
    local deadline = os.clock() + timeoutSeconds
    while os.clock() < deadline do
        local players = Players:GetPlayers()
        if #players > 0 then
            local player = players[1]
            local character = player.Character
            if character and character.Parent then
                local hrp = character:FindFirstChild("HumanoidRootPart")
                local humanoid = character:FindFirstChildOfClass("Humanoid")
                if hrp and humanoid then
                    return character, hrp, humanoid
                end
            end
        end
        task.wait(0.5)
    end
    return nil
end

task.spawn(function()
    if Workspace:GetAttribute(WALK_DISABLED_ATTR) == true then
        Workspace:SetAttribute(WALK_STATUS_ATTR, "disabled_by_attribute")
        return
    end

    local readyOk = waitForGameplayReady(GAMEPLAY_READY_TIMEOUT_SECONDS)
    if not readyOk then
        warn("[HarnessWalkPath] gameplay_ready never reached within "
            .. tostring(GAMEPLAY_READY_TIMEOUT_SECONDS) .. "s")
        Workspace:SetAttribute(WALK_STATUS_ATTR, "failed_no_gameplay_ready")
        return
    end

    if Workspace:GetAttribute(WALK_DISABLED_ATTR) == true then
        Workspace:SetAttribute(WALK_STATUS_ATTR, "disabled_by_attribute")
        return
    end

    local character, hrp, humanoid = waitForPlayerCharacter(CHARACTER_TIMEOUT_SECONDS)
    if not character or not hrp or not humanoid then
        warn("[HarnessWalkPath] no player character within "
            .. tostring(CHARACTER_TIMEOUT_SECONDS) .. "s")
        Workspace:SetAttribute(WALK_STATUS_ATTR, "failed_no_character")
        return
    end

    -- Extra settle delay after gameplay_ready so the first LoadingScreen
    -- fade completes and the character physics stabilize before we
    -- start teleporting.
    task.wait(1.5)

    -- Anchor + lift the character into invincible flight mode for the
    -- duration of the walk. Teleporting a gravity-bound character across
    -- chunks causes it to fall into void between loaded chunks and die
    -- (that was the "character destroyed at waypoint 10" bug from the
    -- previous harness run). Anchoring the HRP and lifting to Y+200
    -- sidesteps the physics simulation entirely — the character glides
    -- like a spectator between teleports, triggering StreamingService
    -- via the HRP position changes without dying.
    local FLIGHT_LIFT_STUDS = 200
    local priorHrpAnchored = hrp.Anchored
    local priorHumanoidState = humanoid:GetState()
    local priorPlatformStand = humanoid.PlatformStand
    hrp.Anchored = true
    humanoid.PlatformStand = true
    pcall(function()
        humanoid:ChangeState(Enum.HumanoidStateType.Physics)
    end)

    local spawnPos = hrp.Position + Vector3.new(0, FLIGHT_LIFT_STUDS, 0)
    hrp.CFrame = CFrame.new(spawnPos) * (hrp.CFrame - hrp.CFrame.Position)
    hrp.AssemblyLinearVelocity = Vector3.zero
    hrp.AssemblyAngularVelocity = Vector3.zero

    print(string.format(
        "[HarnessWalkPath] starting scripted walk from (%.1f, %.1f, %.1f) [flight mode]",
        spawnPos.X, spawnPos.Y, spawnPos.Z
    ))
    Workspace:SetAttribute(WALK_STATUS_ATTR, "running")
    local walkStart = os.clock()
    local waypointsReached = 0

    for i, rel in ipairs(WAYPOINTS_RELATIVE) do
        if Workspace:GetAttribute(WALK_DISABLED_ATTR) == true then
            Workspace:SetAttribute(WALK_STATUS_ATTR, "aborted_by_attribute")
            break
        end
        if not character.Parent then
            warn("[HarnessWalkPath] character destroyed at waypoint " .. tostring(i))
            Workspace:SetAttribute(WALK_STATUS_ATTR, "failed_character_lost")
            break
        end

        local target = spawnPos + rel
        print(string.format(
            "[HarnessWalkPath] waypoint %d/%d → (%.1f, %.1f, %.1f)",
            i, #WAYPOINTS_RELATIVE, target.X, target.Y, target.Z
        ))

        local ok, err = pcall(function()
            local currentCFrame = hrp.CFrame
            hrp.CFrame = CFrame.new(target)
                * (currentCFrame - currentCFrame.Position)
            hrp.AssemblyLinearVelocity = Vector3.zero
            hrp.AssemblyAngularVelocity = Vector3.zero
        end)
        if not ok then
            warn("[HarnessWalkPath] teleport failed at waypoint "
                .. tostring(i) .. ": " .. tostring(err))
        else
            waypointsReached += 1
        end

        task.wait(WAYPOINT_PAUSE_SECONDS)
    end

    -- Restore character state so the harness can continue normally if
    -- the remaining play-wait is used for anything else.
    if character.Parent and hrp.Parent then
        hrp.Anchored = priorHrpAnchored
        humanoid.PlatformStand = priorPlatformStand
        pcall(function()
            humanoid:ChangeState(priorHumanoidState)
        end)
    end

    local duration = os.clock() - walkStart
    print(string.format(
        "[HarnessWalkPath] walk complete — duration %.2fs, reached %d/%d waypoints",
        duration, waypointsReached, #WAYPOINTS_RELATIVE
    ))
    Workspace:SetAttribute(WALK_STATUS_ATTR, "complete")
    Workspace:SetAttribute("ArnisHarnessWalkPathWaypointsReached", waypointsReached)
    Workspace:SetAttribute("ArnisHarnessWalkPathDurationSeconds", duration)

    -- Second telemetry report with the accumulated walk-window flicker
    -- samples. The original Report() at gameplay_ready fired ~20+
    -- seconds before this walk started, so its flicker block only
    -- contained pre-walk stationary samples (all zeroes). Firing a
    -- NEW record now captures the walk window's signal — the whole
    -- reason this scripted walk exists.
    if TelemetryReporter ~= nil and type(TelemetryReporter.Report) == "function" then
        local ok, err = pcall(function()
            TelemetryReporter.Report({
                status = "success_post_walk",
                chunksImported = 0,
                totalInstances = 0,
                totalFeatures = 0,
                errorMessage = nil,
                errorDetail = ("walk reached %d/%d waypoints in %.2fs"):format(
                    waypointsReached,
                    #WAYPOINTS_RELATIVE,
                    duration
                ),
            })
        end)
        if not ok then
            warn("[HarnessWalkPath] post-walk TelemetryReporter.Report failed: "
                .. tostring(err))
        else
            print("[HarnessWalkPath] post-walk telemetry report fired")
        end
    end
end)
