return function()
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local Assert = require(script.Parent.Assert)
    local WorldProbeTerrain = require(ReplicatedStorage.Shared.WorldProbeTerrain)

    local flatSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 12 },
        { terrainY = 12 },
        { terrainY = 12 },
        { terrainY = 12 },
        { terrainY = 12 },
    }, {
        centerIndex = 3,
        samplePattern = "cross_5",
        sampleRadiusStuds = 12,
        neighborPairs = {
            { 3, 1 },
            { 3, 2 },
            { 3, 4 },
            { 3, 5 },
        },
    })
    Assert.equal(flatSummary.status, "ok", "expected flat terrain summary to be valid")
    Assert.equal(flatSummary.sampleCount, 5, "expected all flat terrain samples to count")
    Assert.equal(flatSummary.missingSampleCount, 0, "expected no missing flat terrain samples")
    Assert.equal(flatSummary.centerTerrainY, 12, "expected center terrain height to be preserved")
    Assert.equal(flatSummary.heightRangeStuds, 0, "expected flat terrain to have zero height range")
    Assert.equal(flatSummary.maxStepStuds, 0, "expected flat terrain to have zero max step")
    Assert.equal(flatSummary.meanAbsStepStuds, 0, "expected flat terrain to have zero mean step")

    local steppedSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 8.04 },
        { terrainY = 10.06 },
        { terrainY = 12.04 },
        { terrainY = 14.04 },
        { terrainY = 16.04 },
    }, {
        centerIndex = 3,
        samplePattern = "cross_5",
        sampleRadiusStuds = 12,
        neighborPairs = {
            { 3, 1 },
            { 3, 2 },
            { 3, 4 },
            { 3, 5 },
        },
    })
    Assert.equal(steppedSummary.status, "ok", "expected stepped terrain summary to remain valid")
    Assert.equal(steppedSummary.centerTerrainY, 12, "expected center terrain height to round to tenths")
    Assert.equal(steppedSummary.minTerrainY, 8, "expected minimum terrain height to round to tenths")
    Assert.equal(steppedSummary.maxTerrainY, 16, "expected maximum terrain height to round to tenths")
    Assert.equal(steppedSummary.heightRangeStuds, 8, "expected height range to reflect min/max terrain spread")
    Assert.equal(steppedSummary.maxStepStuds, 4, "expected max step to follow the steepest adjacent pair")
    Assert.equal(steppedSummary.meanAbsStepStuds, 3, "expected mean step to average the adjacent terrain deltas")

    local sparseSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = nil },
        { terrainY = nil },
        { terrainY = 10.04 },
        { terrainY = nil },
        { terrainY = nil },
    }, {
        centerIndex = 3,
        samplePattern = "cross_5",
        sampleRadiusStuds = 12,
        neighborPairs = {
            { 3, 1 },
            { 3, 2 },
            { 3, 4 },
            { 3, 5 },
        },
    })
    Assert.equal(
        sparseSummary.status,
        "insufficient_samples",
        "expected sparse terrain summary to report insufficient samples instead of inventing step metrics"
    )
    Assert.equal(sparseSummary.sampleCount, 1, "expected sparse terrain summary to count only real terrain hits")
    Assert.equal(sparseSummary.missingSampleCount, 4, "expected sparse terrain summary to count missing samples")
    Assert.equal(sparseSummary.maxStepStuds, nil, "expected sparse terrain summary not to invent max step")
    Assert.equal(sparseSummary.meanAbsStepStuds, nil, "expected sparse terrain summary not to invent mean step")
end
