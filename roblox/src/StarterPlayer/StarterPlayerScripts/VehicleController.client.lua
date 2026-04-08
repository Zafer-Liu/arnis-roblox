--[[
    VehicleController.client.lua
    AAA-grade player vehicle mechanics: car, jetpack, parachute,
    wingsuit, and grapple hook.

    DISPATCHER — this file is the thin entry point that wires
    together the mode modules under `Traversal/`. All shared state,
    constants, utilities, and HUD construction live in
    `Traversal/SharedState.lua`. All mode-specific behavior lives in
    `Traversal/CarController.lua`, `JetpackController.lua`,
    `ParachuteController.lua`, `WingsuitController.lua`, and
    `GrappleController.lua`.

    This file owns: character lifecycle, key bindings, mode
    transitions, full cleanup, and the per-frame render loop that
    fans out into each mode's update function.

    Keybinds:
        V       Spawn car / enter nearby car / exit car
        J       Toggle jetpack
        P       Deploy / retract parachute (must be falling)
        WASD    Drive / steer (car) or directional thrust (jetpack/parachute)
        Space   Handbrake (car) / ascend (jetpack)
        LShift  Descend (jetpack)
        H       Horn (car)
        E       Exit vehicle (car)
        A/D     Bank left/right (parachute)
        S       Flare / increase angle of attack (parachute)
        G       Grapple hook (Just Cause style)
        C       Cinematic orbit camera
]]

-- Client scripts replicate asynchronously; `script.Parent.Traversal` can
-- return nil for a few milliseconds after the dispatcher runs but before
-- the Traversal folder arrives. WaitForChild with a bounded timeout
-- blocks until the folder + submodules exist. Without this, running on
-- a real Roblox client (not Studio) throws:
--   `Traversal is not a valid member of PlayerScripts`
local traversalFolder = script.Parent:WaitForChild("Traversal", 10)
if traversalFolder == nil then
    error("[VehicleController] Traversal module folder failed to replicate within 10s")
end
local SharedState = require(traversalFolder:WaitForChild("SharedState", 5))
local CarController = require(traversalFolder:WaitForChild("CarController", 5))
local JetpackController = require(traversalFolder:WaitForChild("JetpackController", 5))
local ParachuteController = require(traversalFolder:WaitForChild("ParachuteController", 5))
local WingsuitController = require(traversalFolder:WaitForChild("WingsuitController", 5))
local GrappleController = require(traversalFolder:WaitForChild("GrappleController", 5))

local S = SharedState

local Players = S.Players
local UserInputService = S.UserInputService
local RunService = S.RunService
local player = S.player

-- Local aliases of frequently called mode functions (no semantic change,
-- just avoids repeating module prefix in the transitions/input/render blocks).
local spawnCar = CarController.spawnCar
local destroyCar = CarController.destroyCar
local enterCar = CarController.enterCar
local exitCar = CarController.exitCar
local updateCar = CarController.updateCar

local deployJetpack = JetpackController.deployJetpack
local cleanupJetpack = JetpackController.cleanupJetpack
local updateJetpack = JetpackController.updateJetpack

local deployParachute = ParachuteController.deployParachute
local retractParachute = ParachuteController.retractParachute
local updateParachute = ParachuteController.updateParachute

local enterWingsuit = WingsuitController.enterWingsuit
local exitWingsuit = WingsuitController.exitWingsuit
local updateWingsuit = WingsuitController.updateWingsuit

local fireGrapple = GrappleController.fireGrapple
local releaseGrapple = GrappleController.releaseGrapple
local updateGrapple = GrappleController.updateGrapple

--------------------------------------------------------------------------------
-- TRANSITIONS
--------------------------------------------------------------------------------
-- Fix #3: all transitions use task.delay instead of task.wait (no yielding in render thread)
local function transitionToJetpack()
    if S.transitionLock then
        return
    end
    S.transitionLock = true

    local hrp = S.getHRP()

    -- If wingsuit active, exit it first (no task.delay needed, purely local cleanup)
    if S.mode == "wingsuit" then
        exitWingsuit()
    end

    -- If in car, eject upward then deploy after delay
    if S.mode == "car" then
        exitCar()
        if hrp then
            hrp.AssemblyLinearVelocity = hrp.AssemblyLinearVelocity + Vector3.new(0, 40, 0)
        end
        task.delay(0.3, function()
            deployJetpack()
            S.transitionLock = false
        end)
        return
    end

    -- If parachute active, retract first
    if S.mode == "parachute" then
        retractParachute()
        task.delay(0.15, function()
            deployJetpack()
            S.transitionLock = false
        end)
        return
    end

    deployJetpack()
    S.transitionLock = false
end

local function transitionToParachute()
    if S.transitionLock then
        return
    end
    S.transitionLock = true

    -- If wingsuit active, drop it before deploying chute (smooth handoff)
    if S.mode == "wingsuit" then
        exitWingsuit()
    end

    -- If jetpack active, swap seamlessly
    if S.mode == "jetpack" then
        cleanupJetpack()
        task.delay(0.1, function()
            deployParachute()
            S.transitionLock = false
        end)
        return
    end

    -- If in car, eject first
    if S.mode == "car" then
        exitCar()
        task.delay(0.2, function()
            deployParachute()
            S.transitionLock = false
        end)
        return
    end

    deployParachute()
    S.transitionLock = false
end

local function transitionToCar()
    if S.transitionLock then
        return
    end
    S.transitionLock = true

    local function doCarEntry()
        -- Check if a car already exists nearby (proximity entry)
        if S.carModel and S.carSeat then
            local hrp = S.getHRP()
            if hrp and (hrp.Position - S.carSeat.Position).Magnitude < 15 then
                enterCar(S.carSeat)
                S.transitionLock = false
                return
            else
                destroyCar()
            end
        end

        if not S.carModel then
            local seat = spawnCar()
            if seat then
                task.delay(0.3, function()
                    enterCar(seat)
                    S.transitionLock = false
                end)
                return
            end
        end

        S.transitionLock = false
    end

    -- Clean up other modes
    if S.mode == "wingsuit" then
        exitWingsuit()
    end
    if S.mode == "jetpack" then
        cleanupJetpack()
        task.delay(0.1, doCarEntry)
        return
    end
    if S.mode == "parachute" then
        retractParachute()
        task.delay(0.1, doCarEntry)
        return
    end

    doCarEntry()
end

local function toggleCarMode()
    if S.mode == "car" then
        exitCar()
    else
        task.spawn(transitionToCar)
    end
end

local function toggleJetpackMode()
    if S.mode == "jetpack" then
        cleanupJetpack()
    else
        task.spawn(transitionToJetpack)
    end
end

local function toggleParachuteMode()
    if S.mode == "parachute" then
        retractParachute()
    else
        task.spawn(transitionToParachute)
    end
end

--------------------------------------------------------------------------------
-- CLEANUP ON DEATH / RESPAWN
--------------------------------------------------------------------------------
local function fullCleanup()
    if S.mode == "car" then
        exitCar()
    end
    cleanupJetpack()
    retractParachute()
    exitWingsuit()
    releaseGrapple()
    destroyCar()

    S.mode = "none"
    S.customCamActive = false
    S.cinematicMode = false
    S.camCurrentPos = nil
    S.jetpackFuel = S.JETPACK_FUEL_MAX

    S.disableDOF()

    S.restoreDefaultCamera(S.getHumanoid())

    S.setHUDMode("none")
end

local function onCharacterAdded(character)
    fullCleanup()

    local hum = character:WaitForChild("Humanoid", 10)
    if hum then
        S.restoreDefaultCamera(hum)
        S.publishClientCameraTelemetry(hum)
        hum.Died:Connect(function()
            fullCleanup()
        end)
    end
end

player.CharacterAdded:Connect(onCharacterAdded)
if player.Character then
    onCharacterAdded(player.Character)
end

-- Cleanup on leave
Players.PlayerRemoving:Connect(function(p)
    if p == player then
        fullCleanup()
    end
end)

--------------------------------------------------------------------------------
-- INPUT
--------------------------------------------------------------------------------
UserInputService.InputBegan:Connect(function(input, gameProcessed)
    if gameProcessed then
        return
    end
    local character = S.getCharacter()
    if not character then
        return
    end

    local keyCode = input.KeyCode

    if keyCode == Enum.KeyCode.V or keyCode == Enum.KeyCode.ButtonY then
        toggleCarMode()
    elseif keyCode == Enum.KeyCode.J or keyCode == Enum.KeyCode.ButtonX then
        toggleJetpackMode()
    elseif keyCode == Enum.KeyCode.P or (keyCode == Enum.KeyCode.ButtonA and S.mode ~= "car") then
        toggleParachuteMode()
    elseif keyCode == Enum.KeyCode.G or keyCode == Enum.KeyCode.ButtonL1 then
        fireGrapple()
    elseif keyCode == Enum.KeyCode.E and S.mode == "car" then
        exitCar()
    elseif keyCode == Enum.KeyCode.H and S.mode == "car" then
        if S.carHornSound and not S.carHornSound.IsPlaying then
            S.carHornSound:Play()
        end
    elseif keyCode == Enum.KeyCode.C then
        S.cinematicMode = not S.cinematicMode
        if S.cinematicMode then
            S.enableCinematicDOF()
            S.controlHints.Text = "[C] Exit cinematic view"
            S.controlHintTimer = S.HUD_FADE_DELAY
            if not S.controlHintsVisible then
                S.controlHintsVisible = true
                S.tweenProperty(S.controlHints, { TextTransparency = 0, BackgroundTransparency = 0.4 }, 0.3)
            end
        else
            S.disableDOF()
            S.restoreDefaultCamera(S.getHumanoid())
            S.setHUDMode(S.mode)
        end
    elseif keyCode == Enum.KeyCode.ButtonB then
        -- ButtonB exits the current active mode
        if S.mode == "car" then
            exitCar()
        elseif S.mode == "jetpack" then
            cleanupJetpack()
        elseif S.mode == "parachute" then
            retractParachute()
        end
    end
end)

--------------------------------------------------------------------------------
-- MAIN RENDER LOOP
--------------------------------------------------------------------------------
RunService.RenderStepped:Connect(function(dt)
    -- Fix #12: resolve character references once at top of render loop
    local char = S.getCharacter()
    if not char then
        return
    end
    local hrp = char:FindFirstChild("HumanoidRootPart")
    local hum = char:FindFirstChildOfClass("Humanoid")
    if not hrp then
        return
    end

    -- Fix #10: read gamepad once per frame
    S.frameGamepad = S.readGamepad()

    -- Update active mode
    updateCar(dt)
    updateJetpack(dt)
    updateParachute(dt)
    updateWingsuit(dt)
    updateGrapple(dt)

    -- Update HUD values
    S.updateHUDValues(dt)

    -- Jetpack fuel recharge on ground
    if not S.jetpackActive and S.isOnGround() then
        S.jetpackFuel = math.min(S.JETPACK_FUEL_MAX, S.jetpackFuel + S.JETPACK_FUEL_RECHARGE_RATE * dt)
    end

    -- Detect if player fell out of car seat
    if S.mode == "car" and S.carSeat then
        if hum and not hum.Sit then
            S.mode = "none"
            S.customCamActive = false
            S.camCurrentPos = nil -- fix #11
            S.restoreDefaultCamera(hum)
            S.setHUDMode("none")
        end
    end

    -- Anti-fall-through safety net for car
    if S.mode == "car" and S.carBody then
        if S.carBody.Position.Y < -50 then
            S.carBody.CFrame = CFrame.new(S.lastSafePosition + Vector3.new(0, 5, 0))
            S.carBody.AssemblyLinearVelocity = Vector3.zero
            S.carBody.AssemblyAngularVelocity = Vector3.zero
        else
            S.lastSafePosition = S.carBody.Position
        end
    end

    -- Cinematic orbit camera (fix #8: lerp FOV instead of hard-set 60)
    if S.cinematicMode then
        local camera = S.getCamera()
        if not camera then
            return
        end
        S.cinematicAngle = S.cinematicAngle + dt * 0.3
        local radius = 150
        local height = 80
        local target = hrp.Position
        local camPos = target
            + Vector3.new(math.cos(S.cinematicAngle) * radius, height, math.sin(S.cinematicAngle) * radius)
        camera.CameraType = Enum.CameraType.Scriptable
        camera.CFrame = CFrame.lookAt(camPos, target)
        camera.FieldOfView = S.lerp(camera.FieldOfView, 60, math.min(1, 4 * dt))
    elseif S.mode == "none" and not S.customCamActive and hum then
        local camera = S.getCamera()
        if camera and (camera.CameraType == Enum.CameraType.Fixed or camera.CameraSubject ~= hum) then
            S.restoreDefaultCamera(hum)
        end
    end

    if hum then
        S.publishClientCameraTelemetry(hum)
    end

    -- Fix #11: removed prevMode comparison; camCurrentPos is now reset in exit functions
end)
