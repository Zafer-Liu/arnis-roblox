--[[
    Traversal/SharedState.lua

    Shared services, constants, state table, utility helpers, and HUD
    construction for the traversal system. Mode-specific modules
    (CarController, JetpackController, ParachuteController,
    WingsuitController, GrappleController) all receive the returned
    table `S` and mutate its state fields directly. This preserves
    the original monolith's single-shared-scope semantics bit-for-bit.

    Every constant, state variable, and utility is lifted from
    VehicleController.client.lua (pre-split) without semantic change.
]]

local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local TweenService = game:GetService("TweenService")
local CollectionService = game:GetService("CollectionService")
local Debris = game:GetService("Debris")
local HttpService = game:GetService("HttpService")
local Lighting = game:GetService("Lighting")
local Workspace = game:GetService("Workspace")

local player = Players.LocalPlayer
local playerGui = player:WaitForChild("PlayerGui")

local S = {}

-- Services
S.Players = Players
S.UserInputService = UserInputService
S.RunService = RunService
S.TweenService = TweenService
S.CollectionService = CollectionService
S.Debris = Debris
S.HttpService = HttpService
S.Lighting = Lighting
S.Workspace = Workspace
S.player = player
S.playerGui = playerGui

--------------------------------------------------------------------------------
-- Constants
--------------------------------------------------------------------------------
S.CAR_TAG = "PlayerVehiclePart"
S.JETPACK_TAG = "JetpackPart"
S.PARACHUTE_TAG = "ParachutePart"

-- Car physics — Far Cry / Just Cause feel: fast, drifty, weighty
S.CAR_MAX_SPEED = 180 -- studs/s (~54m/s, fast and dynamic like Just Cause)
S.CAR_BOOST_SPEED = 280 -- studs/s during nitro boost
S.CAR_BOOST_DURATION = 3.0 -- seconds of nitro
S.CAR_BOOST_COOLDOWN = 8.0 -- seconds between boosts
S.CAR_MOTOR_ANGULAR_VEL = 55 -- rad/s at full throttle
S.CAR_TORQUE = 2400 -- aggressive acceleration
S.CAR_STEER_ANGLE = 38 -- degrees — responsive
S.CAR_STEER_SPEED = 10 -- quick snap into turns
S.CAR_HIGH_SPEED_STEER_FACTOR = 0.35 -- tight at speed
S.SUSPENSION_REST_LENGTH = 1.6
S.SUSPENSION_STIFFNESS = 1400 -- very firm — planted feeling
S.SUSPENSION_DAMPING = 150 -- no bounce, just bite
S.DRIFT_GRIP_REDUCTION = 0.3 -- more slidey drift (Far Cry style)
S.ENGINE_IDLE_VIBRATION = 0.02
-- Rapier-style acceleration: jerk-limited smooth ramp instead of instant torque
S.CAR_ACCEL_RAMP_TIME = 0.6 -- seconds from 0 to full torque (smooth curve)
S.CAR_DECEL_RAMP_TIME = 0.3 -- faster braking response than acceleration
-- Per-road-surface friction (matches OSM surface tags compiled in manifest).
-- Multiplies the base wheel friction. 1.0 = standard asphalt grip.
S.SURFACE_FRICTION = {
    asphalt = 1.0,
    concrete = 0.95,
    paved = 1.0,
    paving_stones = 0.85,
    cobblestone = 0.7,
    sett = 0.7,
    unpaved = 0.6,
    compacted = 0.65,
    gravel = 0.5,
    fine_gravel = 0.55,
    pebblestone = 0.45,
    dirt = 0.5,
    earth = 0.5,
    grass = 0.45,
    sand = 0.35,
    mud = 0.3,
    ice = 0.15,
    snow = 0.25,
    wood = 0.7,
    metal = 0.6,
    rubber = 1.05,
    default = 0.85,
}
-- Dynamic FOV
S.CAR_FOV_MIN = 70
S.CAR_FOV_MAX = 95 -- cinematic warp at top speed
S.CAR_FOV_BOOST = 105 -- extreme during nitro

-- Jetpack physics — Far Cry 4 buzzer / Just Cause thrust vectoring
S.JETPACK_MAX_THRUST = 6500 -- snappier response
S.JETPACK_BOOST_THRUST = 12000 -- hold shift for burst (Just Cause style)
S.JETPACK_BOOST_FUEL_COST = 3.0 -- fuel/second during boost
S.JETPACK_RAMP_TIME = 0.25 -- faster ramp for responsiveness
S.JETPACK_DAMPING = 0.45 -- slightly less drag for more momentum
S.JETPACK_MAX_HORIZONTAL_SPEED = 120 -- studs/s — fast traverse
S.JETPACK_MAX_VERTICAL_SPEED = 80 -- studs/s — quick ascent
S.JETPACK_FUEL_MAX = 45 -- seconds (forces strategic use)
S.JETPACK_FUEL_RECHARGE_RATE = 0.8 -- faster recharge for dynamic play
-- Rapier-style angular damping: smooth rotation instead of snappy
S.JETPACK_TILT_SPEED = 4.0 -- degrees/frame toward velocity
S.JETPACK_MAX_TILT = 25 -- degrees of character lean

-- Parachute physics — Just Cause wingsuit/chute hybrid
S.CHUTE_GLIDE_RATIO = 4.5 -- better glide (Just Cause style)
S.CHUTE_DESCENT_RATE = -6 -- studs/s (slower, more hang time)
S.CHUTE_FORWARD_SPEED = math.abs(S.CHUTE_DESCENT_RATE) * S.CHUTE_GLIDE_RATIO
S.CHUTE_TURN_RATE = 2.0 -- rad/s, responsive banking
S.CHUTE_FLARE_LIFT = 6 -- decent flare for precision landings
S.CHUTE_FLARE_STALL_TIME = 3.0
S.CHUTE_STALL_DESCENT = -25
S.CHUTE_WIND_STRENGTH = 2.0 -- noticeable wind for dynamic feel
S.CHUTE_HORIZONTAL_DRAG = 0.08 -- less drag = more speed preservation
-- Dive mechanic: hold W to tuck and gain speed, trade altitude for velocity
S.CHUTE_DIVE_DESCENT_RATE = -20 -- studs/s when diving
S.CHUTE_DIVE_FORWARD_SPEED = 60 -- studs/s — fast dive like Just Cause
S.CHUTE_DIVE_RECOVERY_TIME = 0.5 -- seconds to smoothly transition out of dive

-- Grapple hook (Just Cause signature) — raycast from camera, rope-pull toward impact
S.GRAPPLE_MAX_RANGE = 200
S.GRAPPLE_PULL_SPEED = 80
S.GRAPPLE_TIMEOUT = 5
S.GRAPPLE_ARRIVAL_RADIUS = 6 -- studs: auto-release when this close to hit point
S.GRAPPLE_VELOCITY_LERP = 10 -- per-second lerp rate from current vel to pull vel (smooth)
S.GRAPPLE_PULL_MAX_FORCE = 60000 -- BodyVelocity MaxForce component
S.GRAPPLE_BEAM_WIDTH = 0.18

-- Wingsuit (auto from freefall) — Far Cry style glide between jetpack / parachute
S.WINGSUIT_GLIDE_RATIO = 4.0
S.WINGSUIT_FORWARD_SPEED = 90
S.WINGSUIT_DESCENT_RATE = -22
S.WINGSUIT_MIN_FREEFALL_TIME = 2
S.WINGSUIT_TILT_LERP = 4.0 -- per-second gyro/velocity lerp rate (smooth)
S.WINGSUIT_PITCH_DEGREES = 35 -- forward tilt of character in glide pose
S.WINGSUIT_MAX_FORCE_H = 8000
S.WINGSUIT_MAX_FORCE_V = 20000

-- Camera (uses CAR_FOV_MIN/MAX/BOOST from physics block above)
S.CAR_CAM_OFFSET = Vector3.new(0, 8, 22)
S.CAR_CAM_TILT_FACTOR = 0.04
S.CAR_FOV_SPEED_RANGE = 130 -- ramp to max over full speed range

S.JETPACK_CAM_OFFSET = Vector3.new(0, 4, 16)
S.JETPACK_CAM_SHAKE_INTENSITY = 0.15

S.CHUTE_CAM_OFFSET = Vector3.new(0, 10, 24)
S.CHUTE_FOV = 90 -- wide panoramic view of the city

S.DEFAULT_FOV = 70

-- Frame-rate-independent lerp rate (per second)
S.CAM_LERP_RATE = 6

-- Pre-computed jetpack flame tiers (fix #1: zero per-frame NumberSequence allocs)
S.FLAME_SIZE_TIERS = {}
S.FLAME_SPEED_TIERS = {}
for tier = 0, 10 do
    local t = tier / 10
    local baseSize = 0.5 + t * 2.0
    S.FLAME_SIZE_TIERS[tier] = NumberSequence.new({
        NumberSequenceKeypoint.new(0, baseSize),
        NumberSequenceKeypoint.new(0.3, baseSize * 0.6),
        NumberSequenceKeypoint.new(1, 0),
    })
    S.FLAME_SPEED_TIERS[tier] = NumberRange.new(5 + t * 15, 10 + t * 20)
end

-- Cached Color3 values for brake/turn lights (fix #6: no per-frame Color3 allocs)
S.BRAKE_ON = Color3.fromRGB(255, 20, 20)
S.BRAKE_OFF = Color3.fromRGB(80, 10, 10)
S.TURN_ON = Color3.fromRGB(255, 180, 30)
S.TURN_OFF = Color3.fromRGB(80, 60, 20)
S.FUEL_LOW_COLOR = Color3.fromRGB(255, 60, 60)
S.FUEL_MID_COLOR = Color3.fromRGB(255, 180, 50)

-- HUD
S.HUD_FADE_DELAY = 5
S.HUD_FONT = Enum.Font.GothamBold
S.HUD_BG_COLOR = Color3.fromRGB(15, 17, 25)
S.HUD_TEXT_COLOR = Color3.fromRGB(220, 225, 235)
S.HUD_ACCENT_COLOR = Color3.fromRGB(80, 180, 255)

--------------------------------------------------------------------------------
-- State
--------------------------------------------------------------------------------
S.mode = "none" -- "none" | "car" | "jetpack" | "parachute" | "wingsuit"

-- Grapple hook state (works across all modes)
S.grappleActive = false
S.grappleTargetPos = nil
S.grappleHitInstance = nil
S.grappleTimer = 0
S.grappleBodyVelocity = nil
S.grappleBeam = nil
S.grappleAttachChar = nil
S.grappleAttachTarget = nil
S.grappleAnchorPart = nil -- invisible anchor part at hit point for beam

-- Wingsuit state (auto-engages from sustained freefall)
S.wingsuitActive = false
S.wingsuitBodyVelocity = nil
S.wingsuitGyro = nil
S.wingsuitFreefallTimer = 0
S.wingsuitCurrentVel = Vector3.zero

-- Car state
S.carModel = nil
S.carBody = nil
S.carSeat = nil
S.carWheels = {} -- { part, motor, spring, steerHinge (front only) }
S.carBrakeLights = {}
S.carExhaustEmitter = nil
S.carEngineSound = nil
S.carTireScreechSound = nil
S.carHornSound = nil
S.carIdleVibration = nil
S.carSteerAngle = 0
S.carPrevSpeed = 0
S.carIsBraking = false
S.prevBraking = false
S.prevTurnState = 0 -- -1 left, 0 none, 1 right
S.carGyro = nil
S.carThrottleSmooth = 0
S.carBoostActive = false
S.carBoostTimer = 0
S.carBoostCooldown = 0
S.carSurfaceFriction = 1.0
S.carSurfaceFrictionTimer = 0

-- Jetpack state
S.jetpackForce = nil
S.jetpackEmitters = {}
S.jetpackLights = {}
S.jetpackThrustSound = nil
S.jetpackWindSound = nil
S.jetpackFuel = S.JETPACK_FUEL_MAX
S.jetpackThrustLevel = 0 -- 0..1 ramp
S.jetpackActive = false

-- Parachute state
S.chuteActive = false
S.chuteCanopy = nil
S.chuteForce = nil
S.chuteLift = nil
S.chuteGyro = nil
S.chuteHeading = 0
S.chuteStalled = false
S.chuteStallTimer = 0
S.chuteWindOffset = Vector3.new(0, 0, 0)
S.chuteWindSound = nil
S.chuteFlutterSound = nil
S.chuteLandedConn = nil
-- NOTE: `chuteDiveBlend` was an implicit global in the monolith (used without
-- declaration inside updateParachute). We retain that behavior by leaving it
-- unset here and letting ParachuteController read/write it through the same
-- field path on `S` — functionally equivalent because both refer to a single
-- shared nil-initialized slot.
S.chuteDiveBlend = nil

-- Camera state
S.customCamActive = false
S.camTargetFOV = nil -- set in init below (uses S.DEFAULT_FOV)
S.camCurrentPos = nil

-- G-force tracking
S.lastCarVelocity = Vector3.zero

-- Anti-fall-through safety net
S.lastSafePosition = Vector3.new(0, 100, 0)

-- Cinematic orbit camera
S.cinematicMode = false
S.cinematicAngle = 0

-- Transition state
S.transitionLock = false

-- Shared per-frame cached input
S.frameGamepad = nil
S.lastPublishedClientTelemetry = {}

S.camTargetFOV = S.DEFAULT_FOV

--------------------------------------------------------------------------------
-- Utility
--------------------------------------------------------------------------------
function S.tagPart(part, tag)
    CollectionService:AddTag(part, tag)
end

function S.cleanupByTag(tag)
    for _, obj in ipairs(CollectionService:GetTagged(tag)) do
        obj:Destroy()
    end
end

function S.lerp(a, b, t)
    return a + (b - a) * math.clamp(t, 0, 1)
end

function S.lerpVector3(a, b, t)
    return a:Lerp(b, math.clamp(t, 0, 1))
end

function S.getCamera()
    return Workspace.CurrentCamera
end

function S.tweenProperty(obj, props, duration, style, direction)
    style = style or Enum.EasingStyle.Quad
    direction = direction or Enum.EasingDirection.Out
    local tween = TweenService:Create(obj, TweenInfo.new(duration, style, direction), props)
    tween:Play()
    return tween
end

function S.getOrCreateDOF()
    local dof = Lighting:FindFirstChildOfClass("DepthOfFieldEffect")
    if not dof then
        dof = Instance.new("DepthOfFieldEffect")
        dof.Parent = Lighting
    end
    return dof
end

function S.enableCinematicDOF()
    local dof = S.getOrCreateDOF()
    dof.FarIntensity = 0.5
    dof.FocusDistance = 200
    dof.InFocusRadius = 100
    dof.NearIntensity = 0
    dof.Enabled = true
end

function S.enableParachuteDOF()
    local dof = S.getOrCreateDOF()
    dof.FarIntensity = 0.3
    dof.FocusDistance = 300
    dof.InFocusRadius = 200
    dof.NearIntensity = 0
    dof.Enabled = true
end

function S.disableDOF()
    local dof = Lighting:FindFirstChildOfClass("DepthOfFieldEffect")
    if dof then
        dof.Enabled = false
    end
end

function S.restoreDefaultCamera(humanoid)
    local camera = S.getCamera()
    if not camera then
        return
    end

    camera.CameraType = Enum.CameraType.Custom
    if humanoid then
        camera.CameraSubject = humanoid
    end
    S.tweenProperty(camera, { FieldOfView = S.DEFAULT_FOV }, 0.4)
end

function S.setPlayerAttributeIfChanged(name, nextValue)
    if player:GetAttribute(name) == nextValue then
        return
    end
    player:SetAttribute(name, nextValue)
end

function S.publishClientCameraTelemetry(humanoid)
    local camera = S.getCamera()
    local subject = camera and camera.CameraSubject or nil
    local telemetry = {
        ArnisClientCameraType = camera and tostring(camera.CameraType) or nil,
        ArnisClientCameraSubject = subject and subject:GetFullName() or nil,
        ArnisClientCameraSubjectClass = subject and subject.ClassName or nil,
        ArnisClientCameraMode = S.mode,
    }
    local telemetryChanged = false

    player:SetAttribute("ArnisVehicleControllerReady", true)

    for attributeName, nextValue in pairs(telemetry) do
        if S.lastPublishedClientTelemetry[attributeName] ~= nextValue then
            S.setPlayerAttributeIfChanged(attributeName, nextValue)
            S.lastPublishedClientTelemetry[attributeName] = nextValue
            telemetryChanged = true
        end
    end

    local humanoidState = humanoid and tostring(humanoid:GetState()) or nil
    if S.lastPublishedClientTelemetry.ArnisClientHumanoidState ~= humanoidState then
        local nextValue = humanoidState
        S.setPlayerAttributeIfChanged("ArnisClientHumanoidState", nextValue)
        S.lastPublishedClientTelemetry.ArnisClientHumanoidState = nextValue
        telemetryChanged = true
    end

    if telemetryChanged then
        print("ARNIS_CLIENT_CAMERA " .. HttpService:JSONEncode({
            mode = telemetry.ArnisClientCameraMode,
            cameraType = telemetry.ArnisClientCameraType,
            cameraSubject = telemetry.ArnisClientCameraSubject,
            cameraSubjectClass = telemetry.ArnisClientCameraSubjectClass,
            humanoidState = humanoidState,
        }))
    end
end

-- Sound assets from Roblox library
S.SOUND_ENGINE_LOOP = "rbxassetid://9112854440"
S.SOUND_TIRE_SCREECH = "rbxassetid://9114368685"
S.SOUND_HORN = "rbxassetid://9113651830"
S.SOUND_JET_THRUST = "rbxasset://sounds/action_falling.ogg"
S.SOUND_WIND_RUSH = "rbxasset://sounds/action_falling.ogg"
S.SOUND_CHUTE_DEPLOY = "rbxassetid://9113636898"
S.SOUND_CHUTE_FLUTTER = "rbxasset://sounds/action_falling.ogg"

function S.makeSound(parent, name, looped, volume, soundId)
    local s = Instance.new("Sound")
    s.Name = name
    s.Looped = looped or false
    s.Volume = volume or 0.5
    s.SoundId = soundId or ""
    s.Parent = parent
    return s
end

function S.getCharacter()
    return player.Character
end

function S.getHRP()
    local char = S.getCharacter()
    return char and char:FindFirstChild("HumanoidRootPart")
end

function S.getHumanoid()
    local char = S.getCharacter()
    return char and char:FindFirstChildOfClass("Humanoid")
end

function S.isOnGround()
    local hum = S.getHumanoid()
    if not hum then
        return false
    end
    local state = hum:GetState()
    return state == Enum.HumanoidStateType.Running or state == Enum.HumanoidStateType.Landed
end

-- Returns a table with gamepad axis/trigger values, or nil if no gamepad.
-- Fields: thumbstickX, thumbstickY, rightTrigger, leftTrigger
function S.readGamepad()
    local ok, state = pcall(function()
        return UserInputService:GetGamepadState(Enum.UserInputType.Gamepad1)
    end)
    if not ok or not state then
        return nil
    end

    local result = { thumbstickX = 0, thumbstickY = 0, rightTrigger = 0, leftTrigger = 0 }
    for _, input in ipairs(state) do
        if input.KeyCode == Enum.KeyCode.Thumbstick1 then
            result.thumbstickX = input.Position.X
            result.thumbstickY = input.Position.Y
        elseif input.KeyCode == Enum.KeyCode.ButtonR2 then
            result.rightTrigger = input.Position.Z
        elseif input.KeyCode == Enum.KeyCode.ButtonL2 then
            result.leftTrigger = input.Position.Z
        end
    end
    return result
end

--------------------------------------------------------------------------------
-- HUD
--------------------------------------------------------------------------------
local screenGui = Instance.new("ScreenGui")
screenGui.Name = "VehicleHUD"
screenGui.ResetOnSpawn = false
screenGui.IgnoreGuiInset = true
screenGui.DisplayOrder = 10
screenGui.Parent = playerGui
S.screenGui = screenGui

-- Main container at bottom center
local hudContainer = Instance.new("Frame")
hudContainer.Name = "HUDContainer"
hudContainer.Size = UDim2.new(0, 500, 0, 120)
hudContainer.Position = UDim2.new(0.5, -250, 1, -130)
hudContainer.BackgroundTransparency = 1
hudContainer.Parent = screenGui
S.hudContainer = hudContainer

-- Control hints
local controlHints = Instance.new("TextLabel")
controlHints.Name = "ControlHints"
controlHints.Size = UDim2.new(1, 0, 0, 24)
controlHints.Position = UDim2.new(0, 0, 1, -24)
controlHints.BackgroundTransparency = 0.4
controlHints.BackgroundColor3 = S.HUD_BG_COLOR
controlHints.TextColor3 = S.HUD_TEXT_COLOR
controlHints.Font = S.HUD_FONT
controlHints.TextSize = 13
controlHints.Text = "[V/Y] Car   [J/X] Jetpack   [P/A] Parachute"
controlHints.Parent = hudContainer
Instance.new("UICorner", controlHints).CornerRadius = UDim.new(0, 6)
S.controlHints = controlHints

-- Mode icon
local modeIcon = Instance.new("TextLabel")
modeIcon.Name = "ModeIcon"
modeIcon.Size = UDim2.new(0, 40, 0, 40)
modeIcon.Position = UDim2.new(0, 0, 0, 0)
modeIcon.BackgroundTransparency = 0.3
modeIcon.BackgroundColor3 = S.HUD_BG_COLOR
modeIcon.TextColor3 = S.HUD_ACCENT_COLOR
modeIcon.Font = S.HUD_FONT
modeIcon.TextSize = 22
modeIcon.Text = ""
modeIcon.Visible = false
modeIcon.Parent = hudContainer
Instance.new("UICorner", modeIcon).CornerRadius = UDim.new(0, 8)
S.modeIcon = modeIcon

-- Speedometer
local speedLabel = Instance.new("TextLabel")
speedLabel.Name = "Speed"
speedLabel.Size = UDim2.new(0, 120, 0, 36)
speedLabel.Position = UDim2.new(0.5, -60, 0, 0)
speedLabel.BackgroundTransparency = 0.3
speedLabel.BackgroundColor3 = S.HUD_BG_COLOR
speedLabel.TextColor3 = S.HUD_TEXT_COLOR
speedLabel.Font = S.HUD_FONT
speedLabel.TextSize = 20
speedLabel.Text = ""
speedLabel.Visible = false
speedLabel.Parent = hudContainer
Instance.new("UICorner", speedLabel).CornerRadius = UDim.new(0, 8)
S.speedLabel = speedLabel

-- Altitude
local altLabel = Instance.new("TextLabel")
altLabel.Name = "Altitude"
altLabel.Size = UDim2.new(0, 100, 0, 30)
altLabel.Position = UDim2.new(1, -100, 0, 0)
altLabel.BackgroundTransparency = 0.3
altLabel.BackgroundColor3 = S.HUD_BG_COLOR
altLabel.TextColor3 = S.HUD_TEXT_COLOR
altLabel.Font = S.HUD_FONT
altLabel.TextSize = 16
altLabel.Text = ""
altLabel.Visible = false
altLabel.Parent = hudContainer
Instance.new("UICorner", altLabel).CornerRadius = UDim.new(0, 6)
S.altLabel = altLabel

-- Fuel bar (jetpack)
local fuelBarBg = Instance.new("Frame")
fuelBarBg.Name = "FuelBarBG"
fuelBarBg.Size = UDim2.new(0, 160, 0, 12)
fuelBarBg.Position = UDim2.new(0.5, -80, 0, 44)
fuelBarBg.BackgroundTransparency = 0.3
fuelBarBg.BackgroundColor3 = S.HUD_BG_COLOR
fuelBarBg.Visible = false
fuelBarBg.Parent = hudContainer
Instance.new("UICorner", fuelBarBg).CornerRadius = UDim.new(0, 4)
S.fuelBarBg = fuelBarBg

local fuelBarFill = Instance.new("Frame")
fuelBarFill.Name = "FuelFill"
fuelBarFill.Size = UDim2.new(1, -4, 1, -4)
fuelBarFill.Position = UDim2.new(0, 2, 0, 2)
fuelBarFill.BackgroundColor3 = S.HUD_ACCENT_COLOR
fuelBarFill.BorderSizePixel = 0
fuelBarFill.Parent = fuelBarBg
Instance.new("UICorner", fuelBarFill).CornerRadius = UDim.new(0, 3)
S.fuelBarFill = fuelBarFill

S.controlHintTimer = 0
S.controlHintsVisible = true

function S.setHUDMode(newMode)
    local isCar = newMode == "car"
    local isJet = newMode == "jetpack"
    local isChute = newMode == "parachute"
    local isWing = newMode == "wingsuit"
    local isActive = isCar or isJet or isChute or isWing

    S.modeIcon.Visible = isActive
    S.speedLabel.Visible = isCar
    S.altLabel.Visible = isJet or isChute or isWing
    S.fuelBarBg.Visible = isJet

    if isCar then
        S.modeIcon.Text = "CAR"
        S.controlHints.Text = "[WASD] Drive   [Space] Brake   [H] Horn   [E/B] Exit   [G] Grapple"
    elseif isJet then
        S.modeIcon.Text = "JET"
        S.controlHints.Text = "[WASD/LS] Move   [Space/RT] Up   [Shift/LT] Down   [J/X] Off   [G] Grapple"
    elseif isChute then
        S.modeIcon.Text = "CHUTE"
        S.controlHints.Text = "[A/D/LS] Steer   [S] Flare   [P/B] Cut away   [G] Grapple"
    elseif isWing then
        S.modeIcon.Text = "WINGSUIT"
        S.controlHints.Text = "[A/D] Bank   [P] Chute   [J] Jetpack   [G] Grapple"
    else
        S.modeIcon.Text = ""
        S.controlHints.Text = "[V/Y] Car   [J/X] Jetpack   [P/A] Parachute   [G] Grapple"
    end

    -- Show hints, start fade timer
    S.controlHintTimer = S.HUD_FADE_DELAY
    if not S.controlHintsVisible then
        S.controlHintsVisible = true
        S.tweenProperty(S.controlHints, { TextTransparency = 0, BackgroundTransparency = 0.4 }, 0.3)
    end
end

function S.updateHUDValues(dt)
    -- Control hints fade
    if S.controlHintTimer > 0 then
        S.controlHintTimer = S.controlHintTimer - dt
        if S.controlHintTimer <= 0 and S.controlHintsVisible then
            S.controlHintsVisible = false
            S.tweenProperty(S.controlHints, { TextTransparency = 1, BackgroundTransparency = 1 }, 0.8)
        end
    end

    local hrp = S.getHRP()
    if not hrp then
        return
    end

    -- Speed (car)
    if S.mode == "car" and S.carBody then
        local speed = S.carBody.AssemblyLinearVelocity.Magnitude
        S.speedLabel.Text = string.format("%d km/h", math.floor(speed * 0.5))
    end

    -- Altitude
    if S.mode == "jetpack" or S.mode == "parachute" or S.mode == "wingsuit" then
        S.altLabel.Text = string.format("ALT %d", math.floor(hrp.Position.Y))
    end

    -- Fuel bar
    if S.mode == "jetpack" then
        local frac = math.clamp(S.jetpackFuel / S.JETPACK_FUEL_MAX, 0, 1)
        local barWidth = math.max(0, frac)
        S.fuelBarFill.Size = UDim2.new(barWidth, 0, 1, -4)
        if frac < 0.2 then
            S.fuelBarFill.BackgroundColor3 = S.FUEL_LOW_COLOR
        elseif frac < 0.5 then
            S.fuelBarFill.BackgroundColor3 = S.FUEL_MID_COLOR
        else
            S.fuelBarFill.BackgroundColor3 = S.HUD_ACCENT_COLOR
        end
    end
end

return S
