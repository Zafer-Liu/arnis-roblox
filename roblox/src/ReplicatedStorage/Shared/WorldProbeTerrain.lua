local WorldProbeTerrain = {}

local function roundTenths(value)
    if type(value) ~= "number" then
        return nil
    end
    return math.round(value * 10) / 10
end

function WorldProbeTerrain.summarizeTerrainSamples(samples, options)
    options = options or {}
    local totalSlots = if type(samples) == "table" then #samples else 0
    local samplePattern = if type(options.samplePattern) == "string" and options.samplePattern ~= "" then options.samplePattern else "unknown"
    local sampleRadiusStuds = roundTenths(options.sampleRadiusStuds)
    local centerIndex = if type(options.centerIndex) == "number" then options.centerIndex else math.ceil(totalSlots * 0.5)
    local neighborPairs = if type(options.neighborPairs) == "table" then options.neighborPairs else {}

    local sampleCount = 0
    local minTerrainY = nil
    local maxTerrainY = nil

    if type(samples) == "table" then
        for _, sample in ipairs(samples) do
            local height = if type(sample) == "table" then sample.terrainY else nil
            if type(height) == "number" then
                sampleCount += 1
                if minTerrainY == nil or height < minTerrainY then
                    minTerrainY = height
                end
                if maxTerrainY == nil or height > maxTerrainY then
                    maxTerrainY = height
                end
            end
        end
    end

    local missingSampleCount = math.max(totalSlots - sampleCount, 0)
    local centerTerrainY = nil
    if type(samples) == "table" and type(samples[centerIndex]) == "table" and type(samples[centerIndex].terrainY) == "number" then
        centerTerrainY = samples[centerIndex].terrainY
    end

    local totalStep = 0
    local stepCount = 0
    local maxStepStuds = nil
    for _, pair in ipairs(neighborPairs) do
        local firstIndex = pair[1]
        local secondIndex = pair[2]
        local firstHeight = if type(samples) == "table" and type(samples[firstIndex]) == "table" then samples[firstIndex].terrainY else nil
        local secondHeight = if type(samples) == "table" and type(samples[secondIndex]) == "table" then samples[secondIndex].terrainY else nil
        if type(firstHeight) == "number" and type(secondHeight) == "number" then
            local delta = math.abs(firstHeight - secondHeight)
            totalStep += delta
            stepCount += 1
            if maxStepStuds == nil or delta > maxStepStuds then
                maxStepStuds = delta
            end
        end
    end

    local status = "ok"
    if centerTerrainY == nil or sampleCount < 3 or stepCount <= 0 then
        status = "insufficient_samples"
    end

    return {
        status = status,
        samplePattern = samplePattern,
        sampleRadiusStuds = sampleRadiusStuds,
        sampleCount = sampleCount,
        missingSampleCount = missingSampleCount,
        centerTerrainY = roundTenths(centerTerrainY),
        minTerrainY = roundTenths(minTerrainY),
        maxTerrainY = roundTenths(maxTerrainY),
        heightRangeStuds = if minTerrainY ~= nil and maxTerrainY ~= nil then roundTenths(maxTerrainY - minTerrainY) else nil,
        maxStepStuds = if status == "ok" then roundTenths(maxStepStuds) else nil,
        meanAbsStepStuds = if status == "ok" and stepCount > 0 then roundTenths(totalStep / stepCount) else nil,
    }
end

return WorldProbeTerrain
