local ReplicatedStorage = game:GetService("ReplicatedStorage")

local ChunkSchema = require(ReplicatedStorage.Shared.ChunkSchema)
local ManifestLoader = require(script.Parent.Parent.ImportService.ManifestLoader)
local Assert = require(script.Parent.Assert)

return function()
    local manifest = ManifestLoader.LoadNamedSample("SampleManifest")
    local validated = ChunkSchema.validateManifest(manifest)
    Assert.equal(validated.schemaVersion, "0.4.0", "expected current schema version")
    Assert.equal(#validated.chunks, 1, "expected one sample chunk")

    local function expectRejectedSchemaVersion(schemaVersion)
        local oldManifest = {
            schemaVersion = schemaVersion,
            meta = {
                worldName = "Test",
                generator = "test",
                source = "test",
                metersPerStud = 1,
                chunkSizeStuds = 256,
                bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            },
            chunks = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    roads = {},
                    rails = {},
                    buildings = {},
                    water = {},
                    props = {},
                },
            },
        }

        local ok, err = pcall(function()
            ChunkSchema.validateManifest(oldManifest)
        end)
        Assert.falsy(ok, "expected schemaVersion " .. schemaVersion .. " to be rejected")
        Assert.truthy(
            tostring(err):find("0.4.0", 1, true) ~= nil or tostring(err):find("schemaVersion", 1, true) ~= nil,
            "expected rejection error to mention schema version"
        )
    end

    expectRejectedSchemaVersion("0.1.0")
    expectRejectedSchemaVersion("0.2.0")
    expectRejectedSchemaVersion("0.3.0")

    local missingSchemaVersion = {
        meta = {
            worldName = "Test",
            generator = "test",
            source = "test",
            metersPerStud = 1,
            chunkSizeStuds = 256,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
        },
        chunks = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                roads = {},
                rails = {},
                buildings = {},
                water = {},
                props = {},
            },
        },
    }
    local okMissingSchemaVersion, errMissingSchemaVersion = pcall(function()
        ChunkSchema.validateManifest(missingSchemaVersion)
    end)
    Assert.falsy(okMissingSchemaVersion, "expected missing schemaVersion to fail validation")
    Assert.truthy(
        tostring(errMissingSchemaVersion):find("schemaVersion", 1, true) ~= nil,
        "expected missing schemaVersion error to mention schemaVersion"
    )

    local badManifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "Test",
            generator = "test",
            source = "test",
            metersPerStud = 0.3,
            chunkSizeStuds = 256,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            totalFeatures = 0,
        },
        chunks = {},
    }
    local ok = pcall(function()
        ChunkSchema.validateManifest(badManifest)
    end)
    Assert.falsy(ok, "expected empty chunk list to fail validation")

    local manifestWithChunkRefs = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "Test",
            generator = "test",
            source = "test",
            metersPerStud = 0.3,
            chunkSizeStuds = 256,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            totalFeatures = 1,
        },
        chunkRefs = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                partitionVersion = "subplans.v1",
                subplans = {
                    {
                        id = "terrain",
                        layer = "terrain",
                        featureCount = 1,
                        streamingCost = 8,
                        bounds = {
                            minX = 0,
                            minY = 0,
                            maxX = 128,
                            maxY = 128,
                        },
                    },
                },
            },
        },
        chunks = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                roads = {},
                rails = {},
                buildings = {},
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }
    local validatedWithChunkRefs = ChunkSchema.validateManifest(manifestWithChunkRefs)
    Assert.equal(
        validatedWithChunkRefs.chunkRefs[1].partitionVersion,
        "subplans.v1",
        "expected valid chunkRefs to pass"
    )

    local manifestWithRoofIncluded = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "Test",
            generator = "test",
            source = "test",
            metersPerStud = 0.3,
            chunkSizeStuds = 256,
            bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            totalFeatures = 1,
        },
        chunks = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                roads = {},
                rails = {},
                buildings = {
                    {
                        id = "bldg_1",
                        material = "Concrete",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 4, z = 0 },
                            { x = 4, z = 4 },
                        },
                        baseY = 0,
                        height = 12,
                        roof = "gabled",
                        roofIncluded = true,
                    },
                },
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }
    local validatedWithRoofIncluded = ChunkSchema.validateManifest(manifestWithRoofIncluded)
    Assert.equal(
        validatedWithRoofIncluded.chunks[1].buildings[1].roofIncluded,
        true,
        "expected optional roofIncluded flag to validate when present"
    )

    local missingPartitionVersion = table.clone(manifestWithChunkRefs)
    missingPartitionVersion.chunkRefs = {
        table.clone(manifestWithChunkRefs.chunkRefs[1]),
    }
    missingPartitionVersion.chunkRefs[1].partitionVersion = nil
    local okMissingPartitionVersion = pcall(function()
        ChunkSchema.validateManifest(missingPartitionVersion)
    end)
    Assert.falsy(
        okMissingPartitionVersion,
        "expected subplans without partitionVersion to fail validation"
    )

    local malformedChunkRef = table.clone(manifestWithChunkRefs)
    malformedChunkRef.chunkRefs = {
        table.clone(manifestWithChunkRefs.chunkRefs[1]),
    }
    malformedChunkRef.chunkRefs[1].featureCount = "many"
    local okMalformedChunkRef = pcall(function()
        ChunkSchema.validateManifest(malformedChunkRef)
    end)
    Assert.falsy(okMalformedChunkRef, "expected malformed chunkRef metadata to fail validation")

    local fractionalChunkRef = table.clone(manifestWithChunkRefs)
    fractionalChunkRef.chunkRefs = {
        table.clone(manifestWithChunkRefs.chunkRefs[1]),
    }
    fractionalChunkRef.chunkRefs[1].featureCount = 1.5
    local okFractionalChunkRef = pcall(function()
        ChunkSchema.validateManifest(fractionalChunkRef)
    end)
    Assert.falsy(
        okFractionalChunkRef,
        "expected fractional chunkRef featureCount to fail validation"
    )

    local malformedSubplan = table.clone(manifestWithChunkRefs)
    malformedSubplan.chunkRefs = {
        table.clone(manifestWithChunkRefs.chunkRefs[1]),
    }
    malformedSubplan.chunkRefs[1].subplans = {
        table.clone(manifestWithChunkRefs.chunkRefs[1].subplans[1]),
    }
    malformedSubplan.chunkRefs[1].subplans[1].bounds = {
        minX = 0,
        maxX = 128,
        maxY = 128,
    }
    local okMalformedSubplan = pcall(function()
        ChunkSchema.validateManifest(malformedSubplan)
    end)
    Assert.falsy(okMalformedSubplan, "expected malformed subplan bounds to fail validation")

    local fractionalSubplan = table.clone(manifestWithChunkRefs)
    fractionalSubplan.chunkRefs = {
        table.clone(manifestWithChunkRefs.chunkRefs[1]),
    }
    fractionalSubplan.chunkRefs[1].subplans = {
        table.clone(manifestWithChunkRefs.chunkRefs[1].subplans[1]),
    }
    fractionalSubplan.chunkRefs[1].subplans[1].featureCount = 1.5
    local okFractionalSubplan = pcall(function()
        ChunkSchema.validateManifest(fractionalSubplan)
    end)
    Assert.falsy(okFractionalSubplan, "expected fractional subplan featureCount to fail validation")
end
