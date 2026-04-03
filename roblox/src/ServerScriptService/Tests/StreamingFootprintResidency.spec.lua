return function()
    local Workspace = game:GetService("Workspace")

    local Assert = require(script.Parent.Assert)
    local ChunkLoader = require(script.Parent.Parent.ImportService.ChunkLoader)
    local StreamingService = require(script.Parent.Parent.ImportService.StreamingService)

    local function makeChunk(chunkId, originX)
        return {
            id = chunkId,
            originStuds = { x = originX, y = 0, z = 0 },
            roads = {},
            rails = {},
            buildings = {},
            water = {},
            props = {},
            landuse = {},
            barriers = {},
        }
    end

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "StreamingFootprintResidencyTest",
            generator = "test",
            source = "test",
            metersPerStud = 1,
            chunkSizeStuds = 100,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
        },
        chunkRefs = {
            {
                id = "overlap_chunk",
                originStuds = { x = 800, y = 0, z = 0 },
                footprintBounds = {
                    minX = 350,
                    minY = -50,
                    maxX = 1250,
                    maxY = 50,
                },
                shards = { "fake" },
                featureCount = 1,
                streamingCost = 1,
            },
        },
        GetChunk = function(_, chunkId)
            return makeChunk(chunkId, 800)
        end,
    }

    local options = {
        worldRootName = "StreamingFootprintResidencyWorld",
        config = {
            StreamingEnabled = true,
            StreamingTargetRadius = 400,
            HighDetailRadius = 400,
            ChunkSizeStuds = 100,
            TerrainMode = "none",
            RoadMode = "none",
            BuildingMode = "none",
            WaterMode = "none",
            LanduseMode = "none",
        },
    }

    ChunkLoader.Clear()
    StreamingService.Start(manifest, options)
    StreamingService.Update(Vector3.new(0, 0, 0))

    Assert.equal(
        Workspace:GetAttribute("ArnisStreamingCandidateChunkCount"),
        1,
        "expected footprint-overlapping chunk to be admitted even when its center bucket is outside the search radius"
    )

    local loaded = ChunkLoader.ListLoadedChunks(options.worldRootName)
    Assert.equal(#loaded, 1, "expected footprint-overlapping chunk to load from residency admission")
    Assert.equal(loaded[1], "overlap_chunk", "expected the overlapping chunk to be the loaded residency target")

    StreamingService.Stop()
    local worldRoot = Workspace:FindFirstChild(options.worldRootName)
    if worldRoot then
        worldRoot:Destroy()
    end
    ChunkLoader.Clear()
end
