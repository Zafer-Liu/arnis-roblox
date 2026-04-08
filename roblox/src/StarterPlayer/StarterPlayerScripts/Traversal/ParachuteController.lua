--[[
    Traversal/ParachuteController.lua

    Owns the canopy construction, deploy/retract, pendulum physics,
    steering, dive/flare mechanics, stall detection, HUD indicator,
    and camera follow. Lifted verbatim from VehicleController.client.lua.
]]

local SharedState = require(script.Parent.SharedState)
local S = SharedState

local UserInputService = S.UserInputService
local CollectionService = S.CollectionService
local Debris = S.Debris

local M = {}

function M.deployParachute()
    local hrp = S.getHRP()
    local hum = S.getHumanoid()
    local char = S.getCharacter()
    if not hrp or not hum or not char then
        return
    end

    -- Only deploy when falling fast enough
    if hrp.AssemblyLinearVelocity.Y > -5 then
        -- Flash HUD red to show deploy failed
        local originalColor = S.controlHints.TextColor3
        S.controlHints.TextColor3 = Color3.fromRGB(255, 80, 80)
        S.controlHints.Text = "Need more altitude to deploy!"
        S.controlHintTimer = S.HUD_FADE_DELAY
        if not S.controlHintsVisible then
            S.controlHintsVisible = true
            S.tweenProperty(S.controlHints, { TextTransparency = 0, BackgroundTransparency = 0.4 }, 0.3)
        end
        task.delay(1.5, function()
            S.controlHints.TextColor3 = originalColor
            S.setHUDMode(S.mode)
        end)
        return
    end

    S.chuteActive = true
    S.chuteStalled = false
    S.chuteStallTimer = 0
    S.chuteHeading = select(2, hrp.CFrame:ToEulerAnglesYXZ())

    -- Random wind offset
    S.chuteWindOffset =
        Vector3.new((math.random() - 0.5) * S.CHUTE_WIND_STRENGTH * 2, 0, (math.random() - 0.5) * S.CHUTE_WIND_STRENGTH * 2)

    -- Rectangular canopy from multiple panels
    local canopyRoot = Instance.new("Part")
    canopyRoot.Name = "CanopyRoot"
    canopyRoot.Size = Vector3.new(2, 2, 2) -- some volume for physics presence
    canopyRoot.Transparency = 1
    canopyRoot.CanCollide = false
    canopyRoot.Massless = false -- real mass enables pendulum swing
    canopyRoot.CustomPhysicalProperties = PhysicalProperties.new(0.05, 0, 0, 0, 0)
    canopyRoot.Anchored = false
    canopyRoot.CFrame = hrp.CFrame * CFrame.new(0, 18, 0)
    canopyRoot.Parent = char
    S.tagPart(canopyRoot, S.PARACHUTE_TAG)

    -- Weld canopy root to follow player via BallSocketConstraint (allows swing)
    local rootAttachHRP = Instance.new("Attachment")
    rootAttachHRP.Position = Vector3.new(0, 2, 0)
    rootAttachHRP.Parent = hrp
    S.tagPart(rootAttachHRP, S.PARACHUTE_TAG)

    local rootAttachCanopy = Instance.new("Attachment")
    rootAttachCanopy.Position = Vector3.new(0, -8, 0) -- bottom of the "rope"
    rootAttachCanopy.Parent = canopyRoot
    S.tagPart(rootAttachCanopy, S.PARACHUTE_TAG)

    -- BallSocketConstraint: keeps canopy tethered at correct distance, allows swing
    local mainSocket = Instance.new("BallSocketConstraint")
    mainSocket.Attachment0 = rootAttachHRP
    mainSocket.Attachment1 = rootAttachCanopy
    mainSocket.LimitsEnabled = true
    mainSocket.UpperAngle = 25 -- max pendulum swing angle (degrees)
    mainSocket.Restitution = 0 -- no bounce
    mainSocket.Visible = false
    mainSocket.Parent = canopyRoot
    S.tagPart(mainSocket, S.PARACHUTE_TAG)

    -- Keep canopy floating above player: counteract its own weight plus a small upward bias
    local canopyMass = canopyRoot:GetMass()
    local canopyLift = Instance.new("BodyForce")
    canopyLift.Name = "CanopyLift"
    canopyLift.Force = Vector3.new(0, workspace.Gravity * canopyMass * 1.15, 0) -- 1.15x gravity for gentle upward pull
    canopyLift.Parent = canopyRoot
    S.tagPart(canopyLift, S.PARACHUTE_TAG)

    S.chuteCanopy = canopyRoot

    -- Canopy panels (rectangular, alternating orange and white)
    local panelCount = 7
    local panelWidth = 3.5
    local panelHeight = 0.4
    local panelDepth = 8

    for i = 1, panelCount do
        local panel = Instance.new("Part")
        panel.Name = "Panel" .. i
        panel.Size = Vector3.new(panelWidth - 0.1, panelHeight, panelDepth)
        panel.Material = Enum.Material.Fabric

        -- White canopy with red center panel
        if i == math.ceil(panelCount / 2) then
            panel.Color = Color3.fromRGB(200, 30, 30) -- red center panel
        else
            panel.Color = Color3.fromRGB(245, 245, 245) -- clean white
        end

        local xOff = (i - (panelCount + 1) / 2) * panelWidth
        panel.CFrame = canopyRoot.CFrame * CFrame.new(xOff, 0, 0)
        panel.Anchored = false
        panel.CanCollide = false
        panel.Massless = true
        panel.Parent = char
        S.tagPart(panel, S.PARACHUTE_TAG)

        local pw = Instance.new("WeldConstraint")
        pw.Part0 = canopyRoot
        pw.Part1 = panel
        pw.Parent = canopyRoot

        -- Lines from each panel edge to player
        for _, lineXOff in ipairs({ -panelWidth / 2 + 0.3, panelWidth / 2 - 0.3 }) do
            local lineAttachTop = Instance.new("Attachment")
            lineAttachTop.Position = Vector3.new(xOff + lineXOff, -panelHeight / 2, 0)
            lineAttachTop.Parent = canopyRoot

            local lineAttachBot = Instance.new("Attachment")
            lineAttachBot.Position = Vector3.new(
                (xOff + lineXOff) * 0.15, -- converge toward center at player
                2,
                0
            )
            lineAttachBot.Parent = hrp
            S.tagPart(lineAttachBot, S.PARACHUTE_TAG)

            local beam = Instance.new("Beam")
            beam.Attachment0 = lineAttachTop
            beam.Attachment1 = lineAttachBot
            beam.Width0 = 0.05
            beam.Width1 = 0.05
            beam.Color = ColorSequence.new(Color3.fromRGB(180, 180, 180))
            beam.FaceCamera = true
            beam.Parent = canopyRoot
        end
    end

    -- Physics: BodyForce for lift, BodyVelocity for descent rate limiting
    S.chuteForce = Instance.new("BodyForce")
    S.chuteForce.Force = Vector3.new(0, 0, 0)
    S.chuteForce.Parent = hrp
    S.tagPart(S.chuteForce, S.PARACHUTE_TAG)

    S.chuteLift = Instance.new("BodyVelocity")
    S.chuteLift.MaxForce = Vector3.new(4000, 20000, 4000) -- also damp horizontal to prevent wild swings
    S.chuteLift.Velocity = Vector3.new(0, S.CHUTE_DESCENT_RATE, 0)
    S.chuteLift.P = 2000 -- softer P for smoother velocity tracking
    S.chuteLift.Parent = hrp
    S.tagPart(S.chuteLift, S.PARACHUTE_TAG)

    -- Gyro for controlled heading
    S.chuteGyro = Instance.new("BodyGyro")
    S.chuteGyro.MaxTorque = Vector3.new(10000, 10000, 10000)
    S.chuteGyro.P = 3000
    S.chuteGyro.D = 200
    S.chuteGyro.Parent = hrp
    S.tagPart(S.chuteGyro, S.PARACHUTE_TAG)

    -- Sounds
    S.chuteWindSound = S.makeSound(hrp, "ChuteWind", true, 0.3, S.SOUND_WIND_RUSH)
    S.chuteWindSound:Play()
    S.tagPart(S.chuteWindSound, S.PARACHUTE_TAG)

    S.chuteFlutterSound = S.makeSound(hrp, "ChuteFlutter", true, 0.15, S.SOUND_CHUTE_FLUTTER)
    S.chuteFlutterSound:Play()
    S.tagPart(S.chuteFlutterSound, S.PARACHUTE_TAG)

    -- Deploy whoosh
    local deploySnd = S.makeSound(hrp, "ChuteDeploy", false, 0.5, S.SOUND_CHUTE_DEPLOY)
    deploySnd:Play()
    Debris:AddItem(deploySnd, 2)

    -- Auto-retract on landing
    S.chuteLandedConn = hum.StateChanged:Connect(function(_, newState)
        if newState == Enum.HumanoidStateType.Landed or newState == Enum.HumanoidStateType.Running then
            M.retractParachute()
        end
    end)

    S.enableParachuteDOF()

    S.mode = "parachute"
    S.customCamActive = true
    S.setHUDMode("parachute")
end

function M.retractParachute()
    if not S.chuteActive then
        return
    end
    S.chuteActive = false

    if S.chuteLandedConn then
        S.chuteLandedConn:Disconnect()
        S.chuteLandedConn = nil
    end

    -- Fix #5: destroy physics forces IMMEDIATELY (no lingering drag/lift)
    if S.chuteForce then
        S.chuteForce:Destroy()
    end
    if S.chuteLift then
        S.chuteLift:Destroy()
    end
    if S.chuteGyro then
        S.chuteGyro:Destroy()
    end
    S.chuteForce = nil
    S.chuteLift = nil
    S.chuteGyro = nil

    -- Visual parts fade out over 1 second (fix #14: guard nil via IsA check)
    local canopyParts = CollectionService:GetTagged(S.PARACHUTE_TAG)
    for _, obj in ipairs(canopyParts) do
        if obj and obj:IsA("BasePart") and obj.Name ~= "HumanoidRootPart" then
            S.tweenProperty(obj, { Transparency = 1 }, 1.0)
        end
    end

    task.delay(1.1, function()
        S.cleanupByTag(S.PARACHUTE_TAG)
    end)

    S.chuteCanopy = nil
    S.chuteWindSound = nil
    S.chuteFlutterSound = nil

    if S.mode == "parachute" then
        S.disableDOF()
        S.mode = "none"
        S.customCamActive = false
        S.camCurrentPos = nil -- fix #11: reset here, not in render loop
        S.restoreDefaultCamera(S.getHumanoid())
        S.setHUDMode("none")
    end
end

function M.updateParachute(dt)
    if not S.chuteActive or not S.chuteForce or not S.chuteLift then
        return
    end

    local hrp = S.getHRP()
    if not hrp then
        return
    end

    local vel = hrp.AssemblyLinearVelocity
    local mass = hrp.AssemblyMass

    -- Steering input
    local steerInput = 0
    local flareInput = 0

    if UserInputService:IsKeyDown(Enum.KeyCode.A) then
        steerInput = -1
    elseif UserInputService:IsKeyDown(Enum.KeyCode.D) then
        steerInput = 1
    end

    if UserInputService:IsKeyDown(Enum.KeyCode.S) then
        flareInput = 1
    end

    -- Dive input: hold W to tuck and gain speed (Just Cause style)
    local diveInput = UserInputService:IsKeyDown(Enum.KeyCode.W) and 1 or 0

    -- Gamepad thumbstick X for parachute steering (additive, clamped to -1..1) — fix #10
    local gpChute = S.frameGamepad
    if gpChute and math.abs(gpChute.thumbstickX) > 0.1 then
        steerInput = math.clamp(steerInput + gpChute.thumbstickX, -1, 1)
    end

    -- Update heading
    S.chuteHeading = S.chuteHeading + steerInput * S.CHUTE_TURN_RATE * dt

    -- Stall detection: sustained flare causes canopy collapse
    if S.chuteStalled then
        S.chuteStallTimer = S.chuteStallTimer + dt
        if S.chuteStallTimer > 1.5 then
            -- Re-inflate after stall recovery period
            S.chuteStalled = false
            S.chuteStallTimer = 0
        end
    else
        -- Accumulate flare time; reset when not flaring
        if flareInput > 0 then
            S.chuteStallTimer = S.chuteStallTimer + dt
        else
            S.chuteStallTimer = math.max(0, S.chuteStallTimer - dt * 0.5) -- slowly recover
        end
        -- Trigger stall after sustained flare
        if S.chuteStallTimer > S.CHUTE_FLARE_STALL_TIME then
            S.chuteStalled = true
            S.chuteStallTimer = 0
        end
    end

    -- Calculate forces
    local headingDir = Vector3.new(math.sin(S.chuteHeading), 0, math.cos(S.chuteHeading))

    local descentRate = S.CHUTE_DESCENT_RATE
    local forwardSpeed = S.CHUTE_FORWARD_SPEED

    if S.chuteStalled then
        -- Canopy collapsed: rapid descent, minimal forward
        descentRate = S.CHUTE_STALL_DESCENT
        forwardSpeed = S.CHUTE_FORWARD_SPEED * 0.2
    elseif diveInput > 0 and flareInput == 0 then
        -- Dive: trade altitude for speed (Just Cause wingsuit feel)
        -- Smooth transition into dive using recovery time
        S.chuteDiveBlend = math.min((S.chuteDiveBlend or 0) + dt / S.CHUTE_DIVE_RECOVERY_TIME, 1)
        descentRate = S.lerp(S.CHUTE_DESCENT_RATE, S.CHUTE_DIVE_DESCENT_RATE, S.chuteDiveBlend)
        forwardSpeed = S.lerp(S.CHUTE_FORWARD_SPEED, S.CHUTE_DIVE_FORWARD_SPEED, S.chuteDiveBlend)
    elseif flareInput > 0 then
        -- Flare: slow descent, reduce forward speed
        S.chuteDiveBlend = 0
        descentRate = S.CHUTE_DESCENT_RATE + S.CHUTE_FLARE_LIFT * flareInput
        forwardSpeed = S.CHUTE_FORWARD_SPEED * (1 - flareInput * 0.4)
    else
        -- Normal glide: smoothly recover from dive
        S.chuteDiveBlend = math.max((S.chuteDiveBlend or 0) - dt / S.CHUTE_DIVE_RECOVERY_TIME, 0)
        if S.chuteDiveBlend > 0 then
            descentRate = S.lerp(S.CHUTE_DESCENT_RATE, S.CHUTE_DIVE_DESCENT_RATE, S.chuteDiveBlend)
            forwardSpeed = S.lerp(S.CHUTE_FORWARD_SPEED, S.CHUTE_DIVE_FORWARD_SPEED, S.chuteDiveBlend)
        end
    end

    -- BodyVelocity controls descent rate and provides a gentle horizontal target
    local targetHorizVel = headingDir * forwardSpeed * 0.3
    S.chuteLift.Velocity = Vector3.new(targetHorizVel.X, descentRate, targetHorizVel.Z)

    -- BodyForce for forward glide + wind
    local forwardForce = headingDir * forwardSpeed * mass
    -- Gentle, perlin-noise-modulated wind instead of constant random push
    local windT = tick() * 0.3
    local windDynamic = Vector3.new(
        math.noise(windT, 0) * S.CHUTE_WIND_STRENGTH,
        0,
        math.noise(windT, 100) * S.CHUTE_WIND_STRENGTH
    ) * mass
    -- Drag: oppose horizontal velocity proportional to speed (stronger = more stable glide)
    local horizVel = Vector3.new(vel.X, 0, vel.Z)
    local dragForce = -horizVel * mass * S.CHUTE_HORIZONTAL_DRAG

    S.chuteForce.Force = forwardForce + windDynamic + dragForce

    -- Bank angle (gyro)
    if S.chuteGyro then
        local bankAngle = steerInput * math.rad(15)
        S.chuteGyro.CFrame = CFrame.new(hrp.Position)
            * CFrame.Angles(0, S.chuteHeading, bankAngle)
            * CFrame.Angles(math.rad(-10 - flareInput * 15), 0, 0) -- slight forward lean, more on flare
    end

    -- Pendulum swing: apply a small lateral impulse opposite to turn direction
    -- so the canopy visibly swings when the player steers.
    if S.chuteCanopy then
        if math.abs(steerInput) > 0.1 then
            -- Push canopy laterally against the turn (rope swings outward)
            local rightVec = CFrame.Angles(0, S.chuteHeading, 0).RightVector
            local swingForce = rightVec * steerInput * -60 * dt
            -- Apply as a small velocity nudge (mass is 0.05 kg)
            S.chuteCanopy.AssemblyLinearVelocity = S.chuteCanopy.AssemblyLinearVelocity + swingForce
        end
    end

    -- Sound
    if S.chuteWindSound then
        local speedFrac = math.clamp(vel.Magnitude / 60, 0, 1)
        S.chuteWindSound.Volume = 0.15 + speedFrac * 0.35
        S.chuteWindSound.PlaybackSpeed = 0.8 + speedFrac * 0.4
    end

    if S.chuteFlutterSound then
        S.chuteFlutterSound.Volume = S.chuteStalled and 0.4 or 0.15
    end

    -- Camera: wide FOV, above and behind, looking down slightly
    if S.customCamActive then
        local camera = S.getCamera()
        if not camera then
            return
        end
        local behindOffset = -headingDir * S.CHUTE_CAM_OFFSET.Z + Vector3.new(0, S.CHUTE_CAM_OFFSET.Y, 0)
        local targetPos = hrp.Position + behindOffset

        if S.camCurrentPos then
            S.camCurrentPos = S.lerpVector3(S.camCurrentPos, targetPos, math.min(1, S.CAM_LERP_RATE * dt))
        else
            S.camCurrentPos = targetPos
        end

        camera.CameraType = Enum.CameraType.Scriptable
        camera.CFrame = CFrame.new(S.camCurrentPos, hrp.Position + Vector3.new(0, -3, 0))
        camera.FieldOfView = S.lerp(camera.FieldOfView, S.CHUTE_FOV, math.min(1, S.CAM_LERP_RATE * dt))
    end
end

return M
