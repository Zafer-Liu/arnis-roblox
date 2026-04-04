return function()
    local ManifestLoader = require(script.Parent.Parent.ImportService.ManifestLoader)
    local Assert = require(script.Parent.Assert)
    local ServerStorage = game:GetService("ServerStorage")

    local container = Instance.new("Folder")
    container.Name = "ManifestSubplansSpecTemp"
    container.Parent = script

    local shardFolder = Instance.new("Folder")
    shardFolder.Name = "ShardFolder"
    shardFolder.Parent = container

    local shardModule = Instance.new("ModuleScript")
    shardModule.Name = "TestShard_001"
    shardModule.Source = [[
        game:SetAttribute(
            "ManifestSubplansShardRequireCount",
            (game:GetAttribute("ManifestSubplansShardRequireCount") or 0) + 1
        )
        return {
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
                {
                    id = "1_0",
                    originStuds = { x = 256, y = 0, z = 0 },
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
    ]]
    shardModule.Parent = shardFolder

    local indexModule = Instance.new("ModuleScript")
    indexModule.Name = "ManifestSubplansIndex"
    indexModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "ManifestSubplans",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 2,
            },
            shardFolder = "ManifestSubplansChunks",
            shards = { "TestShard_001" },
            chunkRefs = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    shards = { "StaleShard_999" },
                    featureCount = 1,
                    streamingCost = 8,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1,
                            streamingCost = 8,
                        },
                    },
                },
            },
        }
    ]]
    indexModule.Parent = container

    local sampleData = ServerStorage:FindFirstChild("SampleData")
    local createdSampleData = false
    if not sampleData then
        sampleData = Instance.new("Folder")
        sampleData.Name = "SampleData"
        sampleData.Parent = ServerStorage
        createdSampleData = true
    end
    sampleData:SetAttribute("VertigoSyncEditPreviewIgnore", true)

    local sampleShardFolder = Instance.new("Folder")
    sampleShardFolder.Name = "ManifestSubplansChunks"
    sampleShardFolder.Parent = sampleData

    local sampleShardModule = shardModule:Clone()
    sampleShardModule.Parent = sampleShardFolder

    local sampleIndexModule = indexModule:Clone()
    sampleIndexModule.Parent = sampleData

    local malformedIndexModule = Instance.new("ModuleScript")
    malformedIndexModule.Name = "ManifestSubplansMalformedIndex"
    malformedIndexModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "ManifestSubplansMalformed",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 1,
            },
            shardFolder = "ManifestSubplansChunks",
            shards = { "TestShard_001" },
            chunkRefs = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    featureCount = 1.5,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1.5,
                            streamingCost = 8,
                        },
                    },
                },
            },
        }
    ]]
    malformedIndexModule.Parent = container

    local seededIndexModule = Instance.new("ModuleScript")
    seededIndexModule.Name = "ManifestSubplansSeededIndex"
    seededIndexModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "ManifestSubplansSeeded",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 2,
            },
            shardFolder = "ManifestSubplansChunks",
            shards = { "TestShard_001" },
            chunkRefs = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    shards = { "TestShard_001" },
                    featureCount = 1,
                    streamingCost = 8,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1,
                            streamingCost = 8,
                        },
                    },
                },
            },
        }
    ]]
    seededIndexModule.Parent = container

    local splitTerrainShardBase = Instance.new("ModuleScript")
    splitTerrainShardBase.Name = "SplitTerrainShard_001"
    splitTerrainShardBase.Source = [[
        return {
            chunks = {
                {
                    id = "2_0",
                    originStuds = { x = 512, y = 0, z = 0 },
                    terrain = {
                        cellSizeStuds = 4,
                        width = 2,
                        depth = 2,
                        material = "Grass",
                    },
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
    ]]
    splitTerrainShardBase.Parent = shardFolder

    local splitTerrainShardHeights = Instance.new("ModuleScript")
    splitTerrainShardHeights.Name = "SplitTerrainShard_002"
    splitTerrainShardHeights.Source = [[
        return {
            chunks = {
                {
                    id = "2_0",
                    terrain = {
                        heights = { 1, 2 },
                    },
                },
            },
        }
    ]]
    splitTerrainShardHeights.Parent = shardFolder

    local splitTerrainShardMaterials = Instance.new("ModuleScript")
    splitTerrainShardMaterials.Name = "SplitTerrainShard_003"
    splitTerrainShardMaterials.Source = [[
        return {
            chunks = {
                {
                    id = "2_0",
                    terrain = {
                        heights = { 3, 4 },
                        materials = { "Grass", "Grass", "Rock", "Rock" },
                    },
                },
            },
        }
    ]]
    splitTerrainShardMaterials.Parent = shardFolder

    local splitTerrainIndexModule = Instance.new("ModuleScript")
    splitTerrainIndexModule.Name = "ManifestSubplansSplitTerrainIndex"
    splitTerrainIndexModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "ManifestSubplansSplitTerrain",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 1,
            },
            shardFolder = "ManifestSubplansChunks",
            shards = { "SplitTerrainShard_001", "SplitTerrainShard_002", "SplitTerrainShard_003" },
            chunkRefs = {
                {
                    id = "2_0",
                    originStuds = { x = 512, y = 0, z = 0 },
                    featureCount = 1,
                    streamingCost = 8,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1,
                            streamingCost = 8,
                        },
                    },
                    shards = { "SplitTerrainShard_001", "SplitTerrainShard_002", "SplitTerrainShard_003" },
                },
            },
        }
    ]]
    splitTerrainIndexModule.Parent = container

    local routeBundleFolder = Instance.new("Folder")
    routeBundleFolder.Name = "RouteBundle"
    routeBundleFolder.Parent = container

    local routeSessionModule = Instance.new("ModuleScript")
    routeSessionModule.Name = "route-session"
    routeSessionModule.Source = [[
        return {
            planetary_store_path = "/tmp/planetary.sqlite",
            radius_studs = 300,
            steps = {
                {
                    step_index = 0,
                    focus_lat = 30.265,
                    focus_lon = -97.749,
                    plan = {
                        scene_id = "austin",
                        chunk_ids = { "0_0" },
                    },
                },
            },
        }
    ]]
    routeSessionModule.Parent = routeBundleFolder

    local hydratedRouteModule = Instance.new("ModuleScript")
    hydratedRouteModule.Name = "hydrated-route"
    hydratedRouteModule.Source = [[
        return {
            session = {
                planetary_store_path = "/tmp/planetary.sqlite",
            },
            steps = {
                {
                    step_index = 0,
                    entering = {
                        chunk_refs = {
                            {
                                scene_id = "austin",
                                chunk_id = "0_0",
                            },
                        },
                    },
                    retained = {
                        chunk_refs = {
                            {
                                scene_id = "austin",
                                chunk_id = "0_0",
                            },
                        },
                    },
                    leaving = {
                        chunk_refs = {},
                    },
                },
            },
        }
    ]]
    hydratedRouteModule.Parent = routeBundleFolder

    local routeScheduleModule = Instance.new("ModuleScript")
    routeScheduleModule.Name = "route-schedule"
    routeScheduleModule.Source = [[
        return {
            session = {
                planetary_store_path = "/tmp/planetary.sqlite",
            },
            steps = {
                {
                    step_index = 0,
                    active = {
                        chunk_refs = {
                            {
                                scene_id = "austin",
                                chunk_id = "0_0",
                            },
                        },
                        chunk_ids = { "0_0" },
                    },
                    prefetch = {
                        chunk_refs = {},
                        chunk_ids = {},
                    },
                    retain = {
                        chunk_refs = {
                            {
                                scene_id = "austin",
                                chunk_id = "0_0",
                            },
                        },
                        chunk_ids = { "0_0" },
                    },
                },
            },
        }
    ]]
    routeScheduleModule.Parent = routeBundleFolder

    local routeLaneFolder = Instance.new("Folder")
    routeLaneFolder.Name = "route-lanes"
    routeLaneFolder.Parent = routeBundleFolder

    local routeLaneModule = Instance.new("ModuleScript")
    routeLaneModule.Name = "step-000-active"
    routeLaneModule.Source = [[
        return {
            chunk_refs = {
                {
                    scene_id = "austin",
                    chunk_id = "0_0",
                },
            },
            chunk_ids = { "0_0" },
        }
    ]]
    routeLaneModule.Parent = routeLaneFolder

    local routeManifestFolder = Instance.new("Folder")
    routeManifestFolder.Name = "route-manifests"
    routeManifestFolder.Parent = routeBundleFolder

    local routeManifestModule = Instance.new("ModuleScript")
    routeManifestModule.Name = "step-000-active-manifest"
    routeManifestModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "RouteBundleManifest",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 1,
            },
            chunkRefs = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    shards = { "TestShard_001" },
                    featureCount = 1,
                    streamingCost = 8,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1,
                            streamingCost = 8,
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
    ]]
    routeManifestModule.Parent = routeManifestFolder

    local routeRuntimeFolder = Instance.new("Folder")
    routeRuntimeFolder.Name = "route-runtime"
    routeRuntimeFolder.Parent = routeBundleFolder

    local routeRuntimeStepFolder = Instance.new("Folder")
    routeRuntimeStepFolder.Name = "step-000-active"
    routeRuntimeStepFolder.Parent = routeRuntimeFolder

    local routeRuntimeShardFolder = Instance.new("Folder")
    routeRuntimeShardFolder.Name = "PlanetaryChunks"
    routeRuntimeShardFolder.Parent = routeRuntimeStepFolder

    local routeRuntimeShardModule = shardModule:Clone()
    routeRuntimeShardModule.Parent = routeRuntimeShardFolder

    local routeRuntimeIndexModule = Instance.new("ModuleScript")
    routeRuntimeIndexModule.Name = "PlanetaryIndex"
    routeRuntimeIndexModule.Source = [[
        return {
            schemaVersion = "0.4.0",
            meta = {
                worldName = "RouteBundleRuntime",
                generator = "test",
                source = "test",
                metersPerStud = 0.3,
                chunkSizeStuds = 256,
                bbox = {
                    minLat = 0,
                    minLon = 0,
                    maxLat = 1,
                    maxLon = 1,
                },
                totalFeatures = 1,
            },
            shardFolder = "PlanetaryChunks",
            shards = { "TestShard_001" },
            chunkRefs = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    shards = { "TestShard_001" },
                    featureCount = 1,
                    streamingCost = 8,
                    partitionVersion = "subplans.v1",
                    subplans = {
                        {
                            id = "terrain",
                            layer = "terrain",
                            featureCount = 1,
                            streamingCost = 8,
                        },
                    },
                },
            },
        }
    ]]
    routeRuntimeIndexModule.Parent = routeRuntimeStepFolder

    local routeCatalogModule = Instance.new("ModuleScript")
    routeCatalogModule.Name = "route-catalog"
    routeCatalogModule.Source = [[
        return {
            step_count = 1,
            schedule_out = "route-schedule.json",
            route_session_module_path = "route-session",
            hydrated_route_module_path = "hydrated-route",
            schedule_module_path = "route-schedule",
            lane_dir = "route-lanes",
            payload_dir = "route-payloads",
            manifest_dir = "route-manifests",
            runtime_dir = "route-runtime",
            payloads = {
                {
                    step_index = 0,
                    lane = "active",
                    chunk_count = 1,
                    materializable = true,
                    scene_id = "austin",
                    lane_module_path = "route-lanes/step-000-active",
                    manifest_module_path = "route-manifests/step-000-active-manifest",
                    runtime_index_module_path = "route-runtime/step-000-active/PlanetaryIndex",
                },
                {
                    step_index = 0,
                    lane = "prefetch",
                    chunk_count = 0,
                    materializable = false,
                    reason = "cross-scene lane",
                },
            },
        }
    ]]
    routeCatalogModule.Parent = routeBundleFolder

    local sampleRouteSessionModule = routeSessionModule:Clone()
    sampleRouteSessionModule.Parent = sampleData

    local sampleHydratedRouteModule = hydratedRouteModule:Clone()
    sampleHydratedRouteModule.Parent = sampleData

    local sampleRouteScheduleModule = routeScheduleModule:Clone()
    sampleRouteScheduleModule.Parent = sampleData

    local sampleRouteLaneFolder = routeLaneFolder:Clone()
    sampleRouteLaneFolder.Parent = sampleData

    local sampleRouteManifestFolder = routeManifestFolder:Clone()
    sampleRouteManifestFolder.Parent = sampleData

    local sampleRouteRuntimeFolder = routeRuntimeFolder:Clone()
    sampleRouteRuntimeFolder.Parent = sampleData

    local sampleRouteCatalogModule = routeCatalogModule:Clone()
    sampleRouteCatalogModule.Name = "ManifestRouteCatalog"
    sampleRouteCatalogModule.Parent = sampleData

    game:SetAttribute("ManifestSubplansShardRequireCount", 0)

    local ok, err = xpcall(function()
        local handle = ManifestLoader.LoadShardedModuleHandle(indexModule, shardFolder, 0, {
            freshRequire = true,
        })

        Assert.equal(#handle.chunkRefs, 1, "expected one chunk ref")
        Assert.equal(
            handle.chunkRefs[1].partitionVersion,
            "subplans.v1",
            "expected partitionVersion to survive sharded handle load"
        )
        Assert.truthy(type(handle.chunkRefs[1].subplans) == "table", "expected subplans table on loaded chunk ref")
        Assert.equal(
            handle.chunkRefs[1].subplans[1].id,
            "terrain",
            "expected terrain subplan metadata to survive sharded handle load"
        )
        Assert.equal(
            handle.chunkRefs[1].shards[1],
            "StaleShard_999",
            "expected handle creation to keep additive shard metadata without eagerly scanning shard modules"
        )
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            0,
            "expected sharded handle creation to remain lazy when chunkRefs metadata is present"
        )

        local boundedChunkIds = handle:GetChunkIdsWithinRadius(Vector3.new(128, 0, 128), 32)
        Assert.equal(
            table.concat(boundedChunkIds, ","),
            "0_0",
            "expected bounded radius queries to stay seed-backed instead of forcing canonical full enumeration"
        )
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            0,
            "expected bounded radius queries to remain lazy when seed chunkRefs are present"
        )

        game:SetAttribute("ManifestSubplansShardRequireCount", 0)
        local directHandle = ManifestLoader.LoadShardedModuleHandle(indexModule, shardFolder, 0, {
            freshRequire = true,
        })
        local directChunk = directHandle:GetChunk("0_0")
        Assert.equal(
            directChunk.id,
            "0_0",
            "expected stale seed shard names to fall back to canonical index shards for direct chunk loads"
        )
        Assert.equal(
            directHandle.chunkRefs[1].shards[1],
            "TestShard_001",
            "expected direct chunk load to repair stale seed shard names instead of failing early"
        )
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            1,
            "expected direct chunk fallback to require only the canonical shard on demand"
        )

        game:SetAttribute("ManifestSubplansShardRequireCount", 0)
        local seededHandle = ManifestLoader.LoadShardedModuleHandle(seededIndexModule, shardFolder, 0, {
            freshRequire = true,
        })
        local seededChunkRef = seededHandle:ResolveChunkRef("0_0")
        Assert.equal(
            seededChunkRef.shards[1],
            "TestShard_001",
            "expected seeded chunk refs to preserve authoritative shard metadata"
        )
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            0,
            "expected ResolveChunkRef to stay lazy when seed shard metadata is already authoritative"
        )

        local splitTerrainHandle = ManifestLoader.LoadShardedModuleHandle(splitTerrainIndexModule, shardFolder, 0, {
            freshRequire = true,
        })
        local splitTerrainChunk = splitTerrainHandle:GetChunk("2_0")
        Assert.equal(
            #splitTerrainChunk.terrain.heights,
            4,
            "expected split terrain height fragments to merge back into one terrain grid"
        )
        Assert.equal(
            table.concat(splitTerrainChunk.terrain.heights, ","),
            "1,2,3,4",
            "expected split terrain height fragments to preserve source ordering"
        )
        Assert.equal(
            #splitTerrainChunk.terrain.materials,
            4,
            "expected split terrain material fragments to merge back into one terrain grid"
        )
        Assert.equal(
            table.concat(splitTerrainChunk.terrain.materials, ","),
            "Grass,Grass,Rock,Rock",
            "expected split terrain material fragments to preserve source ordering"
        )

        game:SetAttribute("ManifestSubplansShardRequireCount", 0)
        local allChunkIds = handle:GetChunkIdsWithinRadius(nil, nil)
        Assert.equal(
            table.concat(allChunkIds, ","),
            "0_0,1_0",
            "expected canonical enumeration to include chunks omitted from additive chunkRefs metadata"
        )
        Assert.equal(#handle.chunkRefs, 2, "expected canonical chunk refs to be cached after full enumeration")

        local chunk = handle:GetChunk("0_0")
        Assert.equal(chunk.id, "0_0", "expected rebuilt chunk ref to remain loadable")
        Assert.equal(
            handle.chunkRefs[1].shards[1],
            "TestShard_001",
            "expected canonical shard truth to replace stale shard metadata after chunk materialization"
        )
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            1,
            "expected canonical chunk ref resolution to remain lazy until full enumeration is requested"
        )

        local malformedOk = pcall(function()
            ManifestLoader.LoadShardedModuleHandle(malformedIndexModule, shardFolder, 0, {
                freshRequire = true,
            })
        end)
        Assert.falsy(malformedOk, "expected malformed chunkRefs/subplans metadata to be rejected at handle creation")
        Assert.equal(
            game:GetAttribute("ManifestSubplansShardRequireCount"),
            1,
            "expected malformed handle creation to fail before loading additional shard modules"
        )

        local frozen = ManifestLoader.FreezeHandleForChunkIds(handle, { "0_0" })
        Assert.equal(#frozen.chunkRefs, 1, "expected frozen handle to keep selected chunk refs")
        Assert.equal(
            frozen.chunkRefs[1].partitionVersion,
            "subplans.v1",
            "expected frozen handle to keep partitionVersion"
        )
        Assert.truthy(type(frozen.chunkRefs[1].subplans) == "table", "expected frozen handle to keep subplans table")
        Assert.equal(frozen.chunkRefs[1].subplans[1].id, "terrain", "expected frozen handle to keep subplan metadata")
        Assert.equal(frozen.chunkRefs[1].shards[1], "TestShard_001", "expected frozen handle to keep shard metadata")

        local materializedFromHandle = handle:MaterializeManifest()
        Assert.equal(
            #materializedFromHandle.chunkRefs,
            2,
            "expected materialized handle manifest to include canonical chunk refs"
        )
        Assert.equal(
            #materializedFromHandle.chunks,
            2,
            "expected materialized handle manifest to include chunks omitted from additive chunkRefs metadata"
        )
        Assert.equal(
            materializedFromHandle.chunkRefs[1].partitionVersion,
            "subplans.v1",
            "expected materialized handle manifest to keep partitionVersion"
        )
        Assert.equal(
            materializedFromHandle.chunkRefs[1].subplans[1].id,
            "terrain",
            "expected materialized handle manifest to keep subplans"
        )
        Assert.equal(
            materializedFromHandle.chunkRefs[1].shards[1],
            "TestShard_001",
            "expected materialized handle manifest to keep rebuilt shard metadata"
        )

        local materializedFromIndex = ManifestLoader.LoadFromShardedModuleIndex(indexModule, shardFolder, 0)
        Assert.equal(
            #materializedFromIndex.chunkRefs,
            2,
            "expected direct sharded manifest load to keep canonical chunk refs"
        )
        Assert.equal(
            #materializedFromIndex.chunks,
            2,
            "expected direct sharded manifest load to include all canonical chunks"
        )
        Assert.equal(
            materializedFromIndex.chunkRefs[1].partitionVersion,
            "subplans.v1",
            "expected direct sharded manifest load to keep partitionVersion"
        )
        Assert.equal(
            materializedFromIndex.chunkRefs[1].subplans[1].id,
            "terrain",
            "expected direct sharded manifest load to keep subplans"
        )
        Assert.equal(
            materializedFromIndex.chunkRefs[1].shards[1],
            "TestShard_001",
            "expected direct sharded manifest load to keep canonical shard truth"
        )

        local namedSampleManifest = ManifestLoader.LoadNamedShardedSample("ManifestSubplansIndex", 0)
        Assert.equal(
            #namedSampleManifest.chunkRefs,
            2,
            "expected named sharded sample load to keep canonical chunk refs"
        )
        Assert.equal(
            #namedSampleManifest.chunks,
            2,
            "expected named sharded sample load to include all canonical chunks"
        )
        Assert.equal(
            namedSampleManifest.chunkRefs[1].partitionVersion,
            "subplans.v1",
            "expected named sharded sample load to keep partitionVersion"
        )
        Assert.equal(
            namedSampleManifest.chunkRefs[1].subplans[1].id,
            "terrain",
            "expected named sharded sample load to keep subplans"
        )
        Assert.equal(
            namedSampleManifest.chunkRefs[1].shards[1],
            "TestShard_001",
            "expected named sharded sample load to keep canonical shard truth"
        )

        local routeCatalog = ManifestLoader.LoadRouteCatalogFromModule(routeCatalogModule)
        Assert.equal(routeCatalog.step_count, 1, "expected route catalog to load as a table")
        Assert.equal(
            routeCatalog.route_session_module_path,
            "route-session",
            "expected route catalog to preserve route-session module metadata"
        )
        Assert.equal(
            ManifestLoader.ResolveModuleByPath(routeBundleFolder, "route-lanes/step-000-active", 0).Name,
            "step-000-active",
            "expected route module path resolution to follow route catalog relative paths"
        )

        local namedRouteCatalog = ManifestLoader.LoadNamedRouteCatalog("ManifestRouteCatalog", 0)
        Assert.equal(
            namedRouteCatalog.payloads[1].runtime_index_module_path,
            "route-runtime/step-000-active/PlanetaryIndex",
            "expected named route catalog load to preserve runtime module metadata"
        )

        local routeSession = ManifestLoader.LoadRouteSessionFromCatalogModule(routeCatalogModule, 0)
        Assert.equal(routeSession.steps[1].plan.scene_id, "austin", "expected route session load")

        local hydratedRoute = ManifestLoader.LoadHydratedRouteFromCatalogModule(routeCatalogModule, 0)
        Assert.equal(
            hydratedRoute.steps[1].retained.chunk_refs[1].chunk_id,
            "0_0",
            "expected hydrated route load from route catalog"
        )

        local routeSchedule = ManifestLoader.LoadRouteScheduleFromCatalogModule(routeCatalogModule, 0)
        Assert.equal(
            routeSchedule.steps[1].active.chunk_ids[1],
            "0_0",
            "expected route schedule load from route catalog"
        )

        local routeManifest = ManifestLoader.LoadRouteLaneManifestFromCatalogModule(routeCatalogModule, 0, "active", 0)
        Assert.equal(#routeManifest.chunks, 1, "expected route lane manifest load from route catalog")
        Assert.equal(routeManifest.chunkRefs[1].id, "0_0", "expected route lane manifest to preserve chunk refs")

        local routeHandle = ManifestLoader.LoadRouteLaneRuntimeHandleFromCatalogModule(
            routeCatalogModule,
            0,
            "active",
            0,
            { freshRequire = true }
        )
        Assert.equal(routeHandle.chunkRefs[1].id, "0_0", "expected route lane runtime handle to expose chunk refs")
        Assert.equal(
            routeHandle:GetChunk("0_0").id,
            "0_0",
            "expected route lane runtime handle to materialize chunk data"
        )
        Assert.equal(
            routeHandle:MaterializeManifest().chunks[1].id,
            "0_0",
            "expected route lane runtime handle to materialize manifest truth"
        )

        local missingRuntimeOk = pcall(function()
            ManifestLoader.LoadRouteLaneRuntimeHandleFromCatalogModule(routeCatalogModule, 0, "prefetch", 0)
        end)
        Assert.falsy(missingRuntimeOk, "expected non-materializable route lanes to reject runtime handle loads")
    end, debug.traceback)

    sampleShardFolder:Destroy()
    sampleIndexModule:Destroy()
    malformedIndexModule:Destroy()
    seededIndexModule:Destroy()
    splitTerrainIndexModule:Destroy()
    sampleRouteSessionModule:Destroy()
    sampleHydratedRouteModule:Destroy()
    sampleRouteScheduleModule:Destroy()
    sampleRouteLaneFolder:Destroy()
    sampleRouteManifestFolder:Destroy()
    sampleRouteRuntimeFolder:Destroy()
    sampleRouteCatalogModule:Destroy()
    if createdSampleData then
        sampleData:Destroy()
    end
    game:SetAttribute("ManifestSubplansShardRequireCount", nil)
    container:Destroy()

    if not ok then
        error(err)
    end
end
