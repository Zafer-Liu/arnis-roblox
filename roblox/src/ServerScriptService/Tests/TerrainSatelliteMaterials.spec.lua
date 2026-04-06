return function()
    local TerrainBuilder = require(script.Parent.Parent.ImportService.Builders.TerrainBuilder)
    local Assert = require(script.Parent.Assert)

    -- Verify that satellite-derived per-cell materials are used as the primary
    -- material source when terrainGrid.materials[] is populated.

    local satelliteChunk = {
        id = "terrain_satellite_materials",
        originStuds = { x = 0, y = 0, z = 0 },
        terrain = {
            cellSizeStuds = 16,
            width = 3,
            depth = 3,
            heights = {
                0, 0, 0,
                0, 0, 0,
                0, 0, 0,
            },
            materials = {
                "Sand", "Limestone", "Pavement",
                "Mud", "Sandstone", "Slate",
                "Asphalt", "Concrete", "Grass",
            },
            material = "Grass",
        },
    }

    local plan = TerrainBuilder.PrepareChunk(satelliteChunk)

    Assert.truthy(plan, "expected terrain build plan to be created")

    -- Satellite materials should be the primary source; slope should not override.
    Assert.equal(
        plan.cellMaterials[1][1],
        Enum.Material.Sand,
        "expected satellite-derived Sand material at (0,0)"
    )
    Assert.equal(
        plan.cellMaterials[1][2],
        Enum.Material.Limestone,
        "expected satellite-derived Limestone material at (1,0)"
    )
    Assert.equal(
        plan.cellMaterials[1][3],
        Enum.Material.Pavement,
        "expected satellite-derived Pavement material at (2,0)"
    )
    Assert.equal(
        plan.cellMaterials[2][1],
        Enum.Material.Mud,
        "expected satellite-derived Mud material at (0,1)"
    )
    Assert.equal(
        plan.cellMaterials[2][2],
        Enum.Material.Sandstone,
        "expected satellite-derived Sandstone material at (1,1)"
    )
    Assert.equal(
        plan.cellMaterials[2][3],
        Enum.Material.Slate,
        "expected satellite-derived Slate material at (2,1)"
    )
    Assert.equal(
        plan.cellMaterials[3][1],
        Enum.Material.Asphalt,
        "expected satellite-derived Asphalt material at (0,2)"
    )
    Assert.equal(
        plan.cellMaterials[3][2],
        Enum.Material.Concrete,
        "expected satellite-derived Concrete material at (1,2)"
    )

    -- Verify material richness reports the expanded palette.
    local stats = plan.terrainStats
    Assert.truthy(stats, "expected terrain stats to be present")
    Assert.truthy(
        stats.materialKindCount >= 8,
        "expected at least 8 distinct material kinds from satellite palette, got " .. tostring(stats.materialKindCount)
    )
    Assert.truthy(
        stats.nonGrassCellCount >= 8,
        "expected at least 8 non-grass cells from satellite palette, got " .. tostring(stats.nonGrassCellCount)
    )

    -- Verify slope-based fallback still works when satellite materials are absent.
    local slopeChunk = {
        id = "terrain_satellite_fallback",
        originStuds = { x = 0, y = 0, z = 0 },
        terrain = {
            cellSizeStuds = 16,
            width = 3,
            depth = 3,
            heights = {
                0, 0, 0,
                0, 40, 0,
                0, 0, 0,
            },
            material = "Grass",
        },
    }

    local slopePlan = TerrainBuilder.PrepareChunk(slopeChunk)
    Assert.truthy(slopePlan, "expected slope-fallback terrain build plan to be created")

    -- The steep center cell should get Rock from slope classification.
    local centerMat = slopePlan.cellMaterials[2][2]
    Assert.truthy(
        centerMat == Enum.Material.Rock or centerMat == Enum.Material.Ground,
        "expected slope-based fallback to classify steep center cell as Rock or Ground, got " .. tostring(centerMat)
    )
end
