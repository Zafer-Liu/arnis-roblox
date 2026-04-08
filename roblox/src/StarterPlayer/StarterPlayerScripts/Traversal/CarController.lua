--[[
    Traversal/CarController.lua

    Owns all car spawn/enter/exit/drive/physics/HUD/camera behavior.
    Lifted verbatim from VehicleController.client.lua. Function bodies
    are unchanged except for references to shared state, which now
    route through the shared state table `S` from SharedState.lua.
]]

local SharedState = require(script.Parent.SharedState)
local S = SharedState

local UserInputService = S.UserInputService
local TweenService = S.TweenService
local CollectionService = S.CollectionService

local M = {}

local function createCarBody(spawnCF)
    local model = Instance.new("Model")
    model.Name = "PlayerCar"

    -- Main chassis (lower body, heavy for stability)
    local chassis = Instance.new("Part")
    chassis.Name = "Chassis"
    chassis.Size = Vector3.new(7, 1.5, 15)
    chassis.Material = Enum.Material.SmoothPlastic
    chassis.Color = Color3.fromRGB(25, 25, 30)
    chassis.CFrame = spawnCF * CFrame.new(0, 1.2, 0)
    chassis.Anchored = false
    chassis.CustomPhysicalProperties = PhysicalProperties.new(8, 0.3, 0.1, 1, 1) -- heavy bottom
    chassis.Parent = model
    S.tagPart(chassis, S.CAR_TAG)

    -- Upper body shell
    local bodyLower = Instance.new("Part")
    bodyLower.Name = "BodyLower"
    bodyLower.Size = Vector3.new(7.4, 2.2, 15.2)
    bodyLower.Material = Enum.Material.SmoothPlastic
    bodyLower.Color = Color3.fromRGB(180, 28, 28)
    bodyLower.CFrame = spawnCF * CFrame.new(0, 2.8, 0)
    bodyLower.Anchored = false
    bodyLower.CanCollide = false
    bodyLower.Massless = true
    bodyLower.Parent = model
    S.tagPart(bodyLower, S.CAR_TAG)
    local bw1 = Instance.new("WeldConstraint")
    bw1.Part0 = chassis
    bw1.Part1 = bodyLower
    bw1.Parent = chassis

    -- Hood (sloped front)
    local hood = Instance.new("Part")
    hood.Name = "Hood"
    hood.Size = Vector3.new(6.8, 1.0, 5)
    hood.Material = Enum.Material.SmoothPlastic
    hood.Color = Color3.fromRGB(175, 25, 25)
    hood.CFrame = spawnCF * CFrame.new(0, 3.6, -5.5) * CFrame.Angles(math.rad(-12), 0, 0)
    hood.Anchored = false
    hood.CanCollide = false
    hood.Massless = true
    hood.Parent = model
    S.tagPart(hood, S.CAR_TAG)
    local hw = Instance.new("WeldConstraint")
    hw.Part0 = chassis
    hw.Part1 = hood
    hw.Parent = chassis

    -- Trunk (slightly sloped rear)
    local trunk = Instance.new("Part")
    trunk.Name = "Trunk"
    trunk.Size = Vector3.new(6.8, 0.8, 4)
    trunk.Material = Enum.Material.SmoothPlastic
    trunk.Color = Color3.fromRGB(175, 25, 25)
    trunk.CFrame = spawnCF * CFrame.new(0, 3.6, 5) * CFrame.Angles(math.rad(6), 0, 0)
    trunk.Anchored = false
    trunk.CanCollide = false
    trunk.Massless = true
    trunk.Parent = model
    S.tagPart(trunk, S.CAR_TAG)
    local tw = Instance.new("WeldConstraint")
    tw.Part0 = chassis
    tw.Part1 = trunk
    tw.Parent = chassis

    -- Roof / cabin
    local cabin = Instance.new("Part")
    cabin.Name = "Cabin"
    cabin.Size = Vector3.new(6.6, 2.2, 6)
    cabin.Material = Enum.Material.SmoothPlastic
    cabin.Color = Color3.fromRGB(165, 22, 22)
    cabin.CFrame = spawnCF * CFrame.new(0, 5, 0.5)
    cabin.Anchored = false
    cabin.CanCollide = false
    cabin.Massless = true
    cabin.Parent = model
    S.tagPart(cabin, S.CAR_TAG)
    local cw = Instance.new("WeldConstraint")
    cw.Part0 = chassis
    cw.Part1 = cabin
    cw.Parent = chassis

    -- Windshield (glass, transparent, angled)
    local windshield = Instance.new("Part")
    windshield.Name = "Windshield"
    windshield.Size = Vector3.new(6.2, 2.4, 0.2)
    windshield.Material = Enum.Material.Glass
    windshield.Color = Color3.fromRGB(180, 210, 235)
    windshield.Transparency = 0.6
    windshield.CFrame = spawnCF * CFrame.new(0, 5, -2.6) * CFrame.Angles(math.rad(-20), 0, 0)
    windshield.Anchored = false
    windshield.CanCollide = false
    windshield.Massless = true
    windshield.Parent = model
    S.tagPart(windshield, S.CAR_TAG)
    local wsw = Instance.new("WeldConstraint")
    wsw.Part0 = chassis
    wsw.Part1 = windshield
    wsw.Parent = chassis

    -- Rear windshield
    local rearGlass = Instance.new("Part")
    rearGlass.Name = "RearGlass"
    rearGlass.Size = Vector3.new(6.2, 2.0, 0.2)
    rearGlass.Material = Enum.Material.Glass
    rearGlass.Color = Color3.fromRGB(170, 200, 225)
    rearGlass.Transparency = 0.65
    rearGlass.CFrame = spawnCF * CFrame.new(0, 5, 3.6) * CFrame.Angles(math.rad(15), 0, 0)
    rearGlass.Anchored = false
    rearGlass.CanCollide = false
    rearGlass.Massless = true
    rearGlass.Parent = model
    S.tagPart(rearGlass, S.CAR_TAG)
    local rgw = Instance.new("WeldConstraint")
    rgw.Part0 = chassis
    rgw.Part1 = rearGlass
    rgw.Parent = chassis

    -- Dashboard (visible through windshield)
    local dashboard = Instance.new("Part")
    dashboard.Name = "Dashboard"
    dashboard.Size = Vector3.new(5.8, 0.6, 2)
    dashboard.Material = Enum.Material.SmoothPlastic
    dashboard.Color = Color3.fromRGB(40, 40, 45)
    dashboard.CFrame = spawnCF * CFrame.new(0, 3.8, -1.5)
    dashboard.Anchored = false
    dashboard.CanCollide = false
    dashboard.Massless = true
    dashboard.Parent = model
    S.tagPart(dashboard, S.CAR_TAG)
    local dw = Instance.new("WeldConstraint")
    dw.Part0 = chassis
    dw.Part1 = dashboard
    dw.Parent = chassis

    -- VehicleSeat
    local seat = Instance.new("VehicleSeat")
    seat.Name = "DriveSeat"
    seat.Size = Vector3.new(4, 0.5, 3)
    seat.CFrame = spawnCF * CFrame.new(0, 2.5, 0.5)
    seat.Anchored = false
    seat.CanCollide = false
    seat.Massless = true
    seat.MaxSpeed = S.CAR_MAX_SPEED
    seat.Torque = 0 -- we drive motors manually
    seat.TurnSpeed = 0
    seat.Parent = model
    S.tagPart(seat, S.CAR_TAG)
    local sw = Instance.new("WeldConstraint")
    sw.Part0 = chassis
    sw.Part1 = seat
    sw.Parent = chassis

    -- Anti-flip gyro: keeps car upright with mild yaw damping to prevent spin-outs
    local gyro = Instance.new("BodyGyro")
    gyro.MaxTorque = Vector3.new(50000, 5000, 50000) -- strong roll/pitch, mild yaw stabilization
    gyro.P = 10000
    gyro.D = 800
    gyro.CFrame = chassis.CFrame
    gyro.Parent = chassis

    -- Engine idle vibration (subtle, not enough to move the car)
    local idleVib = Instance.new("BodyPosition")
    idleVib.MaxForce = Vector3.new(0, 20, 0)
    idleVib.P = 3000
    idleVib.D = 300
    idleVib.Position = chassis.Position
    idleVib.Parent = chassis

    -- Headlights
    for _, side in ipairs({ -2.5, 2.5 }) do
        local light = Instance.new("Part")
        light.Name = "Headlight"
        light.Shape = Enum.PartType.Ball
        light.Size = Vector3.new(1.2, 1.2, 1.2)
        light.Material = Enum.Material.Neon
        light.Color = Color3.fromRGB(255, 250, 230)
        light.CFrame = spawnCF * CFrame.new(side, 2.8, -7.8)
        light.Anchored = false
        light.CanCollide = false
        light.Massless = true
        light.Parent = model
        S.tagPart(light, S.CAR_TAG)
        local lw = Instance.new("WeldConstraint")
        lw.Part0 = chassis
        lw.Part1 = light
        lw.Parent = chassis

        local spot = Instance.new("SpotLight")
        spot.Range = 80
        spot.Brightness = 3
        spot.Angle = 50
        spot.Face = Enum.NormalId.Front
        spot.Color = Color3.fromRGB(255, 248, 225)
        spot.Parent = light
    end

    -- Brake / tail lights
    local brakeLights = {}
    for _, side in ipairs({ -2.8, 2.8 }) do
        local tail = Instance.new("Part")
        tail.Name = "BrakeLight"
        tail.Size = Vector3.new(1.6, 0.9, 0.3)
        tail.Material = Enum.Material.Neon
        tail.Color = Color3.fromRGB(80, 10, 10) -- dim by default
        tail.CFrame = spawnCF * CFrame.new(side, 2.8, 7.8)
        tail.Anchored = false
        tail.CanCollide = false
        tail.Massless = true
        tail.Parent = model
        S.tagPart(tail, S.CAR_TAG)
        local tlw = Instance.new("WeldConstraint")
        tlw.Part0 = chassis
        tlw.Part1 = tail
        tlw.Parent = chassis
        table.insert(brakeLights, tail)
    end

    -- Turn signal lights
    for _, data in ipairs({
        { side = -3.6, name = "TurnL" },
        { side = 3.6, name = "TurnR" },
    }) do
        local sig = Instance.new("Part")
        sig.Name = data.name
        sig.Size = Vector3.new(0.4, 0.6, 0.3)
        sig.Material = Enum.Material.Neon
        sig.Color = Color3.fromRGB(60, 40, 5) -- dim amber
        sig.CFrame = spawnCF * CFrame.new(data.side, 2.8, -7.5)
        sig.Anchored = false
        sig.CanCollide = false
        sig.Massless = true
        sig.Parent = model
        S.tagPart(sig, S.CAR_TAG)
        local sigw = Instance.new("WeldConstraint")
        sigw.Part0 = chassis
        sigw.Part1 = sig
        sigw.Parent = chassis
    end

    -- Exhaust particles
    local exhaustAttach = Instance.new("Attachment")
    exhaustAttach.Position = Vector3.new(2, 0.5, 7.8)
    exhaustAttach.Parent = chassis

    local exhaust = Instance.new("ParticleEmitter")
    exhaust.Rate = 15
    exhaust.Speed = NumberRange.new(2, 5)
    exhaust.Lifetime = NumberRange.new(0.4, 1.0)
    exhaust.Size = NumberSequence.new({
        NumberSequenceKeypoint.new(0, 0.3),
        NumberSequenceKeypoint.new(1, 1.2),
    })
    exhaust.Color = ColorSequence.new({
        ColorSequenceKeypoint.new(0, Color3.fromRGB(120, 120, 130)),
        ColorSequenceKeypoint.new(1, Color3.fromRGB(80, 80, 85)),
    })
    exhaust.Transparency = NumberSequence.new({
        NumberSequenceKeypoint.new(0, 0.5),
        NumberSequenceKeypoint.new(1, 1),
    })
    exhaust.LightEmission = 0
    exhaust.SpreadAngle = Vector2.new(10, 10)
    exhaust.Parent = exhaustAttach

    -- Sounds
    local engineSnd = S.makeSound(chassis, "Engine", true, 0.4, S.SOUND_ENGINE_LOOP)
    local screechSnd = S.makeSound(chassis, "TireScreech", false, 0.3, S.SOUND_TIRE_SCREECH)
    local hornSnd = S.makeSound(chassis, "Horn", false, 0.6, S.SOUND_HORN)

    model.PrimaryPart = chassis

    return {
        model = model,
        chassis = chassis,
        seat = seat,
        gyro = gyro,
        idleVib = idleVib,
        brakeLights = brakeLights,
        exhaust = exhaust,
        engineSound = engineSnd,
        screechSound = screechSnd,
        hornSound = hornSnd,
    }
end

local function createWheelWithSuspension(model, chassis, spawnCF, offset, isFront)
    -- Wheel axle (invisible anchor for suspension)
    local axle = Instance.new("Part")
    axle.Name = "Axle_" .. tostring(offset)
    axle.Size = Vector3.new(0.5, 0.5, 0.5)
    axle.Transparency = 1
    axle.CanCollide = false
    axle.Massless = true
    axle.CFrame = spawnCF * CFrame.new(offset)
    axle.Anchored = false
    axle.Parent = model
    S.tagPart(axle, S.CAR_TAG)

    -- Wheel part
    local wheel = Instance.new("Part")
    wheel.Name = "Wheel"
    wheel.Shape = Enum.PartType.Cylinder
    wheel.Size = Vector3.new(2.2, 2.8, 2.8)
    wheel.Material = Enum.Material.SmoothPlastic
    wheel.Color = Color3.fromRGB(30, 30, 35)
    wheel.CFrame = spawnCF * CFrame.new(offset) * CFrame.Angles(0, 0, math.pi / 2)
    wheel.Anchored = false
    wheel.CustomPhysicalProperties = PhysicalProperties.new(
        isFront and 1.5 or 1.5,
        isFront and 1.2 or 0.8, -- rear wheels: slightly less friction for controlled drift, not ice
        0.2,
        1,
        1
    )
    wheel.Parent = model
    S.tagPart(wheel, S.CAR_TAG)

    -- Hub cap (visual detail)
    local hub = Instance.new("Part")
    hub.Name = "Hub"
    hub.Shape = Enum.PartType.Cylinder
    hub.Size = Vector3.new(0.3, 2.0, 2.0)
    hub.Material = Enum.Material.Metal
    hub.Color = Color3.fromRGB(160, 165, 175)
    hub.CFrame = wheel.CFrame
    hub.Anchored = false
    hub.CanCollide = false
    hub.Massless = true
    hub.Parent = model
    S.tagPart(hub, S.CAR_TAG)
    local hubWeld = Instance.new("WeldConstraint")
    hubWeld.Part0 = wheel
    hubWeld.Part1 = hub
    hubWeld.Parent = wheel

    -- Suspension: SpringConstraint between chassis and axle
    local chassisAttach = Instance.new("Attachment")
    chassisAttach.Position = offset + Vector3.new(0, 0.5, 0)
    chassisAttach.Parent = chassis

    local axleAttach = Instance.new("Attachment")
    axleAttach.Position = Vector3.new(0, 0, 0)
    axleAttach.Parent = axle

    local spring = Instance.new("SpringConstraint")
    spring.Attachment0 = chassisAttach
    spring.Attachment1 = axleAttach
    spring.FreeLength = S.SUSPENSION_REST_LENGTH
    spring.Stiffness = S.SUSPENSION_STIFFNESS
    spring.Damping = S.SUSPENSION_DAMPING
    spring.LimitsEnabled = true
    spring.MinLength = 0.5
    spring.MaxLength = 2.5
    spring.Visible = false
    spring.Parent = chassis

    -- Prismatic constraint to keep wheel under the chassis (vertical only)
    local prismatic = Instance.new("PrismaticConstraint")
    prismatic.Attachment0 = chassisAttach
    prismatic.Attachment1 = axleAttach
    prismatic.LimitsEnabled = true
    prismatic.LowerLimit = -1.5
    prismatic.UpperLimit = 0.5
    prismatic.Parent = chassis

    -- Motor: CylindricalConstraint for spinning
    local motorAttach0 = Instance.new("Attachment")
    motorAttach0.Parent = axle

    local motorAttach1 = Instance.new("Attachment")
    motorAttach1.Parent = wheel

    local motor = Instance.new("CylindricalConstraint")
    motor.Attachment0 = motorAttach0
    motor.Attachment1 = motorAttach1
    motor.MotorType = Enum.ActuatorType.Motor
    motor.AngularVelocity = 0
    motor.MotorMaxTorque = S.CAR_TORQUE
    motor.MotorMaxAngularAcceleration = 200
    motor.InclinationAngle = 90
    motor.RotationAxisVisible = false
    motor.Parent = axle

    -- Steering hinge (front wheels only)
    local steerHinge = nil
    if isFront then
        local hingeA0 = Instance.new("Attachment")
        hingeA0.Parent = chassis
        hingeA0.CFrame = CFrame.new(offset)

        local hingeA1 = Instance.new("Attachment")
        hingeA1.Parent = axle

        steerHinge = Instance.new("HingeConstraint")
        steerHinge.Attachment0 = hingeA0
        steerHinge.Attachment1 = hingeA1
        steerHinge.ActuatorType = Enum.ActuatorType.Servo
        steerHinge.TargetAngle = 0
        steerHinge.AngularSpeed = math.rad(120)
        steerHinge.ServoMaxTorque = 20000
        steerHinge.LimitsEnabled = true
        steerHinge.LowerAngle = -S.CAR_STEER_ANGLE
        steerHinge.UpperAngle = S.CAR_STEER_ANGLE
        steerHinge.Parent = chassis
    end

    return {
        part = wheel,
        axle = axle,
        motor = motor,
        spring = spring,
        steerHinge = steerHinge,
        isFront = isFront,
    }
end

function M.spawnCar()
    local hrp = S.getHRP()
    if not hrp then
        return
    end

    local spawnPos = hrp.Position + hrp.CFrame.LookVector * 14
    local spawnCF = CFrame.new(spawnPos)

    local carData = createCarBody(spawnCF)
    local mdl = carData.model
    local chassis = carData.chassis

    -- Create wheels with suspension
    local wheelOffsets = {
        { offset = Vector3.new(-3.5, -0.5, -5.5), front = true },
        { offset = Vector3.new(3.5, -0.5, -5.5), front = true },
        { offset = Vector3.new(-3.5, -0.5, 5.5), front = false },
        { offset = Vector3.new(3.5, -0.5, 5.5), front = false },
    }

    local wheels = {}
    for _, wd in ipairs(wheelOffsets) do
        local w = createWheelWithSuspension(mdl, chassis, spawnCF, wd.offset, wd.front)
        table.insert(wheels, w)
    end

    mdl.Parent = workspace

    S.carModel = mdl
    S.carBody = chassis
    S.carSeat = carData.seat
    S.carWheels = wheels
    S.carBrakeLights = carData.brakeLights
    S.carExhaustEmitter = carData.exhaust
    S.carEngineSound = carData.engineSound
    S.carTireScreechSound = carData.screechSound
    S.carHornSound = carData.hornSound
    S.carGyro = carData.gyro
    S.carIdleVibration = carData.idleVib
    S.carSteerAngle = 0
    S.carPrevSpeed = 0
    S.carIsBraking = false

    -- Start engine sound
    S.carEngineSound:Play()

    return carData.seat
end

function M.destroyCar()
    if S.carModel then
        -- Fade out engine
        if S.carEngineSound and S.carEngineSound.IsPlaying then
            S.tweenProperty(S.carEngineSound, { Volume = 0 }, 0.3)
            task.delay(0.35, function()
                if S.carModel then
                    S.carModel:Destroy()
                end
            end)
        else
            S.carModel:Destroy()
        end
    end

    S.carModel = nil
    S.carBody = nil
    S.carSeat = nil
    S.carWheels = {}
    S.carBrakeLights = {}
    S.carExhaustEmitter = nil
    S.carEngineSound = nil
    S.carTireScreechSound = nil
    S.carHornSound = nil
    S.carGyro = nil
    S.carIdleVibration = nil
    S.cleanupByTag(S.CAR_TAG)
end

function M.enterCar(seat)
    local hum = S.getHumanoid()
    if not hum or not seat then
        return
    end

    -- Fix #13: let the engine handle seat entry — no CFrame tween
    seat:Sit(hum)
    S.mode = "car"
    S.customCamActive = true
    S.setHUDMode("car")
end

function M.exitCar()
    local hum = S.getHumanoid()
    if hum then
        hum.Sit = false
        -- Small upward impulse on exit
        local hrp = S.getHRP()
        if hrp then
            task.defer(function()
                hrp.AssemblyLinearVelocity = hrp.AssemblyLinearVelocity + Vector3.new(0, 10, 0)
            end)
        end
    end
    S.mode = "none"
    S.customCamActive = false
    S.camCurrentPos = nil -- fix #11: reset here, not in render loop
    S.prevBraking = false
    S.prevTurnState = 0
    S.carThrottleSmooth = 0
    S.carBoostActive = false
    S.carBoostTimer = 0
    S.carBoostCooldown = 0
    S.restoreDefaultCamera(hum)
    S.setHUDMode("none")
end

function M.updateCar(dt)
    if S.mode ~= "car" or not S.carBody or not S.carSeat then
        return
    end

    local throttle = S.carSeat.ThrottleFloat -- -1 to 1 from VehicleSeat
    local steer = S.carSeat.SteerFloat -- -1 to 1

    local velocity = S.carBody.AssemblyLinearVelocity
    local speed = velocity.Magnitude

    -- Per-road-surface friction is handled automatically by Roblox physics:
    -- RoadBuilder applies CustomPhysicalProperties per OSM surface tag (asphalt
    -- 0.75, gravel 0.38, ice 0.12, etc.) so wheel grip is realistic without
    -- runtime raycasts. The carSurfaceFriction multiplier below is reserved
    -- for future driving-feel adjustments (e.g. reduced top speed on dirt).
    S.carSurfaceFriction = 1.0

    -- Speed-dependent steering: reduce max angle at high speed to prevent spinouts
    local speedFraction = math.clamp(speed / S.CAR_MAX_SPEED, 0, 1)
    local steerReduction = 1 - speedFraction * (1 - S.CAR_HIGH_SPEED_STEER_FACTOR)
    local targetSteer = steer * S.CAR_STEER_ANGLE * steerReduction
    S.carSteerAngle = S.lerp(S.carSteerAngle, targetSteer, dt * S.CAR_STEER_SPEED)

    -- Handbrake (space)
    local handbrake = UserInputService:IsKeyDown(Enum.KeyCode.Space)

    -- Rapier-style jerk-limited acceleration: smooth ramp instead of instant torque
    local rampTime = if math.abs(throttle) > math.abs(S.carThrottleSmooth or 0) then S.CAR_ACCEL_RAMP_TIME else S.CAR_DECEL_RAMP_TIME
    S.carThrottleSmooth = S.lerp(S.carThrottleSmooth or 0, throttle, dt / math.max(rampTime, 0.01))

    -- Nitro boost (LShift while driving)
    local wantBoost = UserInputService:IsKeyDown(Enum.KeyCode.LeftShift) and math.abs(S.carThrottleSmooth) > 0.5
    if wantBoost and (S.carBoostCooldown or 0) <= 0 then
        S.carBoostActive = true
        S.carBoostTimer = (S.carBoostTimer or 0) + dt
        if S.carBoostTimer >= S.CAR_BOOST_DURATION then
            S.carBoostActive = false
            S.carBoostCooldown = S.CAR_BOOST_COOLDOWN
            S.carBoostTimer = 0
        end
    else
        S.carBoostActive = false
        S.carBoostTimer = 0
        S.carBoostCooldown = math.max((S.carBoostCooldown or 0) - dt, 0)
    end
    local effectiveAngVel = if S.carBoostActive then S.CAR_MOTOR_ANGULAR_VEL * (S.CAR_BOOST_SPEED / S.CAR_MAX_SPEED) else S.CAR_MOTOR_ANGULAR_VEL
    -- Surface friction modulates available torque: low friction = wheels slip,
    -- effective forward force is reduced. This creates the realistic feel of
    -- driving on grass/dirt vs asphalt.
    local effectiveTorque = if S.carBoostActive then S.CAR_TORQUE * 1.8 else S.CAR_TORQUE
    effectiveTorque = effectiveTorque * S.carSurfaceFriction

    -- Update wheels
    for _, w in ipairs(S.carWheels) do
        local motorSpeed = S.carThrottleSmooth * effectiveAngVel
        if handbrake and not w.isFront then
            -- Lock rear wheels for drift (Far Cry style)
            w.motor.AngularVelocity = 0
            w.motor.MotorMaxTorque = effectiveTorque * 5
        else
            w.motor.AngularVelocity = motorSpeed
            w.motor.MotorMaxTorque = effectiveTorque
        end

        -- Steering (front wheels)
        if w.steerHinge then
            w.steerHinge.TargetAngle = S.carSteerAngle
        end
    end

    -- Brake lights: activate on deceleration or handbrake (fix #6: only write on change)
    local decelerating = speed > 2 and (speed < S.carPrevSpeed - 0.5 or throttle < -0.1)
    S.carIsBraking = decelerating or handbrake
    if S.carIsBraking ~= S.prevBraking then
        S.prevBraking = S.carIsBraking
        local brakeColor = S.carIsBraking and S.BRAKE_ON or S.BRAKE_OFF
        for _, bl in ipairs(S.carBrakeLights) do
            bl.Color = brakeColor
        end
    end

    -- Turn signals (fix #6: only write on change, use cached Color3)
    local turnState = (steer < -0.5 and -1) or (steer > 0.5 and 1) or 0
    if turnState ~= S.prevTurnState and S.carModel then
        S.prevTurnState = turnState
        local turnL = S.carModel:FindFirstChild("TurnL")
        local turnR = S.carModel:FindFirstChild("TurnR")
        if turnL then
            turnL.Color = (turnState == -1) and S.TURN_ON or S.TURN_OFF
        end
        if turnR then
            turnR.Color = (turnState == 1) and S.TURN_ON or S.TURN_OFF
        end
    end

    -- Exhaust: more particles when accelerating
    if S.carExhaustEmitter then
        S.carExhaustEmitter.Rate = math.abs(throttle) > 0.1 and 40 or 12
    end

    -- Engine sound pitch scales with speed
    if S.carEngineSound then
        S.carEngineSound.PlaybackSpeed = 0.8 + (speed / S.CAR_MAX_SPEED) * 1.2
        S.carEngineSound.Volume = 0.3 + (speed / S.CAR_MAX_SPEED) * 0.4
    end

    -- Tire screech on hard turns at speed or handbrake (fix #9: fade instead of hard stop)
    if S.carTireScreechSound then
        local shouldScreech = (math.abs(steer) > 0.7 and speed > 30) or (handbrake and speed > 15)
        if shouldScreech then
            if not S.carTireScreechSound.IsPlaying then
                S.carTireScreechSound.Volume = 0.3
                S.carTireScreechSound:Play()
            end
        elseif S.carTireScreechSound.IsPlaying then
            S.carTireScreechSound.Volume = S.lerp(S.carTireScreechSound.Volume, 0, math.min(1, 10 * dt))
            if S.carTireScreechSound.Volume < 0.01 then
                S.carTireScreechSound:Stop()
            end
        end
    end

    -- Anti-flip gyro: keep upright
    if S.carGyro then
        S.carGyro.CFrame = CFrame.new(S.carBody.Position)
            * CFrame.Angles(0, select(2, S.carBody.CFrame:ToEulerAnglesYXZ()), 0)
    end

    -- Engine idle vibration
    if S.carIdleVibration then
        local vibAmt = S.ENGINE_IDLE_VIBRATION * (1 + speed * 0.005)
        S.carIdleVibration.Position = S.carBody.Position + Vector3.new(0, math.sin(tick() * 30) * vibAmt, 0)
    end

    -- G-force camera shake: measure velocity delta since last frame
    local velocityDelta = (S.carBody.AssemblyLinearVelocity - S.lastCarVelocity).Magnitude
    local accelShake = math.clamp(velocityDelta * 0.001, 0, 0.02)
    S.lastCarVelocity = S.carBody.AssemblyLinearVelocity

    S.carPrevSpeed = speed

    -- Chase camera
    if S.customCamActive then
        local camera = S.getCamera()
        if not camera then
            return
        end
        local carCF = S.carBody.CFrame
        local targetPos = (carCF * CFrame.new(
            -steer * 2, -- slight offset into turn
            S.CAR_CAM_OFFSET.Y,
            S.CAR_CAM_OFFSET.Z
        )).Position

        if S.camCurrentPos then
            S.camCurrentPos = S.lerpVector3(S.camCurrentPos, targetPos, math.min(1, S.CAM_LERP_RATE * dt))
        else
            S.camCurrentPos = targetPos
        end

        -- Tilt into turns
        local tiltAngle = -steer * S.CAR_CAM_TILT_FACTOR
        local lookTarget = carCF.Position + carCF.LookVector * 20

        -- G-force shake offset (noise-based, barely perceptible)
        local t = tick()
        local shakeX = (math.noise(t * 10, 0) * 2 - 1) * accelShake
        local shakeY = (math.noise(t * 10 + 100, 0) * 2 - 1) * accelShake
        local shakeOffset = Vector3.new(shakeX, shakeY, 0)

        camera.CameraType = Enum.CameraType.Scriptable
        camera.CFrame = CFrame.new(S.camCurrentPos + shakeOffset, lookTarget) * CFrame.Angles(0, 0, tiltAngle)

        -- Speed-based FOV (fix #7: dt-scaled lerp)
        local fovCap = if S.carBoostActive then S.CAR_FOV_BOOST else S.CAR_FOV_MAX
        local fovTarget = S.CAR_FOV_MIN + (speed / S.CAR_FOV_SPEED_RANGE) * (fovCap - S.CAR_FOV_MIN)
        S.camTargetFOV = math.clamp(fovTarget, S.CAR_FOV_MIN, fovCap)
        camera.FieldOfView = S.lerp(camera.FieldOfView, S.camTargetFOV, math.min(1, S.CAM_LERP_RATE * dt))
    end
end

return M
