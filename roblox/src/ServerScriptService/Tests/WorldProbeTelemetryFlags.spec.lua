return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Workspace = game:GetService("Workspace")
    local Assert = require(script.Parent.Assert)
    local WorldProbeTelemetryFlags = require(ReplicatedStorage.Shared.WorldProbeTelemetryFlags)

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, nil)

    local defaultFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR))
    local defaultPayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        nearbyBuildingModels = 3,
    }
    local defaultShapedPayload = WorldProbeTelemetryFlags.annotateMarkerPayload(defaultPayload, defaultFlags)
    Assert.equal(defaultShapedPayload, defaultPayload, "expected marker annotation to stay in-place for compact defaults")
    Assert.equal(
        defaultShapedPayload.telemetryFamilies,
        nil,
        "expected compact default telemetry markers to omit the family list"
    )

    local requestedFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(
        " player_local,terrain,roads,roads,unknown,water,hotspots,vegetation,structures "
    )
    local requestedPayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        nearbyBuildingModels = 3,
    }
    local requestedShapedPayload = WorldProbeTelemetryFlags.annotateMarkerPayload(requestedPayload, requestedFlags)
    Assert.equal(requestedShapedPayload, requestedPayload, "expected marker annotation to reuse the existing payload table")
    Assert.equal(requestedShapedPayload.worldRootName, "GeneratedWorld_Austin", "expected annotation to preserve the world root name")
    Assert.equal(requestedShapedPayload.nearbyBuildingModels, 3, "expected annotation to preserve the core payload")
    Assert.equal(requestedShapedPayload.telemetryFamilies[1], "terrain", "expected canonical family order to start with terrain")
    Assert.equal(requestedShapedPayload.telemetryFamilies[2], "roads", "expected canonical family order to include roads second")
    Assert.equal(requestedShapedPayload.telemetryFamilies[3], "water", "expected canonical family order to include water third")
    Assert.equal(
        requestedShapedPayload.telemetryFamilies[4],
        "vegetation",
        "expected canonical family order to include vegetation fourth"
    )
    Assert.equal(
        requestedShapedPayload.telemetryFamilies[5],
        "structures",
        "expected canonical family order to include structures fifth"
    )
    Assert.equal(
        requestedShapedPayload.telemetryFamilies[6],
        "hotspots",
        "expected canonical family order to include hotspots sixth"
    )
    Assert.equal(
        requestedShapedPayload.telemetryFamilies[7],
        "player_local",
        "expected canonical family order to keep player-local last"
    )

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, "terrain,player_local,vegetation")
    local workspaceFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(
        Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)
    )
    local workspacePayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        nearbyBuildingModels = 3,
    }
    WorldProbeTelemetryFlags.annotateMarkerPayload(workspacePayload, workspaceFlags)
    Assert.equal(
        workspacePayload.telemetryFamilies[1],
        "terrain",
        "expected workspace telemetry families to parse from the shared workspace attribute"
    )
    Assert.equal(
        workspacePayload.telemetryFamilies[2],
        "vegetation",
        "expected workspace telemetry families to preserve canonical order"
    )
    Assert.equal(
        workspacePayload.telemetryFamilies[3],
        "player_local",
        "expected workspace telemetry families to keep player-local last"
    )

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, nil)
end
