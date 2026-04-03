return function()
    local TerrainBuilder = require(script.Parent.Parent.ImportService.Builders.TerrainBuilder)
    local Assert = require(script.Parent.Assert)

    local chunk = {
        id = "terrain_plan_reuse",
        originStuds = { x = 0, y = 0, z = 0 },
        terrain = {
            cellSizeStuds = 16,
            width = 4,
            depth = 4,
            heights = {
                0,
                1,
                2,
                3,
                1,
                2,
                3,
                4,
                2,
                3,
                4,
                5,
                3,
                4,
                5,
                6,
            },
            material = "Grass",
        },
    }

    local firstPlan = TerrainBuilder.PrepareChunk(chunk)
    local secondPlan = TerrainBuilder.PrepareChunk(chunk)

    Assert.truthy(firstPlan, "expected terrain build plan to be created")
    Assert.equal(firstPlan, secondPlan, "expected terrain build plan to be reused for the same chunk table")
    Assert.equal(
        TerrainBuilder.GetPreparedChunkPlan(chunk),
        firstPlan,
        "expected prepared terrain build plan to stay attached to the chunk"
    )
    Assert.equal(
        firstPlan.writeResolution,
        4,
        "expected terrain plans to use Roblox's required 4-stud write resolution"
    )
    Assert.equal(
        firstPlan.requestedSampleResolution,
        1,
        "expected terrain plans to preserve the configured sampling intent"
    )

    local eastNeighborChunk = {
        id = "terrain_plan_reuse_east",
        originStuds = { x = 64, y = 0, z = 0 },
        terrain = {
            cellSizeStuds = 16,
            width = 4,
            depth = 4,
            heights = {
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
                30,
            },
            material = "Grass",
        },
    }

    local neighborAwarePlan = TerrainBuilder.PrepareChunk(chunk, {
        terrainNeighbors = {
            east = {
                id = eastNeighborChunk.id,
                terrain = eastNeighborChunk.terrain,
            },
        },
    })

    Assert.notEqual(
        neighborAwarePlan,
        firstPlan,
        "expected neighbor-aware terrain context to invalidate a stale seam-blind cached plan"
    )
    Assert.equal(
        neighborAwarePlan.terrainNeighborSignature,
        "east=" .. eastNeighborChunk.id,
        "expected terrain plans to derive a deterministic signature from neighbor context"
    )
    Assert.truthy(
        neighborAwarePlan.sampleInterpolatedHeight(3, 1, 1, 0) > firstPlan.sampleInterpolatedHeight(3, 1, 1, 0),
        "expected east-edge interpolation to pick up the new neighbor height after cache invalidation"
    )

    local implicitReusePlan = TerrainBuilder.PrepareChunk(chunk)
    Assert.equal(
        implicitReusePlan,
        neighborAwarePlan,
        "expected implicit prepare calls to reuse the best cached seam-aware plan instead of downgrading to seam-blind terrain"
    )

    local revisedEastNeighborTerrain = {
        cellSizeStuds = 16,
        width = 4,
        depth = 4,
        heights = {
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
            45,
        },
        material = "Grass",
    }

    local sameIdRevisedNeighborPlan = TerrainBuilder.PrepareChunk(chunk, {
        terrainNeighbors = {
            east = {
                id = eastNeighborChunk.id,
                terrain = revisedEastNeighborTerrain,
            },
        },
    })

    Assert.notEqual(
        sameIdRevisedNeighborPlan,
        neighborAwarePlan,
        "expected revised seam-aware terrain context to invalidate the cached plan even when the neighbor chunk id stays stable"
    )
    Assert.truthy(
        string.find(sameIdRevisedNeighborPlan.terrainNeighborSignature, "east=" .. eastNeighborChunk.id, 1, true) ~= nil,
        "expected revised seam-aware signature to retain the deterministic neighbor id token"
    )
    Assert.notEqual(
        sameIdRevisedNeighborPlan.terrainNeighborSignature,
        neighborAwarePlan.terrainNeighborSignature,
        "expected seam-aware signature to change when the neighbor terrain payload changes under the same chunk id"
    )
    Assert.truthy(
        sameIdRevisedNeighborPlan.sampleInterpolatedHeight(3, 1, 1, 0)
            > neighborAwarePlan.sampleInterpolatedHeight(3, 1, 1, 0),
        "expected east-edge interpolation to pick up revised neighbor terrain heights even when the neighbor id stays stable"
    )
end
