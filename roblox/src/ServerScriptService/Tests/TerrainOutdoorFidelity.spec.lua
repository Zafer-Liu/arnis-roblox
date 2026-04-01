return function()
    local TerrainBuilder = require(script.Parent.Parent.ImportService.Builders.TerrainBuilder)
    local Assert = require(script.Parent.Assert)

    local offsets = TerrainBuilder._buildSubsampleOffsets(4, 1)
    Assert.equal(#offsets, 4, "expected 1-stud requested sampling to create four offsets per axis")
    Assert.equal(offsets[1], -1.5, "expected first 1-stud offset to land near the voxel edge")
    Assert.equal(offsets[4], 1.5, "expected last 1-stud offset to land near the opposite voxel edge")

    local plan = {
        origin = { x = 0, y = 0, z = 0 },
        cellSize = 2,
        gridW = 2,
        gridD = 1,
        rMinX = 0,
        rMinZ = 0,
        cellMaterials = {
            {
                { Name = "Grass" },
                { Name = "Rock" },
            },
        },
        voxelSubsampleOffsets = offsets,
        sampleInterpolatedHeight = function(cellX, _cellZ, _fracX, _fracZ)
            if cellX == 0 then
                return 4
            end
            return 12
        end,
    }

    -- A single 4-stud write voxel spans both 2-stud source cells, so the
    -- supersampled profile should average their heights and split material
    -- ownership evenly across the subsamples.
    local profile = TerrainBuilder._sampleVoxelColumnProfile(plan, 1, 1)
    Assert.equal(profile.sampleCount, 16, "expected a 4x4 supersample grid inside the write voxel")
    Assert.equal(profile.averageHeight, 8, "expected supersampled height to average both source cells")
    Assert.equal(profile.material.Name, "Grass", "expected tied dominant material to resolve deterministically")
    Assert.equal(profile.materialSampleCount, 8, "expected equal subsample ownership across the two source cells")
end
