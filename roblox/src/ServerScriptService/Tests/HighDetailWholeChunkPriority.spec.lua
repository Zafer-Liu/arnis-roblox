return function()
    local Workspace = game:GetService("Workspace")

    local Assert = require(script.Parent.Assert)
    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local ImportService = require(script.Parent.Parent.ImportService)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)

    local originalImportChunk = ImportService.ImportChunk
    local originalImportChunkSubplan = ImportService.ImportChunkSubplan

    local worldRootName = "GeneratedWorld_HighDetailWholeChunkPriority"
    local importOrder = {}

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "HighDetailWholeChunkPriority",
            generator = "test",
            source = "unit",
            metersPerStud = 1.0,
            chunkSizeStuds = 256,
            totalFeatures = 2,
        },
        chunkRefs = {
            {
                id = "priority_building_chunk",
                originStuds = { x = 140, y = 0, z = 0 },
                shards = { "fake" },
                featureCount = 3,
                subplans = {
                    { id = "landuse", layer = "landuse", featureCount = 1, streamingCost = 4 },
                    { id = "roads", layer = "roads", featureCount = 1, streamingCost = 6 },
                    { id = "buildings", layer = "buildings", featureCount = 1, streamingCost = 12 },
                },
            },
            {
                id = "closer_subplan_chunk",
                originStuds = { x = 20, y = 0, z = 0 },
                shards = { "fake" },
                featureCount = 1,
                subplans = {
                    { id = "roads", layer = "roads", featureCount = 1, streamingCost = 1 },
                },
            },
        },
        GetChunk = function(_, chunkId)
            if chunkId == "priority_building_chunk" then
                return {
                    id = chunkId,
                    originStuds = { x = 140, y = 0, z = 0 },
                    terrain = {
                        cellSizeStuds = 16,
                        width = 16,
                        depth = 16,
                        heights = table.create(16 * 16, 0),
                        material = "Grass",
                    },
                    roads = {},
                    rails = {},
                    buildings = {
                        {
                            id = "priority_building",
                            footprint = {
                                { x = 0, z = 0 },
                                { x = 28, z = 0 },
                                { x = 28, z = 18 },
                                { x = 0, z = 18 },
                            },
                            baseY = 0,
                            height = 24,
                            levels = 4,
                            roof = "flat",
                            material = "Concrete",
                        },
                    },
                    water = {},
                    props = {},
                    landuse = {
                        {
                            id = "priority_park",
                            kind = "park",
                            footprint = {
                                { x = 0, z = 0 },
                                { x = 32, z = 0 },
                                { x = 32, z = 32 },
                                { x = 0, z = 32 },
                            },
                        },
                    },
                    barriers = {},
                    subplans = {
                        { id = "landuse", layer = "landuse", featureCount = 1, streamingCost = 4 },
                        { id = "roads", layer = "roads", featureCount = 1, streamingCost = 6 },
                        { id = "buildings", layer = "buildings", featureCount = 1, streamingCost = 12 },
                    },
                }
            end

            return {
                id = chunkId,
                originStuds = { x = 20, y = 0, z = 0 },
                terrain = {
                    cellSizeStuds = 16,
                    width = 16,
                    depth = 16,
                    heights = table.create(16 * 16, 0),
                    material = "Grass",
                },
                roads = {
                    {
                        id = "closer_road",
                        kind = "secondary",
                        widthStuds = 16,
                        points = {
                            { x = 16, y = 0, z = 24 },
                            { x = 96, y = 0, z = 24 },
                        },
                    },
                },
                rails = {},
                buildings = {},
                water = {},
                props = {},
                landuse = {},
                barriers = {},
                subplans = {
                    { id = "roads", layer = "roads", featureCount = 1, streamingCost = 1 },
                },
            }
        end,
    }

    local function ensureChunkFolder(chunkId)
        local worldRoot = Workspace:FindFirstChild(worldRootName)
        if not worldRoot then
            worldRoot = Instance.new("Folder")
            worldRoot.Name = worldRootName
            worldRoot.Parent = Workspace
        end

        local chunkFolder = worldRoot:FindFirstChild(chunkId)
        if not chunkFolder then
            chunkFolder = Instance.new("Folder")
            chunkFolder.Name = chunkId
            chunkFolder.Parent = worldRoot
        end

        return chunkFolder
    end

    local ok, err = xpcall(function()
        ImportService.ImportChunk = function(chunk, options)
            importOrder[#importOrder + 1] = "chunk:" .. chunk.id
            return ensureChunkFolder(chunk.id)
        end

        ImportService.ImportChunkSubplan = function(chunk, subplan, options)
            importOrder[#importOrder + 1] = ("subplan:%s:%s"):format(chunk.id, subplan.id)
            return ensureChunkFolder(chunk.id), 0
        end

        StreamingService.Start(manifest, {
            worldRootName = worldRootName,
            config = {
                StreamingEnabled = true,
                StreamingTargetRadius = 512,
                HighDetailRadius = 512,
                StreamingMaxWorkItemsPerUpdate = 1,
                ChunkSizeStuds = 256,
                TerrainMode = "voxel",
                RoadMode = "mesh",
                BuildingMode = "shellMesh",
                WaterMode = "none",
                LanduseMode = "terrain",
                SubplanRollout = {
                    Enabled = true,
                    AllowedLayers = { "landuse", "roads", "buildings" },
                    AllowedChunkIds = { "priority_building_chunk", "closer_subplan_chunk" },
                },
            },
        })

        StreamingService.Update(Vector3.new(0, 0, 0))

        Assert.equal(
            importOrder[1],
            "chunk:priority_building_chunk",
            "expected high-detail whole-chunk building admission to outrank competing subplan work in the same startup window"
        )
    end, debug.traceback)

    ImportService.ImportChunk = originalImportChunk
    ImportService.ImportChunkSubplan = originalImportChunkSubplan
    StreamingService.Stop()
    ChunkLoader.Clear()

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    if worldRoot then
        worldRoot:Destroy()
    end

    if not ok then
        error(err, 0)
    end
end
