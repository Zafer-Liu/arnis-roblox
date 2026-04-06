return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Workspace = game:GetService("Workspace")
    local Assert = require(script.Parent.Assert)
    local WorldProbeTelemetryFlags = require(ReplicatedStorage.Shared.WorldProbeTelemetryFlags)

    Workspace:SetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR, nil)

    local defaultFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR))
    local defaultMarkerPayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        nearbyBuildingModels = 3,
    }
    local defaultShapedPayload = WorldProbeTelemetryFlags.annotateMarkerPayload(defaultMarkerPayload, defaultFlags)
    Assert.equal(defaultShapedPayload, defaultMarkerPayload, "expected marker annotation to stay in-place for compact defaults")
    Assert.equal(
        defaultShapedPayload.telemetryFamilies,
        nil,
        "expected compact default telemetry markers to omit the family list"
    )

    local defaultLocalExperiencePayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        playerLocalTelemetryEnabled = true,
        localSupport = { surfaceRole = "terrain" },
        localTerrain = { samplePattern = "cross_5" },
        localEnclosure = { nearbyWallParts = 2 },
        localRoofCover = { nearbyRoofParts = 1 },
    }
    WorldProbeTelemetryFlags.shapeLocalExperiencePayload(defaultLocalExperiencePayload, defaultFlags, false)
    Assert.equal(
        defaultLocalExperiencePayload.playerLocalTelemetryEnabled,
        false,
        "expected default/no-family local experience to be a tombstone"
    )
    Assert.equal(
        defaultLocalExperiencePayload.localSupport,
        nil,
        "expected default/no-family local experience to omit live support data"
    )
    Assert.equal(
        defaultLocalExperiencePayload.localTerrain,
        nil,
        "expected default/no-family local experience to omit live terrain data"
    )
    Assert.equal(
        defaultLocalExperiencePayload.localEnclosure,
        nil,
        "expected default/no-family local experience to omit live enclosure data"
    )
    Assert.equal(
        defaultLocalExperiencePayload.localRoofCover,
        nil,
        "expected default/no-family local experience to omit live roof cover data"
    )
    Assert.equal(
        defaultLocalExperiencePayload.telemetryFamilies,
        nil,
        "expected default/no-family local experience markers to stay compact"
    )

    local requestedFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(
        " player_local,terrain,roads,roads,unknown,water,hotspots,vegetation,structures "
    )
    local requestedMarkerPayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        nearbyBuildingModels = 3,
    }
    local requestedShapedPayload = WorldProbeTelemetryFlags.annotateMarkerPayload(requestedMarkerPayload, requestedFlags)
    Assert.equal(requestedShapedPayload, requestedMarkerPayload, "expected marker annotation to reuse the existing payload table")
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

    local requestedLocalExperiencePayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        playerLocalTelemetryEnabled = false,
        localSupport = { surfaceRole = "terrain" },
        localTerrain = { samplePattern = "cross_5" },
        localEnclosure = { nearbyWallParts = 2 },
        localRoofCover = { nearbyRoofParts = 1 },
    }
    WorldProbeTelemetryFlags.shapeLocalExperiencePayload(requestedLocalExperiencePayload, requestedFlags, true)
    Assert.equal(
        requestedLocalExperiencePayload.playerLocalTelemetryEnabled,
        true,
        "expected enabled player-local telemetry to preserve the live marker"
    )
    Assert.equal(
        requestedLocalExperiencePayload.localSupport.surfaceRole,
        "terrain",
        "expected enabled player-local telemetry to preserve live support data"
    )
    Assert.equal(
        requestedLocalExperiencePayload.localTerrain.samplePattern,
        "cross_5",
        "expected enabled player-local telemetry to preserve live terrain data"
    )
    Assert.equal(
        requestedLocalExperiencePayload.telemetryFamilies[1],
        "terrain",
        "expected enabled local experience markers to keep canonical family order"
    )
    Assert.equal(
        requestedLocalExperiencePayload.telemetryFamilies[7],
        "player_local",
        "expected enabled local experience markers to keep player-local last"
    )
    Assert.equal(
        requestedLocalExperiencePayload.localEnclosure.nearbyWallParts,
        2,
        "expected enabled local experience markers to preserve compact enclosure counts"
    )
    Assert.equal(
        requestedLocalExperiencePayload.localRoofCover.nearbyRoofParts,
        1,
        "expected enabled local experience markers to preserve compact roof-cover counts"
    )
    Assert.equal(
        requestedLocalExperiencePayload.localSupport.supportY,
        nil,
        "expected absent support metrics to stay absent in the compact payload"
    )
    Assert.equal(
        requestedLocalExperiencePayload.characterPosition,
        nil,
        "expected enabled local experience markers to drop characterPosition for log safety"
    )

    local disabledAfterEnabledPayload = {
        worldRootName = "GeneratedWorld_Austin",
        worldRootExists = true,
        playerLocalTelemetryEnabled = true,
        localSupport = { surfaceRole = "terrain" },
        localTerrain = { samplePattern = "cross_5" },
        localEnclosure = { nearbyWallParts = 2 },
        localRoofCover = { nearbyRoofParts = 1 },
    }
    WorldProbeTelemetryFlags.shapeLocalExperiencePayload(disabledAfterEnabledPayload, defaultFlags, false)
    Assert.equal(
        disabledAfterEnabledPayload.playerLocalTelemetryEnabled,
        false,
        "expected disabling player-local telemetry to replace the live marker with a tombstone"
    )
    Assert.equal(
        disabledAfterEnabledPayload.localSupport,
        nil,
        "expected the tombstone marker to drop live support data"
    )
    Assert.equal(
        disabledAfterEnabledPayload.localTerrain,
        nil,
        "expected the tombstone marker to drop live terrain data"
    )
    Assert.equal(
        disabledAfterEnabledPayload.localEnclosure,
        nil,
        "expected the tombstone marker to drop live enclosure data"
    )
    Assert.equal(
        disabledAfterEnabledPayload.localRoofCover,
        nil,
        "expected the tombstone marker to drop live roof cover data"
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
