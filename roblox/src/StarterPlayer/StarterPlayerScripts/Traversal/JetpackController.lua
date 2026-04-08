--[[
    Traversal/JetpackController.lua

    Owns jetpack deploy/cleanup/update, physics forces, fuel bar,
    particle tiers, anti-tumble gyro, camera follow. Lifted verbatim
    from VehicleController.client.lua.
]]

local SharedState = require(script.Parent.SharedState)
local S = SharedState

local UserInputService = S.UserInputService
local Debris = S.Debris

local M = {}

function M.deployJetpack()
    local hrp = S.getHRP()
    local char = S.getCharacter()
    if not hrp or not char then
        return
    end

    S.jetpackActive = true
    S.jetpackThrustLevel = 0

    -- Backpack model
    local pack = Instance.new("Part")
    pack.Name = "JetpackBody"
    pack.Size = Vector3.new(3, 3.5, 1.5)
    pack.Material = Enum.Material.Metal
    pack.Color = Color3.fromRGB(60, 62, 68)
    pack.CFrame = hrp.CFrame * CFrame.new(0, 0, 1)
    pack.Anchored = false
    pack.CanCollide = false
    pack.Massless = true
    pack.Parent = char
    S.tagPart(pack, S.JETPACK_TAG)
    local packWeld = Instance.new("WeldConstraint")
    packWeld.Part0 = hrp
    packWeld.Part1 = pack
    packWeld.Parent = hrp

    -- Nozzles
    for _, offset in ipairs({ Vector3.new(-0.8, -1.5, 0.3), Vector3.new(0.8, -1.5, 0.3) }) do
        local nozzle = Instance.new("Part")
        nozzle.Name = "Nozzle"
        nozzle.Shape = Enum.PartType.Cylinder
        nozzle.Size = Vector3.new(1.2, 0.8, 0.8)
        nozzle.Material = Enum.Material.Metal
        nozzle.Color = Color3.fromRGB(45, 45, 50)
        nozzle.CFrame = pack.CFrame * CFrame.new(offset) * CFrame.Angles(math.rad(90), 0, 0)
        nozzle.Anchored = false
        nozzle.CanCollide = false
        nozzle.Massless = true
        nozzle.Parent = char
        S.tagPart(nozzle, S.JETPACK_TAG)
        local nw = Instance.new("WeldConstraint")
        nw.Part0 = pack
        nw.Part1 = nozzle
        nw.Parent = pack

        -- Flame particles per nozzle
        local attach = Instance.new("Attachment")
        attach.Position = Vector3.new(0, -0.5, 0)
        attach.Parent = nozzle

        local emitter = Instance.new("ParticleEmitter")
        emitter.Rate = 120
        emitter.Speed = NumberRange.new(12, 25)
        emitter.Lifetime = NumberRange.new(0.15, 0.4)
        emitter.Size = NumberSequence.new({
            NumberSequenceKeypoint.new(0, 1.8),
            NumberSequenceKeypoint.new(0.3, 1.0),
            NumberSequenceKeypoint.new(1, 0),
        })
        emitter.Color = ColorSequence.new({
            ColorSequenceKeypoint.new(0, Color3.fromRGB(130, 180, 255)), -- blue core
            ColorSequenceKeypoint.new(0.3, Color3.fromRGB(255, 200, 60)), -- orange mantle
            ColorSequenceKeypoint.new(0.7, Color3.fromRGB(255, 100, 20)), -- deep orange
            ColorSequenceKeypoint.new(1, Color3.fromRGB(60, 40, 20)), -- dark smoke tip
        })
        emitter.Transparency = NumberSequence.new({
            NumberSequenceKeypoint.new(0, 0),
            NumberSequenceKeypoint.new(0.6, 0.3),
            NumberSequenceKeypoint.new(1, 1),
        })
        emitter.LightEmission = 1
        emitter.LightInfluence = 0
        emitter.SpreadAngle = Vector2.new(8, 8)
        emitter.Parent = attach
        table.insert(S.jetpackEmitters, emitter)

        -- Heat shimmer (second emitter with high LightEmission)
        local shimmer = Instance.new("ParticleEmitter")
        shimmer.Rate = 40
        shimmer.Speed = NumberRange.new(5, 10)
        shimmer.Lifetime = NumberRange.new(0.3, 0.6)
        shimmer.Size = NumberSequence.new({
            NumberSequenceKeypoint.new(0, 3),
            NumberSequenceKeypoint.new(1, 5),
        })
        shimmer.Transparency = NumberSequence.new({
            NumberSequenceKeypoint.new(0, 0.85),
            NumberSequenceKeypoint.new(1, 1),
        })
        shimmer.LightEmission = 1
        shimmer.LightInfluence = 0
        shimmer.Color = ColorSequence.new(Color3.fromRGB(255, 240, 200))
        shimmer.Parent = attach

        -- Nozzle glow
        local glow = Instance.new("PointLight")
        glow.Range = 12
        glow.Brightness = 2
        glow.Color = Color3.fromRGB(255, 180, 60)
        glow.Parent = nozzle
        table.insert(S.jetpackLights, glow)
    end

    -- Trail
    local trailA0 = Instance.new("Attachment")
    trailA0.Position = Vector3.new(-1, -2, 1)
    trailA0.Parent = hrp
    S.tagPart(trailA0, S.JETPACK_TAG)

    local trailA1 = Instance.new("Attachment")
    trailA1.Position = Vector3.new(1, -2, 1)
    trailA1.Parent = hrp
    S.tagPart(trailA1, S.JETPACK_TAG)

    local trail = Instance.new("Trail")
    trail.Attachment0 = trailA0
    trail.Attachment1 = trailA1
    trail.Lifetime = 0.8
    trail.MinLength = 0.1
    trail.Color = ColorSequence.new({
        ColorSequenceKeypoint.new(0, Color3.fromRGB(255, 200, 80)),
        ColorSequenceKeypoint.new(1, Color3.fromRGB(100, 60, 20)),
    })
    trail.Transparency = NumberSequence.new({
        NumberSequenceKeypoint.new(0, 0.4),
        NumberSequenceKeypoint.new(1, 1),
    })
    trail.LightEmission = 0.5
    trail.FaceCamera = true
    trail.Parent = hrp

    -- Physics force
    S.jetpackForce = Instance.new("BodyForce")
    S.jetpackForce.Force = Vector3.new(0, 0, 0)
    S.jetpackForce.Parent = hrp
    S.tagPart(S.jetpackForce, S.JETPACK_TAG)

    -- Anti-tumble gyro: keeps character upright during flight
    local jetGyro = Instance.new("BodyGyro")
    jetGyro.Name = "JetpackGyro"
    jetGyro.MaxTorque = Vector3.new(40000, 2000, 40000)
    jetGyro.P = 6000
    jetGyro.D = 500
    jetGyro.CFrame = hrp.CFrame
    jetGyro.Parent = hrp
    S.tagPart(jetGyro, S.JETPACK_TAG)

    -- Sounds
    S.jetpackThrustSound = S.makeSound(hrp, "JetThrust", true, 0.3, S.SOUND_JET_THRUST)
    S.jetpackThrustSound:Play()
    S.tagPart(S.jetpackThrustSound, S.JETPACK_TAG)

    S.jetpackWindSound = S.makeSound(hrp, "JetWind", true, 0, S.SOUND_WIND_RUSH)
    S.jetpackWindSound:Play()
    S.tagPart(S.jetpackWindSound, S.JETPACK_TAG)

    -- Startup whoosh
    local startupSnd = S.makeSound(hrp, "JetStart", false, 0.5, S.SOUND_JET_THRUST)
    startupSnd:Play()
    Debris:AddItem(startupSnd, 2)

    S.mode = "jetpack"
    S.customCamActive = true
    S.setHUDMode("jetpack")
end

function M.cleanupJetpack()
    if not S.jetpackActive then
        return
    end
    S.jetpackActive = false

    -- Shutdown sound
    local hrp = S.getHRP()
    if hrp then
        local shutdownSnd = S.makeSound(hrp, "JetStop", false, 0.4, S.SOUND_JET_THRUST)
        shutdownSnd:Play()
        Debris:AddItem(shutdownSnd, 2)
    end

    -- Cleanup all tagged parts
    S.cleanupByTag(S.JETPACK_TAG)

    S.jetpackForce = nil
    S.jetpackEmitters = {}
    S.jetpackLights = {}
    S.jetpackThrustSound = nil
    S.jetpackWindSound = nil
    S.jetpackThrustLevel = 0

    if S.mode == "jetpack" then
        S.mode = "none"
        S.customCamActive = false
        S.camCurrentPos = nil -- fix #11: reset here, not in render loop
        S.restoreDefaultCamera(S.getHumanoid())
        S.setHUDMode("none")
    end
end

function M.updateJetpack(dt)
    if not S.jetpackActive or not S.jetpackForce then
        return
    end

    local hrp = S.getHRP()
    if not hrp then
        return
    end

    -- Fuel
    S.jetpackFuel = S.jetpackFuel - dt
    if S.jetpackFuel <= 0 then
        S.jetpackFuel = 0
        M.cleanupJetpack()
        return
    end

    -- Input
    local cam = workspace.CurrentCamera
    local look = cam.CFrame.LookVector
    local right = cam.CFrame.RightVector

    local thrustDir = Vector3.new(0, 0, 0)
    local isThrusting = false

    -- Keyboard
    if UserInputService:IsKeyDown(Enum.KeyCode.W) then
        thrustDir = thrustDir + Vector3.new(look.X, 0, look.Z).Unit
        isThrusting = true
    end
    if UserInputService:IsKeyDown(Enum.KeyCode.S) then
        thrustDir = thrustDir - Vector3.new(look.X, 0, look.Z).Unit
        isThrusting = true
    end
    if UserInputService:IsKeyDown(Enum.KeyCode.A) then
        thrustDir = thrustDir - Vector3.new(right.X, 0, right.Z).Unit
        isThrusting = true
    end
    if UserInputService:IsKeyDown(Enum.KeyCode.D) then
        thrustDir = thrustDir + Vector3.new(right.X, 0, right.Z).Unit
        isThrusting = true
    end

    -- Gamepad thumbstick (additive to keyboard) — fix #10: use cached frameGamepad
    local gp = S.frameGamepad
    if gp then
        local stickMag = math.sqrt(gp.thumbstickX ^ 2 + gp.thumbstickY ^ 2)
        if stickMag > 0.1 then
            local flatLook = Vector3.new(look.X, 0, look.Z)
            local flatRight = Vector3.new(right.X, 0, right.Z)
            -- Normalize only if non-zero to avoid NaN
            if flatLook.Magnitude > 0.001 then
                flatLook = flatLook.Unit
            end
            if flatRight.Magnitude > 0.001 then
                flatRight = flatRight.Unit
            end
            thrustDir = thrustDir + flatLook * gp.thumbstickY + flatRight * gp.thumbstickX
            isThrusting = true
        end
    end

    -- Clamp thrustDir to horizontal unit (after combining all sources)
    local thrustH = Vector3.new(thrustDir.X, 0, thrustDir.Z)
    if thrustH.Magnitude > 1 then
        thrustDir = thrustH.Unit
    else
        thrustDir = thrustH
    end

    local verticalThrust = 0
    if UserInputService:IsKeyDown(Enum.KeyCode.Space) then
        verticalThrust = 1
        isThrusting = true
    elseif UserInputService:IsKeyDown(Enum.KeyCode.LeftShift) then
        verticalThrust = -0.5
        isThrusting = true
    end

    -- Gamepad triggers for vertical (additive, clamped) — fix #10: reuse cached gp
    if gp then
        if gp.rightTrigger > 0.05 then
            verticalThrust = math.max(verticalThrust, gp.rightTrigger)
            isThrusting = true
        end
        if gp.leftTrigger > 0.05 then
            -- descend is -0.5 max; scale trigger to same range
            local descendVal = -0.5 * gp.leftTrigger
            verticalThrust = math.min(verticalThrust, descendVal)
            isThrusting = true
        end
    end

    -- Thrust ramp (gradual buildup over JETPACK_RAMP_TIME)
    if isThrusting then
        S.jetpackThrustLevel = math.min(1, S.jetpackThrustLevel + dt / S.JETPACK_RAMP_TIME)
    else
        S.jetpackThrustLevel = math.max(0, S.jetpackThrustLevel - dt / (S.JETPACK_RAMP_TIME * 2))
    end

    -- Calculate force
    local mass = hrp.AssemblyMass
    local gravityCompensation = Vector3.new(0, mass * workspace.Gravity, 0)

    local horizontalForce = Vector3.new(0, 0, 0)
    if thrustDir.Magnitude > 0.01 then
        horizontalForce = thrustDir.Unit * S.JETPACK_MAX_THRUST * S.jetpackThrustLevel
    end

    local verticalForce = Vector3.new(0, verticalThrust * S.JETPACK_MAX_THRUST * S.jetpackThrustLevel, 0)

    -- Hover when idle (counteract gravity + gentle oscillation)
    local hoverForce = Vector3.new(0, 0, 0)
    if not isThrusting then
        hoverForce = gravityCompensation + Vector3.new(0, math.sin(tick() * 2) * 15, 0)
    else
        hoverForce = gravityCompensation * 0.95 -- partial gravity compensation when thrusting
    end

    -- Air resistance / damping: much stronger to prevent infinite acceleration
    local vel = hrp.AssemblyLinearVelocity
    local dampingForce = -vel * mass * (1 - S.JETPACK_DAMPING)

    -- Speed cap: apply strong counter-force when exceeding max speed
    local horizVel = Vector3.new(vel.X, 0, vel.Z)
    local horizSpeed = horizVel.Magnitude
    if horizSpeed > S.JETPACK_MAX_HORIZONTAL_SPEED then
        local excess = horizSpeed - S.JETPACK_MAX_HORIZONTAL_SPEED
        dampingForce = dampingForce - horizVel.Unit * excess * mass * 3
    end
    if math.abs(vel.Y) > S.JETPACK_MAX_VERTICAL_SPEED then
        local excessY = math.abs(vel.Y) - S.JETPACK_MAX_VERTICAL_SPEED
        dampingForce = dampingForce - Vector3.new(0, math.sign(vel.Y) * excessY * mass * 3, 0)
    end

    S.jetpackForce.Force = horizontalForce + verticalForce + hoverForce + dampingForce

    -- Update anti-tumble gyro to face movement direction
    local jetGyro = hrp:FindFirstChild("JetpackGyro")
    if jetGyro then
        if horizSpeed > 3 then
            -- Face movement direction with slight forward tilt
            local moveDir = horizVel.Unit
            local tiltAngle = math.clamp(horizSpeed / S.JETPACK_MAX_HORIZONTAL_SPEED, 0, 1) * math.rad(15)
            jetGyro.CFrame = CFrame.lookAt(Vector3.zero, moveDir) * CFrame.Angles(tiltAngle, 0, 0)
        else
            jetGyro.CFrame = CFrame.new()
        end
    end

    -- Character tilt based on movement
    -- (Uses a subtle approach: adjust the force direction slightly)

    -- Particle intensity scales with thrust (fix #1: pre-computed tier lookup, zero allocs)
    local tier = math.floor(S.jetpackThrustLevel * 10 + 0.5)
    tier = math.clamp(tier, 0, 10)
    for _, em in ipairs(S.jetpackEmitters) do
        em.Size = S.FLAME_SIZE_TIERS[tier]
        em.Speed = S.FLAME_SPEED_TIERS[tier]
        em.Rate = 50 + S.jetpackThrustLevel * 150
    end

    -- Nozzle glow intensity
    for _, gl in ipairs(S.jetpackLights) do
        gl.Brightness = 0.5 + S.jetpackThrustLevel * 3
        gl.Range = 6 + S.jetpackThrustLevel * 10
    end

    -- Sound: pitch and volume scale with thrust
    if S.jetpackThrustSound then
        S.jetpackThrustSound.Volume = 0.15 + S.jetpackThrustLevel * 0.5
        S.jetpackThrustSound.PlaybackSpeed = 0.7 + S.jetpackThrustLevel * 0.8
    end

    -- Wind sound at high speed
    if S.jetpackWindSound then
        local speedFrac = math.clamp(vel.Magnitude / 100, 0, 1)
        S.jetpackWindSound.Volume = speedFrac * 0.4
        S.jetpackWindSound.PlaybackSpeed = 0.8 + speedFrac * 0.4
    end

    -- Camera
    if S.customCamActive then
        local camera = S.getCamera()
        if not camera then
            return
        end
        local targetPos = (hrp.CFrame * CFrame.new(0, S.JETPACK_CAM_OFFSET.Y, S.JETPACK_CAM_OFFSET.Z)).Position

        if S.camCurrentPos then
            S.camCurrentPos = S.lerpVector3(S.camCurrentPos, targetPos, math.min(1, S.CAM_LERP_RATE * dt))
        else
            S.camCurrentPos = targetPos
        end

        -- Slight shake at full thrust (fix #2: math.noise for smooth, deterministic shake)
        local shake = Vector3.zero
        if S.jetpackThrustLevel > 0.7 then
            local shakeAmt = (S.jetpackThrustLevel - 0.7) / 0.3 * S.JETPACK_CAM_SHAKE_INTENSITY
            local t = tick()
            shake = Vector3.new(math.noise(t * 12, 0) * shakeAmt, math.noise(t * 12, 100) * shakeAmt, 0)
        end

        camera.CameraType = Enum.CameraType.Scriptable
        camera.CFrame = CFrame.new(S.camCurrentPos + shake, hrp.Position + hrp.CFrame.LookVector * 10)

        -- Pull FOV back slightly (fix #7: dt-scaled lerp)
        local jetFov = S.DEFAULT_FOV + S.jetpackThrustLevel * 8
        camera.FieldOfView = S.lerp(camera.FieldOfView, jetFov, math.min(1, S.CAM_LERP_RATE * dt))
    end
end

return M
