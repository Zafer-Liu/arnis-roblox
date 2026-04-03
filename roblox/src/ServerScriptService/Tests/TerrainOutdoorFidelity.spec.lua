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

    local seamPlan = {
        terrainGrid = {
            width = 2,
            depth = 2,
            heights = {
                4,
                8,
                6,
                10,
            },
        },
        gridW = 2,
        gridD = 2,
        terrainNeighbors = {
            east = {
                id = "1_0",
                terrain = {
                    width = 2,
                    depth = 2,
                    heights = {
                        12,
                        14,
                        16,
                        18,
                    },
                },
            },
            southEast = {
                id = "1_1",
                terrain = {
                    width = 2,
                    depth = 2,
                    heights = {
                        20,
                        22,
                        24,
                        26,
                    },
                },
            },
        },
    }

    Assert.equal(
        TerrainBuilder._resolveNeighborHeightSample(seamPlan, 2, 0),
        12,
        "expected east seam samples to consult the adjacent chunk instead of clamping back into the local edge"
    )
    Assert.equal(
        TerrainBuilder._resolveNeighborHeightSample(seamPlan, 2, 1),
        16,
        "expected east seam rows to preserve adjacent chunk continuity across the shared border"
    )
    Assert.equal(
        TerrainBuilder._resolveNeighborHeightSample(seamPlan, 2, 2),
        20,
        "expected diagonal seam samples to consult the diagonal chunk corner instead of creating a vertical local cliff"
    )

    local blendedCornerPlan = {
        terrainGrid = {
            width = 2,
            depth = 2,
            heights = {
                4,
                8,
                6,
                10,
            },
        },
        gridW = 2,
        gridD = 2,
        terrainNeighbors = {
            east = {
                id = "1_0",
                terrain = {
                    width = 2,
                    depth = 2,
                    heights = {
                        12,
                        14,
                        16,
                        18,
                    },
                },
            },
            south = {
                id = "0_1",
                terrain = {
                    width = 2,
                    depth = 2,
                    heights = {
                        30,
                        32,
                        34,
                        36,
                    },
                },
            },
        },
    }
    Assert.equal(
        TerrainBuilder._resolveNeighborHeightSample(blendedCornerPlan, 2, 2),
        23,
        "expected missing diagonal seam samples to blend adjacent edge neighbors instead of snapping to a single edge"
    )

    local edgeProgressionPlan = {
        terrainGrid = {
            width = 5,
            depth = 1,
            heights = {
                0,
                0,
                0,
                0,
                0,
            },
        },
        gridW = 5,
        gridD = 1,
        terrainNeighbors = {
            east = {
                id = "1_0",
                terrain = {
                    width = 3,
                    depth = 1,
                    heights = {
                        100,
                        200,
                        300,
                    },
                },
            },
        },
    }
    edgeProgressionPlan.sampleInterpolatedHeight = function(cellX, cellZ)
        return TerrainBuilder._resolveNeighborHeightSample(edgeProgressionPlan, cellX, cellZ)
    end
    edgeProgressionPlan.voxelSubsampleOffsets = offsets
    edgeProgressionPlan.cellMaterials = {
        {
            { Name = "Grass" },
            { Name = "Grass" },
            { Name = "Grass" },
            { Name = "Grass" },
            { Name = "Grass" },
        },
    }

    local edgeProfile = TerrainBuilder._sampleVoxelColumnProfile(edgeProgressionPlan, 2, 1)
    Assert.equal(edgeProfile.sampleCount, 16, "expected edge voxel profile to keep full supersampling")
    Assert.equal(
        edgeProfile.averageHeight,
        150,
        "expected edge voxel profile to advance through the neighbor chunk instead of flattening to the border cell"
    )

    local peakProfilePlan = {
        origin = { x = 0, y = 0, z = 0 },
        cellSize = 2,
        gridW = 2,
        gridD = 2,
        rMinX = 0,
        rMinZ = 0,
        cellMaterials = {
            {
                { Name = "Rock" },
                { Name = "Rock" },
            },
            {
                { Name = "Rock" },
                { Name = "Rock" },
            },
        },
        voxelSubsampleOffsets = offsets,
        sampleInterpolatedHeight = function(cellX, cellZ)
            if cellX == 1 and cellZ == 1 then
                return 16
            end
            return 0
        end,
    }

    local peakProfile = TerrainBuilder._sampleVoxelColumnProfile(peakProfilePlan, 1, 1)
    Assert.equal(peakProfile.averageHeight, 4, "expected steep mixed voxels to retain average-height telemetry")
    Assert.equal(peakProfile.heightRange, 16, "expected steep mixed voxels to capture the local height spread")
    Assert.equal(
        peakProfile.peakSampleCoverage,
        0.25,
        "expected steep mixed voxels to record how much of the supersample grid actually reaches the peak"
    )
    Assert.equal(
        peakProfile.surfaceHeight,
        7,
        "expected isolated steep voxels to bias toward the peak without snapping the whole write voxel to a flat peak plane"
    )
    Assert.equal(
        peakProfile.surfaceFillDepth,
        1,
        "expected steep mixed voxels to shrink their rendered fill depth to a single terrain voxel instead of becoming full-height terrain boxes"
    )
    Assert.truthy(
        peakProfile.edgeOccupancyScale < 1,
        "expected steep mixed voxels to taper their top and bottom occupancy instead of writing a fully dense cap"
    )

    local moderateProfilePlan = {
        origin = { x = 0, y = 0, z = 0 },
        cellSize = 2,
        gridW = 2,
        gridD = 2,
        rMinX = 0,
        rMinZ = 0,
        cellMaterials = {
            {
                { Name = "Ground" },
                { Name = "Ground" },
            },
            {
                { Name = "Ground" },
                { Name = "Ground" },
            },
        },
        voxelSubsampleOffsets = offsets,
        sampleInterpolatedHeight = function(cellX, cellZ)
            if cellX == 1 and cellZ == 1 then
                return 3
            end
            return 0
        end,
    }

    local moderateProfile = TerrainBuilder._sampleVoxelColumnProfile(moderateProfilePlan, 1, 1)
    Assert.equal(
        moderateProfile.averageHeight,
        0.75,
        "expected moderate mixed voxels to retain average-height telemetry"
    )
    Assert.equal(
        moderateProfile.heightRange,
        3,
        "expected moderate mixed voxels to capture sub-threshold height spread"
    )
    Assert.equal(
        moderateProfile.peakSampleCoverage,
        0.25,
        "expected moderate mixed voxels to retain peak-coverage telemetry"
    )
    Assert.truthy(
        moderateProfile.surfaceHeight > moderateProfile.averageHeight,
        "expected moderate mixed voxels to bias the surface above the simple average instead of flattening back to it"
    )
    Assert.truthy(
        moderateProfile.surfaceHeight < 3,
        "expected moderate mixed voxels not to snap all the way to the local maximum when the peak occupies only one quadrant"
    )
    Assert.truthy(
        moderateProfile.surfaceFillDepth < 8,
        "expected moderate mixed voxels to reduce fill depth instead of keeping a full terrain slab"
    )
    Assert.truthy(
        peakProfile.edgeOccupancyScale < moderateProfile.edgeOccupancyScale,
        "expected steeper mixed voxels to taper more aggressively than milder mixed voxels"
    )

    local sparsePeakProfilePlan = {
        origin = { x = 0, y = 0, z = 0 },
        cellSize = 4,
        gridW = 1,
        gridD = 1,
        rMinX = 0,
        rMinZ = 0,
        cellMaterials = {
            {
                { Name = "Rock" },
            },
        },
        voxelSubsampleOffsets = offsets,
        sampleInterpolatedHeight = function(cellX, cellZ, fracX, fracZ)
            if cellX == 0 and cellZ == 0 and fracX > 0.75 and fracZ > 0.75 then
                return 16
            end
            return 0
        end,
    }

    local sparsePeakProfile = TerrainBuilder._sampleVoxelColumnProfile(sparsePeakProfilePlan, 1, 1)
    Assert.equal(sparsePeakProfile.peakSampleCoverage, 0.0625, "expected sparse peak coverage to stay measurable")
    Assert.truthy(
        sparsePeakProfile.surfaceHeight > sparsePeakProfile.averageHeight,
        "expected sparse steep peaks to stay above the surrounding average height"
    )
    Assert.truthy(
        sparsePeakProfile.surfaceHeight < 1.05,
        "expected sparse steep peaks to stay very close to the surrounding surface instead of forming a false elevated plane"
    )

    local sparseCliffProfilePlan = {
        origin = { x = 0, y = 0, z = 0 },
        cellSize = 4,
        gridW = 1,
        gridD = 1,
        rMinX = 0,
        rMinZ = 0,
        cellMaterials = {
            {
                { Name = "Rock" },
            },
        },
        voxelSubsampleOffsets = offsets,
        sampleInterpolatedHeight = function(cellX, cellZ, fracX, fracZ)
            if cellX == 0 and cellZ == 0 and fracX > 0.75 and fracZ > 0.75 then
                return 16
            end
            return 0
        end,
    }

    local sparseCliffProfile = TerrainBuilder._sampleVoxelColumnProfile(sparseCliffProfilePlan, 1, 1)
    Assert.truthy(
        sparseCliffProfile.sparseCliffOccupancyScale < 0.5,
        "expected sparse cliff columns to lower their interior occupancy so the column does not read like a solid plane"
    )
    Assert.truthy(
        sparseCliffProfile.sparseCliffOccupancyScale <= 0.2,
        "expected very sparse cliff columns to clamp down to a much lighter occupancy floor instead of leaving a broad vertical slab"
    )
    Assert.truthy(
        sparseCliffProfile.sparseCliffOccupancyScale < peakProfile.sparseCliffOccupancyScale,
        "expected sparser cliffs to taper occupancy more aggressively than denser steep peaks"
    )
end
