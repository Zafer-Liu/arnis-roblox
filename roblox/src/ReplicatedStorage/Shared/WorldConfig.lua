--[[
    WorldConfig — Central configuration for the Arnis HD Pipeline.

    All rendering parameters are configurable here. For open-source users:
    adjust these values based on your hardware capabilities.

    Hardware reference:
    - "insane" preset: M5 Max 36-128GB, RTX 4090+ equivalent
    - "high" preset: M3 Pro 18GB, RTX 3070 equivalent
    - "medium" preset: M1 8GB, GTX 1660 equivalent
]]

local WorldConfig = {
    -- ═══════════════════════════════════════════════════════════════
    -- CHUNK & SCALE
    -- ═══════════════════════════════════════════════════════════════
    ChunkSizeStuds = 256,

    -- ═══════════════════════════════════════════════════════════════
    -- RENDER MODES
    -- ═══════════════════════════════════════════════════════════════
    TerrainMode = "voxel", -- "none" | "debugParts" | "voxel"
    RoadMode = "mesh", -- "none" | "parts" | "mesh" | "hybrid"
    BuildingMode = "shellMesh", -- "none" | "shellParts" | "shellMesh" | "prefab"
    WaterMode = "mesh", -- "none" | "mesh"
    LanduseMode = "fill", -- "none" | "fill"

    -- ═══════════════════════════════════════════════════════════════
    -- TERRAIN FIDELITY
    -- ═══════════════════════════════════════════════════════════════
    VoxelSize = 1, -- studs; 1 = maximum smoothness (4 = fast, 2 = balanced)
    TerrainThickness = 8, -- studs below surface to fill with solid terrain
    SlopeRockThreshold = 1.0, -- rise/run ratio above which terrain becomes Rock (≈45°)
    SlopeGroundThreshold = 0.47, -- rise/run ratio above which terrain becomes Ground (≈25°)
    EnableTerrainSatelliteOverlay = true, -- overlay EditableMesh with satellite texture when available

    -- ═══════════════════════════════════════════════════════════════
    -- BUILDING FIDELITY
    -- ═══════════════════════════════════════════════════════════════
    EnableWindowRendering = true,
    MergeWindowsIntoMesh = true, -- batch window panes into EditableMesh (massive draw call reduction)
    EnableRoomInteriors = true,
    EnableHeroPBR = true,
    WindowSpacing = { -- studs between windows by building usage
        office = 4,
        residential = 6,
        apartments = 6,
        house = 6,
        warehouse = 12,
        industrial = 12,
        default = 8,
    },

    -- ═══════════════════════════════════════════════════════════════
    -- ROAD FIDELITY
    -- ═══════════════════════════════════════════════════════════════
    LaneWidth = 12, -- studs per lane (~3.6m at 0.3 m/stud)
    GroundRoadClearance = 0.75, -- studs above sampled terrain for roads/pathways to avoid burial/z-fighting
    EnableStreetLighting = true,
    StreetLightInterval = 50, -- studs between street lights
    StreetLightRange = 40, -- PointLight range in studs

    -- ═══════════════════════════════════════════════════════════════
    -- WATER FIDELITY
    -- ═══════════════════════════════════════════════════════════════
    WaterCarveDepth = 4, -- studs to carve below water surface

    -- ═══════════════════════════════════════════════════════════════
    -- PROP FIDELITY
    -- ═══════════════════════════════════════════════════════════════
    TreeMetersToStuds = 1 / 0.3, -- conversion factor for real-world tree heights
    EnablePalmRendering = true,

    -- ═══════════════════════════════════════════════════════════════
    -- BUILDING LOD POLICY
    -- ═══════════════════════════════════════════════════════════════
    -- Controls building detail level per streaming ring.
    -- "full"    = all facade elements, windows, rooftop equipment, PBR surfaces
    -- "reduced" = shell walls + roof only, no windows/awnings/facade bands/rooftop equipment/name labels
    -- "minimal" = single bounding-box Part per building, no EditableMesh
    BuildingLodPolicy = {
        NearRingLod = "full",
        MidRingLod = "reduced",
        FarRingLod = "minimal",
    },
    EnableLodReimport = true, -- re-import buildings at higher detail when chunks move to a nearer ring

    -- ═══════════════════════════════════════════════════════════════
    -- STREAMING & LOD
    -- ═══════════════════════════════════════════════════════════════
    StreamingProfile = "local_dev", -- "local_dev" | "production_server"
    StreamingEnabled = false,
    StreamingTargetRadius = 4096,
    HighDetailRadius = 2048,
    StreamingUpdateIntervalSeconds = 0.25,
    StreamingMaxWorkItemsPerUpdate = 4,
    StreamingLookaheadSeconds = 1,
    StreamingMaxLookaheadStuds = 512,
    StreamingImportFrameBudgetSeconds = 1 / 240,
    -- NOTE: These top-level ring defaults are only used when no StreamingProfile
    -- matches. The active profile is "local_dev" which has reduced budgets
    -- (16/24/32 chunks) safe for 8GB tertiary. These larger values are for
    -- high-memory production hosts.
    StreamingRings = {
        near = {
            MaxRadiusStuds = 1024,
            EstimatedBudgetBytes = 1536 * 1024 * 1024,
            MaxChunkCount = 64,
        },
        mid = {
            MaxRadiusStuds = 1536,
            EstimatedBudgetBytes = 1536 * 1024 * 1024,
            MaxChunkCount = 96,
        },
        far = {
            MaxRadiusStuds = 2048,
            EstimatedBudgetBytes = 1024 * 1024 * 1024,
            MaxChunkCount = 128,
        },
    },
    MemoryGuardrails = {
        Enabled = true,
        EstimatedBudgetBytes = 4 * 1024 * 1024 * 1024,
        ResumeBudgetRatio = 0.85,
        CountResidentChunkCost = true,
        CountInFlightCost = true,
        HostProbe = {
            Enabled = false,
            CriticalAvailableBytes = nil,
            CriticalPressureLevel = nil,
        },
    },
    SubplanRollout = {
        Enabled = true,
        AllowedLayers = {},
        AllowedChunkIds = {},
    },
    StreamingProfiles = {
        local_dev = {
            StreamingEnabled = true,
            StreamingTargetRadius = 2048,
            HighDetailRadius = 1024,
            StreamingMaxWorkItemsPerUpdate = 2,
            StreamingLookaheadSeconds = 1,
            StreamingMaxLookaheadStuds = 512,
            StreamingRings = {
                near = {
                    MaxRadiusStuds = 512,
                    EstimatedBudgetBytes = 512 * 1024 * 1024,
                    MaxChunkCount = 16,
                },
                mid = {
                    MaxRadiusStuds = 768,
                    EstimatedBudgetBytes = 512 * 1024 * 1024,
                    MaxChunkCount = 24,
                },
                far = {
                    MaxRadiusStuds = 1024,
                    EstimatedBudgetBytes = 256 * 1024 * 1024,
                    MaxChunkCount = 32,
                },
            },
            MemoryGuardrails = {
                Enabled = true,
                EstimatedBudgetBytes = 2 * 1024 * 1024 * 1024,
                ResumeBudgetRatio = 0.85,
                HostProbe = {
                    Enabled = true,
                    CriticalAvailableBytes = 256 * 1024 * 1024,
                    CriticalPressureLevel = 0.95,
                },
            },
            SubplanRollout = {
                Enabled = true,
                AllowedLayers = {},
                AllowedChunkIds = {},
            },
        },
        production_server = {
            StreamingEnabled = true,
            StreamingTargetRadius = 6144,
            HighDetailRadius = 3072,
            StreamingMaxWorkItemsPerUpdate = 8,
            StreamingLookaheadSeconds = 1.5,
            StreamingMaxLookaheadStuds = 1024,
            StreamingImportFrameBudgetSeconds = 1 / 120,
            StreamingRings = {
                near = {
                    MaxRadiusStuds = 3072,
                    EstimatedBudgetBytes = 3 * 1024 * 1024 * 1024,
                    MaxChunkCount = 128,
                },
                mid = {
                    MaxRadiusStuds = 4608,
                    EstimatedBudgetBytes = 3 * 1024 * 1024 * 1024,
                    MaxChunkCount = 192,
                },
                far = {
                    MaxRadiusStuds = 6144,
                    EstimatedBudgetBytes = 2 * 1024 * 1024 * 1024,
                    MaxChunkCount = 256,
                },
            },
            MemoryGuardrails = {
                Enabled = true,
                EstimatedBudgetBytes = 8 * 1024 * 1024 * 1024,
                ResumeBudgetRatio = 0.9,
                HostProbe = {
                    Enabled = false,
                },
            },
            SubplanRollout = {
                Enabled = true,
                AllowedLayers = {},
                AllowedChunkIds = {},
            },
        },
    },

    -- ═══════════════════════════════════════════════════════════════
    -- ATMOSPHERE & LIGHTING
    -- ═══════════════════════════════════════════════════════════════
    EnableAtmosphere = true, -- set false to skip cinematic lighting setup
    EnableDayNightCycle = true,
    DayNightSpeed = 60, -- 60 = 1 game-day per 24 minutes, 0 = frozen
    DateTime = "2024-06-15T14:00", -- Fixed midday for consistent visual proof; change to "auto" for real-time
    AtmosphereDensity = 0.35, -- additive density applied on top of phase presets (world-scale depth cue)
    AtmosphereOffset = 0.2, -- vertical offset for atmosphere gradient start
    AtmosphereHaze = 0.15, -- additive haze layered on top of phase presets for distance fade

    -- ═══════════════════════════════════════════════════════════════
    -- MINIMAP
    -- ═══════════════════════════════════════════════════════════════
    EnableMinimap = true,
    MinimapRadius = 400, -- world studs visible in minimap
    MinimapSize = 200, -- pixel resolution

    -- ═══════════════════════════════════════════════════════════════
    -- AMBIENT CITY LIFE
    -- ═══════════════════════════════════════════════════════════════
    EnableAmbientLife = true,
    MaxParkedCarsPerChunk = 30,
    MaxNPCsPerChunk = 8,

    -- ═══════════════════════════════════════════════════════════════
    -- DEBUG VISUALIZATION
    -- ═══════════════════════════════════════════════════════════════
    DebugBuildingColors = false, -- walls=RED, roofs=BLUE, floors=YELLOW; makes broken buildings instantly visible

    -- ═══════════════════════════════════════════════════════════════
    -- INSTANCE BUDGETS (set high for powerful hardware)
    -- ═══════════════════════════════════════════════════════════════
    InstanceBudget = {
        MaxPerChunk = 8000,
        MaxPropsPerChunk = 2000,
        MaxWindowsPerChunk = 10000, -- effectively unlimited on M5 Max
    },
}

return WorldConfig
