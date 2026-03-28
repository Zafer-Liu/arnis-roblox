return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Assert = require(script.Parent.Assert)
    local WorldProbeGeometry = require(ReplicatedStorage.Shared.WorldProbeGeometry)

    local probeRoot = Vector3.new(0, 0, 0)

    local shell = Instance.new("Part")
    shell.Name = "ShellWallProbePart"
    shell.Anchored = true
    shell.Size = Vector3.new(120, 24, 24)
    shell.CFrame = CFrame.new(220, 12, 0)
    shell.Parent = workspace

    local withinRadius, nearestDistanceStuds = WorldProbeGeometry.isNearbyShellWall(shell, probeRoot, 180)

    Assert.equal(withinRadius, true, "expected shell wall proximity to use surface distance instead of part centroid")
    Assert.truthy(nearestDistanceStuds ~= nil, "expected nearby shell wall distance")
    Assert.truthy(
        nearestDistanceStuds <= 180,
        "expected nearby shell wall distance to reflect the closest surface within radius"
    )

    shell:Destroy()
end
