local WorldProbeTelemetryFlags = {}

WorldProbeTelemetryFlags.WORKSPACE_ATTR = "ArnisTelemetryFamilies"
WorldProbeTelemetryFlags.SUPPORTED_FAMILY_ORDER = {
    "terrain",
    "roads",
    "water",
    "vegetation",
    "structures",
    "hotspots",
    "player_local",
}
WorldProbeTelemetryFlags.SUPPORTED_FAMILIES = {
    terrain = true,
    roads = true,
    water = true,
    vegetation = true,
    structures = true,
    hotspots = true,
    player_local = true,
}

local function trim(value)
    if type(value) ~= "string" then
        return ""
    end

    local trimmed = string.match(value, "^%s*(.-)%s*$")
    if trimmed == nil then
        return ""
    end
    return trimmed
end

local function normalizeFamilyName(value)
    local familyName = trim(value)
    if familyName == "" then
        return nil
    end
    familyName = string.lower(familyName)
    if not WorldProbeTelemetryFlags.SUPPORTED_FAMILIES[familyName] then
        return nil
    end
    return familyName
end

function WorldProbeTelemetryFlags.parseTelemetryFamilies(value)
    local familySet = {}

    if type(value) == "string" then
        for token in string.gmatch(value, "[^,]+") do
            local familyName = normalizeFamilyName(token)
            if familyName ~= nil then
                familySet[familyName] = true
            end
        end
    elseif type(value) == "table" then
        for _, token in ipairs(value) do
            local familyName = normalizeFamilyName(token)
            if familyName ~= nil then
                familySet[familyName] = true
            end
        end
    end

    local enabledFamilies = {}
    for _, familyName in ipairs(WorldProbeTelemetryFlags.SUPPORTED_FAMILY_ORDER) do
        if familySet[familyName] then
            enabledFamilies[#enabledFamilies + 1] = familyName
        end
    end

    return {
        familySet = familySet,
        enabledFamilies = enabledFamilies,
    }
end

function WorldProbeTelemetryFlags.isEnabled(telemetryFlags, family)
    return type(telemetryFlags) == "table"
        and type(family) == "string"
        and telemetryFlags.familySet ~= nil
        and telemetryFlags.familySet[family] == true
end

return WorldProbeTelemetryFlags
