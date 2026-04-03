return function()
    local Assert = require(script.Parent.Assert)
    local ReplicatedStorage = game:GetService("ReplicatedStorage")
    local WorldProbeTerrain = require(ReplicatedStorage.Shared.WorldProbeTerrain)

    local flatSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 12, terrainMaterial = "Grass" },
        { terrainY = 12, terrainMaterial = "Grass" },
        { terrainY = 12, terrainMaterial = "Grass" },
        { terrainY = 12, terrainMaterial = "Grass" },
        { terrainY = 12, terrainMaterial = "Grass" },
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
        edgeIndices = { 1, 2, 4, 5 },
    })
    Assert.equal(flatSummary.status, "ok", "expected flat terrain summary to be valid")
    Assert.equal(flatSummary.sampleCount, 5, "expected all flat terrain samples to count")
    Assert.equal(flatSummary.missingSampleCount, 0, "expected no missing flat terrain samples")
    Assert.equal(flatSummary.centerTerrainY, 12, "expected center terrain height to be preserved")
    Assert.equal(flatSummary.heightRangeStuds, 0, "expected flat terrain to have zero height range")
    Assert.equal(flatSummary.maxStepStuds, 0, "expected flat terrain to have zero max step")
    Assert.equal(flatSummary.meanAbsStepStuds, 0, "expected flat terrain to have zero mean step")
    Assert.equal(flatSummary.materialKindCount, 1, "expected flat terrain to report one terrain material")
    Assert.equal(flatSummary.dominantMaterial, "Grass", "expected flat terrain to report grass as dominant")
    Assert.equal(flatSummary.dominantMaterialSampleCount, 5, "expected flat terrain to count all grass samples")
    Assert.equal(flatSummary.nonGrassSampleCount, 0, "expected flat terrain to report no non-grass samples")

    local steppedSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 8.04, terrainMaterial = "Grass" },
        { terrainY = 10.06, terrainMaterial = "Mud" },
        { terrainY = 12.04, terrainMaterial = "Grass" },
        { terrainY = 14.04, terrainMaterial = "Grass" },
        { terrainY = 16.04, terrainMaterial = "Mud" },
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
        edgeIndices = { 1, 2, 4, 5 },
    })
    Assert.equal(steppedSummary.status, "ok", "expected stepped terrain summary to remain valid")
    Assert.equal(steppedSummary.centerTerrainY, 12, "expected center terrain height to round to tenths")
    Assert.equal(steppedSummary.minTerrainY, 8, "expected minimum terrain height to round to tenths")
    Assert.equal(steppedSummary.maxTerrainY, 16, "expected maximum terrain height to round to tenths")
    Assert.equal(steppedSummary.heightRangeStuds, 8, "expected height range to reflect min/max terrain spread")
    Assert.equal(steppedSummary.maxStepStuds, 4, "expected max step to follow the steepest adjacent pair")
    Assert.equal(steppedSummary.meanAbsStepStuds, 3, "expected mean step to average the adjacent terrain deltas")
    Assert.equal(steppedSummary.materialKindCount, 2, "expected stepped terrain to preserve multiple materials")
    Assert.equal(steppedSummary.dominantMaterial, "Grass", "expected stepped terrain to report grass as dominant")
    Assert.equal(steppedSummary.dominantMaterialSampleCount, 3, "expected stepped terrain to count dominant samples")
    Assert.equal(steppedSummary.nonGrassSampleCount, 2, "expected stepped terrain to count non-grass samples")

    local sparseSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = nil },
        { terrainY = nil },
        { terrainY = 10.04, terrainMaterial = "Sand" },
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
        edgeIndices = { 1, 2, 4, 5 },
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
    Assert.equal(sparseSummary.materialKindCount, 1, "expected sparse terrain summary to keep real material hits")
    Assert.equal(sparseSummary.dominantMaterial, "Sand", "expected sparse terrain summary to keep the sampled material")
    Assert.equal(
        sparseSummary.nonGrassSampleCount,
        1,
        "expected sparse terrain summary to count non-grass material hits"
    )

    local fullSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 8.0, terrainMaterial = "Grass" },
        { terrainY = 12.0, terrainMaterial = "Grass" },
        { terrainY = 10.0, terrainMaterial = "Grass" },
        { terrainY = 14.0, terrainMaterial = "Rock" },
        { terrainY = 6.0, terrainMaterial = "Grass" },
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
        edgeIndices = { 1, 2, 4, 5 },
    })

    Assert.equal(fullSummary.missingEdgeSampleCount, 0, "expected full edge coverage to report no missing edge samples")
    Assert.equal(fullSummary.edgeTerrainYRangeStuds, 8, "expected edge range to capture the perimeter cliff")
    Assert.equal(
        fullSummary.centerEdgeMaxDeltaStuds,
        4,
        "expected center-to-edge delta to stay visible for seam checks"
    )

    local sparseEdgeSummary = WorldProbeTerrain.summarizeTerrainSamples({
        { terrainY = 10.0, terrainMaterial = "Grass" },
        { terrainY = nil, terrainMaterial = "Grass" },
        { terrainY = 12.0, terrainMaterial = "Grass" },
        { terrainY = 8.0, terrainMaterial = "Rock" },
        { terrainY = 14.0, terrainMaterial = "Grass" },
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
        edgeIndices = { 1, 2, 4, 5 },
    })

    Assert.equal(
        sparseEdgeSummary.missingEdgeSampleCount,
        1,
        "expected one missing edge sample to be reported explicitly"
    )
    Assert.equal(
        sparseEdgeSummary.edgeTerrainYRangeStuds,
        6,
        "expected sparse edge coverage to preserve the sampled edge spread"
    )
    Assert.equal(
        sparseEdgeSummary.centerEdgeMaxDeltaStuds,
        4,
        "expected sparse edge coverage to keep center-edge cliff visibility"
    )
end
