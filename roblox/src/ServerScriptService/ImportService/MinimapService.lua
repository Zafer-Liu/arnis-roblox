local HttpService = game:GetService("HttpService")
local Workspace = game:GetService("Workspace")

local MinimapService = {}

local WORLD_ROOT_ATTR = "ArnisWorldRootName"
local MINIMAP_WORLD_ROOT_ATTR = "ArnisMinimapWorldRootName"
local ENABLED_ATTR = "ArnisMinimapEnabled"
local CHUNK_JSON_ATTR = "ArnisMinimapChunkJson"
local CHUNK_ID_ATTR = "ArnisMinimapChunkId"

local function copyPoints(points)
    local result = table.create(#(points or {}))
    for index, point in ipairs(points or {}) do
        result[index] = {
            x = point.x,
            z = point.z,
        }
    end
    return result
end

local function copyLanduse(landuse)
    local result = table.create(#(landuse or {}))
    for index, entry in ipairs(landuse or {}) do
        result[index] = {
            kind = entry.kind,
            footprint = copyPoints(entry.footprint),
        }
    end
    return result
end

local function copyRoads(roads)
    local result = table.create(#(roads or {}))
    for index, road in ipairs(roads or {}) do
        result[index] = {
            kind = road.kind,
            widthStuds = road.widthStuds,
            points = copyPoints(road.points),
        }
    end
    return result
end

local function copyBuildings(buildings)
    local result = table.create(#(buildings or {}))
    for index, building in ipairs(buildings or {}) do
        -- Building metadata beyond footprint lets the minimap colour
        -- buildings by kind / wall color / height rather than a single
        -- flat "building" hue. Previously the feed was footprint-only,
        -- so every building rendered identically and the minimap
        -- looked jagged and unvaried.
        local wallColor = building.wallColor
        result[index] = {
            footprint = copyPoints(building.footprint),
            kind = building.kind or building.usage or nil,
            height = building.height or nil,
            wallColor = type(wallColor) == "table"
                and { r = wallColor.r, g = wallColor.g, b = wallColor.b }
                or nil,
        }
    end
    return result
end

local function copyWater(water)
    local result = table.create(#(water or {}))
    for index, entry in ipairs(water or {}) do
        result[index] = {
            kind = entry.kind,
            footprint = copyPoints(entry.footprint),
            points = copyPoints(entry.points),
            widthStuds = entry.widthStuds,
        }
    end
    return result
end

-- Railway tracks as a first-class minimap layer. The chunk data
-- carries `rails[]` with the same point-list shape as roads; publishing
-- them here lets the controller draw a distinct color so transit maps
-- actually show the rail network.
local function copyRails(rails)
    local result = table.create(#(rails or {}))
    for index, rail in ipairs(rails or {}) do
        result[index] = {
            kind = rail.kind,
            widthStuds = rail.widthStuds,
            points = copyPoints(rail.points),
        }
    end
    return result
end

-- Walls, fences, and other linear barriers. Same shape as roads.
local function copyBarriers(barriers)
    local result = table.create(#(barriers or {}))
    for index, barrier in ipairs(barriers or {}) do
        result[index] = {
            kind = barrier.kind,
            widthStuds = barrier.widthStuds,
            points = copyPoints(barrier.points),
        }
    end
    return result
end

local function buildChunkSnapshot(chunkData)
    local origin = chunkData.originStuds or {}
    return {
        id = chunkData.id,
        originStuds = {
            x = origin.x or 0,
            z = origin.z or 0,
        },
        landuse = copyLanduse(chunkData.landuse),
        roads = copyRoads(chunkData.roads),
        rails = copyRails(chunkData.rails),
        buildings = copyBuildings(chunkData.buildings),
        water = copyWater(chunkData.water),
        barriers = copyBarriers(chunkData.barriers),
    }
end

function MinimapService.RegisterChunk(chunkFolder, chunkData)
    if not chunkFolder or type(chunkData) ~= "table" then
        return
    end

    chunkFolder:SetAttribute(CHUNK_ID_ATTR, chunkData.id or chunkFolder.Name)
    chunkFolder:SetAttribute(CHUNK_JSON_ATTR, HttpService:JSONEncode(buildChunkSnapshot(chunkData)))
end

function MinimapService.ClearChunk(chunkFolder)
    if not chunkFolder then
        return
    end

    chunkFolder:SetAttribute(CHUNK_JSON_ATTR, nil)
    chunkFolder:SetAttribute(CHUNK_ID_ATTR, nil)
end

function MinimapService.Start(options)
    local resolvedOptions = options or {}
    local worldRootName = Workspace:GetAttribute(WORLD_ROOT_ATTR)
        or resolvedOptions.worldRootName
        or "GeneratedWorld"
    Workspace:SetAttribute(ENABLED_ATTR, true)
    Workspace:SetAttribute(MINIMAP_WORLD_ROOT_ATTR, Workspace:GetAttribute(WORLD_ROOT_ATTR) or worldRootName)
end

function MinimapService.Stop()
    Workspace:SetAttribute(ENABLED_ATTR, false)
    Workspace:SetAttribute(MINIMAP_WORLD_ROOT_ATTR, nil)
end

return MinimapService
