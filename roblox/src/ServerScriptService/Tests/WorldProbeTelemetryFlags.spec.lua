return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Workspace = game:GetService("Workspace")
    local Assert = require(script.Parent.Assert)
    local WorldProbeTelemetryFlags = require(ReplicatedStorage.Shared.WorldProbeTelemetryFlags)

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, nil)

    local defaultFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR))
    Assert.equal(WorldProbeTelemetryFlags.isEnabled(defaultFlags, "terrain"), false, "expected terrain telemetry to be off by default")
    Assert.equal(
        WorldProbeTelemetryFlags.isEnabled(defaultFlags, "player_local"),
        false,
        "expected player-local telemetry to be off by default"
    )
    Assert.equal(#defaultFlags.enabledFamilies, 0, "expected default telemetry flags to stay compact")

    local requestedFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(" player_local,terrain,roads,roads,unknown,water,hotspots ")
    Assert.equal(requestedFlags.enabledFamilies[1], "terrain", "expected canonical telemetry ordering to start with terrain")
    Assert.equal(requestedFlags.enabledFamilies[2], "roads", "expected canonical telemetry ordering to include roads second")
    Assert.equal(requestedFlags.enabledFamilies[3], "water", "expected canonical telemetry ordering to include water third")
    Assert.equal(
        requestedFlags.enabledFamilies[4],
        "hotspots",
        "expected canonical telemetry ordering to include hotspots before player-local"
    )
    Assert.equal(
        requestedFlags.enabledFamilies[5],
        "player_local",
        "expected canonical telemetry ordering to keep player-local last"
    )
    Assert.equal(
        WorldProbeTelemetryFlags.isEnabled(requestedFlags, "terrain"),
        true,
        "expected requested terrain telemetry to be enabled"
    )
    Assert.equal(
        WorldProbeTelemetryFlags.isEnabled(requestedFlags, "structures"),
        false,
        "expected unsupported telemetry families to remain disabled when not requested"
    )

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, "terrain,player_local,vegetation")
    local workspaceFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(
        Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)
    )
    Assert.equal(
        workspaceFlags.enabledFamilies[1],
        "terrain",
        "expected workspace telemetry families to parse from the shared workspace attribute"
    )
    Assert.equal(
        workspaceFlags.enabledFamilies[2],
        "vegetation",
        "expected workspace telemetry families to preserve canonical order"
    )
    Assert.equal(
        workspaceFlags.enabledFamilies[3],
        "player_local",
        "expected workspace telemetry families to keep player-local last"
    )

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, nil)
end
