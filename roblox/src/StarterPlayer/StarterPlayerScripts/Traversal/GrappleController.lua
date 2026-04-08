--[[
    Traversal/GrappleController.lua

    Owns the Just Cause style grapple hook: raycast from camera,
    BodyVelocity-based pull, beam rendering, anchor, timeout, and
    arrival-radius release. Lifted verbatim from
    VehicleController.client.lua.
]]

local SharedState = require(script.Parent.SharedState)
local S = SharedState

local Workspace = S.Workspace

local M = {}

function M.releaseGrapple()
    if not S.grappleActive then
        return
    end
    S.grappleActive = false
    S.grappleTargetPos = nil
    S.grappleHitInstance = nil
    S.grappleTimer = 0

    if S.grappleBodyVelocity then
        S.grappleBodyVelocity:Destroy()
        S.grappleBodyVelocity = nil
    end
    if S.grappleBeam then
        S.grappleBeam:Destroy()
        S.grappleBeam = nil
    end
    if S.grappleAttachChar then
        S.grappleAttachChar:Destroy()
        S.grappleAttachChar = nil
    end
    if S.grappleAttachTarget then
        S.grappleAttachTarget:Destroy()
        S.grappleAttachTarget = nil
    end
    if S.grappleAnchorPart then
        S.grappleAnchorPart:Destroy()
        S.grappleAnchorPart = nil
    end
end

function M.fireGrapple()
    if S.grappleActive then
        -- toggle: second press cancels
        M.releaseGrapple()
        return
    end

    local hrp = S.getHRP()
    local camera = S.getCamera()
    if not hrp or not camera then
        return
    end

    -- Raycast from camera along lookat
    local origin = camera.CFrame.Position
    local direction = camera.CFrame.LookVector * S.GRAPPLE_MAX_RANGE

    local params = RaycastParams.new()
    params.FilterType = Enum.RaycastFilterType.Exclude
    local filter = {}
    local char = S.getCharacter()
    if char then
        table.insert(filter, char)
    end
    if S.carModel then
        table.insert(filter, S.carModel)
    end
    params.FilterDescendantsInstances = filter
    params.IgnoreWater = false

    local result = Workspace:Raycast(origin, direction, params)
    if not result then
        return
    end

    S.grappleActive = true
    S.grappleTargetPos = result.Position
    S.grappleHitInstance = result.Instance
    S.grappleTimer = 0

    -- Smooth BodyVelocity-based pull with damped MaxForce (no instant snap)
    local bv = Instance.new("BodyVelocity")
    bv.Name = "GrappleBodyVelocity"
    bv.MaxForce = Vector3.new(S.GRAPPLE_PULL_MAX_FORCE, S.GRAPPLE_PULL_MAX_FORCE, S.GRAPPLE_PULL_MAX_FORCE)
    bv.P = 3000
    bv.Velocity = hrp.AssemblyLinearVelocity -- start from current vel to avoid snap
    bv.Parent = hrp
    S.grappleBodyVelocity = bv

    -- Anchor an invisible, anchored part at the hit position so the beam stays put
    -- even if the hit instance moves or is destroyed during the pull.
    local anchor = Instance.new("Part")
    anchor.Name = "GrappleAnchor"
    anchor.Size = Vector3.new(0.1, 0.1, 0.1)
    anchor.Transparency = 1
    anchor.CanCollide = false
    anchor.Anchored = true
    anchor.CFrame = CFrame.new(S.grappleTargetPos)
    anchor.Parent = Workspace
    S.grappleAnchorPart = anchor

    local a0 = Instance.new("Attachment")
    a0.Name = "GrappleA0"
    a0.Parent = hrp
    S.grappleAttachChar = a0

    local a1 = Instance.new("Attachment")
    a1.Name = "GrappleA1"
    a1.Parent = anchor
    S.grappleAttachTarget = a1

    local beam = Instance.new("Beam")
    beam.Attachment0 = a0
    beam.Attachment1 = a1
    beam.Width0 = S.GRAPPLE_BEAM_WIDTH
    beam.Width1 = S.GRAPPLE_BEAM_WIDTH
    beam.Color = ColorSequence.new(Color3.fromRGB(30, 30, 30))
    beam.LightEmission = 0.1
    beam.FaceCamera = true
    beam.Segments = 6
    beam.Parent = hrp
    S.grappleBeam = beam
end

function M.updateGrapple(dt)
    if not S.grappleActive or not S.grappleTargetPos or not S.grappleBodyVelocity then
        return
    end

    local hrp = S.getHRP()
    if not hrp then
        M.releaseGrapple()
        return
    end

    S.grappleTimer = S.grappleTimer + dt
    if S.grappleTimer >= S.GRAPPLE_TIMEOUT then
        M.releaseGrapple()
        return
    end

    local toTarget = S.grappleTargetPos - hrp.Position
    local distance = toTarget.Magnitude
    if distance <= S.GRAPPLE_ARRIVAL_RADIUS then
        M.releaseGrapple()
        return
    end

    -- Smooth velocity lerp toward pull direction (no snaps)
    local desiredDir = toTarget.Unit
    local desiredVel = desiredDir * S.GRAPPLE_PULL_SPEED
    local current = S.grappleBodyVelocity.Velocity
    local t = math.clamp(S.GRAPPLE_VELOCITY_LERP * dt, 0, 1)
    S.grappleBodyVelocity.Velocity = current:Lerp(desiredVel, t)
end

return M
