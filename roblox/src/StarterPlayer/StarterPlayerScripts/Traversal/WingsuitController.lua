--[[
    Traversal/WingsuitController.lua

    Owns the auto-engaging wingsuit mode: freefall detection,
    BodyVelocity glide, pitch-forward gyro pose, bank from WASD,
    and exit-on-land. Lifted verbatim from VehicleController.client.lua.
]]

local SharedState = require(script.Parent.SharedState)
local S = SharedState

local UserInputService = S.UserInputService

local M = {}

function M.exitWingsuit()
    if not S.wingsuitActive then
        return
    end
    S.wingsuitActive = false
    S.wingsuitFreefallTimer = 0
    S.wingsuitCurrentVel = Vector3.zero

    if S.wingsuitBodyVelocity then
        S.wingsuitBodyVelocity:Destroy()
        S.wingsuitBodyVelocity = nil
    end
    if S.wingsuitGyro then
        S.wingsuitGyro:Destroy()
        S.wingsuitGyro = nil
    end

    if S.mode == "wingsuit" then
        S.mode = "none"
        S.customCamActive = false
        S.camCurrentPos = nil
        S.restoreDefaultCamera(S.getHumanoid())
        S.setHUDMode("none")
    end
end

function M.enterWingsuit()
    if S.wingsuitActive then
        return
    end

    local hrp = S.getHRP()
    if not hrp then
        return
    end

    S.wingsuitActive = true

    -- BodyVelocity: damped horizontal + vertical for smooth glide (no snap)
    local bv = Instance.new("BodyVelocity")
    bv.Name = "WingsuitBodyVelocity"
    bv.MaxForce = Vector3.new(S.WINGSUIT_MAX_FORCE_H, S.WINGSUIT_MAX_FORCE_V, S.WINGSUIT_MAX_FORCE_H)
    bv.P = 1500 -- soft P for smooth velocity tracking
    bv.Velocity = hrp.AssemblyLinearVelocity -- start from current velocity, no snap
    bv.Parent = hrp
    S.wingsuitBodyVelocity = bv
    S.wingsuitCurrentVel = bv.Velocity

    -- BodyGyro: tilts character into glide pose with stable damping
    local gyro = Instance.new("BodyGyro")
    gyro.Name = "WingsuitGyro"
    gyro.MaxTorque = Vector3.new(20000, 20000, 20000)
    gyro.P = 2500
    gyro.D = 250
    gyro.CFrame = hrp.CFrame
    gyro.Parent = hrp
    S.wingsuitGyro = gyro

    S.mode = "wingsuit"
    S.customCamActive = false -- let default camera follow; wingsuit is a lightweight mode
    S.setHUDMode("wingsuit")
end

function M.updateWingsuit(dt)
    -- Auto-detection of freefall (only when in a passive mode)
    if S.mode == "none" and not S.wingsuitActive and not S.chuteActive and not S.jetpackActive then
        local hum = S.getHumanoid()
        local hrp = S.getHRP()
        if hum and hrp then
            local state = hum:GetState()
            local isFalling = state == Enum.HumanoidStateType.Freefall
                or state == Enum.HumanoidStateType.FallingDown
            if isFalling then
                S.wingsuitFreefallTimer = S.wingsuitFreefallTimer + dt
                if S.wingsuitFreefallTimer >= S.WINGSUIT_MIN_FREEFALL_TIME then
                    M.enterWingsuit()
                end
            else
                S.wingsuitFreefallTimer = 0
            end
        end
    elseif S.mode ~= "wingsuit" then
        S.wingsuitFreefallTimer = 0
    end

    if not S.wingsuitActive or not S.wingsuitBodyVelocity then
        return
    end

    local hrp = S.getHRP()
    if not hrp then
        M.exitWingsuit()
        return
    end

    -- If we've landed, drop out of wingsuit
    local hum = S.getHumanoid()
    if hum then
        local state = hum:GetState()
        if state == Enum.HumanoidStateType.Running
            or state == Enum.HumanoidStateType.Landed
            or state == Enum.HumanoidStateType.Climbing
        then
            M.exitWingsuit()
            return
        end
    end

    -- Steer from camera lookat (horizontal projection), WASD for subtle correction
    local camera = S.getCamera()
    if not camera then
        return
    end
    local look = camera.CFrame.LookVector
    local flatLook = Vector3.new(look.X, 0, look.Z)
    if flatLook.Magnitude < 0.001 then
        return
    end
    flatLook = flatLook.Unit

    local right = camera.CFrame.RightVector
    local flatRight = Vector3.new(right.X, 0, right.Z)
    if flatRight.Magnitude > 0.001 then
        flatRight = flatRight.Unit
    end

    -- Compute desired velocity from glide physics
    -- 4:1 glide ratio: forward 90 studs/s, descent -22 studs/s
    local desiredHoriz = flatLook * S.WINGSUIT_FORWARD_SPEED
    -- Respect glide ratio math (forward/descent ratio should equal WINGSUIT_GLIDE_RATIO)
    local desiredVert = S.WINGSUIT_DESCENT_RATE
    -- Ensure forward matches declared ratio (keeps constants coherent if later tuned)
    local ratioCorrected = math.abs(desiredVert) * S.WINGSUIT_GLIDE_RATIO
    desiredHoriz = flatLook * ratioCorrected

    -- Subtle WASD bank/nudge (A/D) without snapping direction
    local bank = 0
    if UserInputService:IsKeyDown(Enum.KeyCode.A) then
        bank = bank - 1
    end
    if UserInputService:IsKeyDown(Enum.KeyCode.D) then
        bank = bank + 1
    end
    if bank ~= 0 then
        desiredHoriz = desiredHoriz + flatRight * bank * (ratioCorrected * 0.15)
    end

    local desired = Vector3.new(desiredHoriz.X, desiredVert, desiredHoriz.Z)

    -- Smooth lerp from current to desired velocity (no snaps)
    local t = math.clamp(S.WINGSUIT_TILT_LERP * dt, 0, 1)
    S.wingsuitCurrentVel = S.wingsuitCurrentVel:Lerp(desired, t)
    S.wingsuitBodyVelocity.Velocity = S.wingsuitCurrentVel

    -- Smooth pitch-forward gyro (glide pose) aligned with flight direction
    if S.wingsuitGyro then
        local pitch = math.rad(-S.WINGSUIT_PITCH_DEGREES)
        local targetCFrame = CFrame.lookAt(hrp.Position, hrp.Position + flatLook)
            * CFrame.Angles(pitch, 0, math.rad(bank * 18))
        -- BodyGyro naturally damps via P/D; passing the target CFrame is already smooth.
        S.wingsuitGyro.CFrame = targetCFrame
    end
end

return M
