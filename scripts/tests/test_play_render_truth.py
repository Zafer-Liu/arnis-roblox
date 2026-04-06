from __future__ import annotations

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILDING_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "BuildingBuilder.lua"
SCENE_AUDIT = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "SceneAudit.lua"
ROOM_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "RoomBuilder.lua"
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"
IMPORT_SERVICE = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "init.lua"
HIPPED_ROOF_TRUTH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "HippedRoofTruth.spec.lua"
AUSTIN_PREVIEW_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
IMPORT_SIGNATURES = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ImportSignatures.lua"
STREAMING_SERVICE = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
WORLD_PROBE = ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "WorldProbe.client.lua"
WORLD_PROBE_TERRAIN = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTerrain.lua"
BOOTSTRAP_AUSTIN = ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua"
GABLED_IRREGULAR_FOOTPRINT_TRUTH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "GabledIrregularFootprintTruth.spec.lua"
)
SHELLMESH_COURTYARD_TRUTH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "ShellMeshCourtyardTruth.spec.lua"
)


class PlayRenderTruthTests(unittest.TestCase):
    def test_startup_import_and_streaming_share_signature_source(self) -> None:
        self.assertTrue(
            IMPORT_SIGNATURES.exists(),
            "expected ImportSignatures.lua to exist so startup import and streaming share one signature truth",
        )

        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        streaming_source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("ImportSignatures", import_service_source)
        self.assertIn("ImportSignatures", streaming_source)
        self.assertRegex(
            import_service_source,
            r"configSignature\s*=\s*.*configSignature",
            "expected ImportService to register startup chunks with shared config signatures",
        )
        self.assertRegex(
            import_service_source,
            r"layerSignatures\s*=\s*.*layerSignatures",
            "expected ImportService to register startup chunks with shared layer signatures",
        )
        self.assertRegex(
            streaming_source,
            r"ImportSignatures\.[A-Za-z_]+\(",
            "expected StreamingService to derive signatures from the shared ImportSignatures helper",
        )
        self.assertIn('model:GetAttribute("ArnisChunkId")', streaming_source)
        self.assertIn("coherentEnvelopeReady", streaming_source)
        self.assertIn("coherentEnvelopeSourceId", streaming_source)
        self.assertIn("coherentEnvelopeCandidateCount", streaming_source)
        self.assertIn("local function isStartupEnvelopeReady(envelopeTelemetry)", streaming_source)
        self.assertIn("local function selectStartupEnvelopeTelemetry(candidateTelemetryBySourceId)", streaming_source)
        self.assertIn("local function countStartupEnvelopeCandidates(candidateTelemetryBySourceId)", streaming_source)
        self.assertIn("and coherentEnvelopeReady", streaming_source)

    def test_roof_only_builder_uses_rooftop_base_metadata(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertRegex(
            source,
            r"minHeight",
            "expected BuildingBuilder roof-only path to consult rooftop base metadata when present",
        )

    def test_room_interiors_are_gated_by_config_before_runtime_build(self) -> None:
        source = IMPORT_SERVICE.read_text(encoding="utf-8")

        self.assertIn("config.EnableRoomInteriors", source)
        self.assertRegex(
            source,
            r"if\s+config\.EnableRoomInteriors\s*~=\s*false\s+then[\s\S]*RoomBuilder\.BuildAll",
            "expected runtime import to gate RoomBuilder.BuildAll behind EnableRoomInteriors",
        )

    def test_shell_mesh_rooms_skip_shell_terrain_fill_when_interiors_are_enabled(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertIn("shouldFillTerrainInterior", source)
        self.assertIn("config.EnableRoomInteriors ~= false", source)
        self.assertIn("building.rooms", source)
        self.assertRegex(
            source,
            r"if\s+shouldFillTerrainInterior\(building,\s*config\)\s+then[\s\S]*fillInterior\(",
            "expected shell-mesh roomed buildings to skip shell terrain fill when interiors are enabled",
        )

    def test_shell_mesh_terrain_fill_keeps_edge_clearance_from_shell_walls(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertIn("INTERIOR_FILL_EDGE_CLEARANCE", source)
        self.assertIn("distanceToPolygonEdges2D", source)
        self.assertIn("distanceToPolygonWithHoleEdges2D", source)
        self.assertRegex(
            source,
            r"distanceToPolygonWithHoleEdges2D\(x,\s*z,\s*footprintXZ,\s*holeXZ\)\s*>=\s*INTERIOR_FILL_EDGE_CLEARANCE",
            "expected shell terrain fill to keep a bounded edge clearance from shell walls",
        )

    def test_top_floor_room_ceilings_clamp_to_imported_building_top(self) -> None:
        source = ROOM_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local rawCeilingCenterY = buildingBaseY + (room.floorY or 0) + floorHeight", source)
        self.assertIn("ceilingCenterY = math.min(rawCeilingCenterY, buildingTopY - ceilingThickness * 0.5)", source)
        self.assertIn('local buildingTopY = buildingModel:GetAttribute("ArnisImportBuildingTopY")', source)

    def test_terrain_plan_preserves_requested_sampling_intent_while_staying_on_roblox_write_resolution(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local REQUESTED_SAMPLE_RESOLUTION = WorldConfig.VoxelSize or 1", source)
        self.assertIn("requestedSampleResolution = REQUESTED_SAMPLE_RESOLUTION", source)
        self.assertIn("writeResolution = TERRAIN_WRITE_RESOLUTION", source)
        self.assertIn("-- Roblox terrain requires a 4-stud write resolution.", source)

    def test_sub_4_stud_terrain_materials_quantize_to_center_owning_source_cells(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local function buildSubsampleOffsets(writeResolution, requestedSampleResolution)", source)
        self.assertIn("local function sampleVoxelColumnProfile(plan, ix, globalIz)", source)
        self.assertIn("voxelSubsampleOffsets = voxelSubsampleOffsets", source)
        self.assertIn("sampleVoxelColumnProfile = sampleVoxelColumnProfile", source)
        self.assertIn("dominantMaterialName", source)
        self.assertIn("averageHeight = totalHeight / sampleCount", source)
        self.assertIn("heightRange = maxHeight - minHeight", source)
        self.assertIn("local peakSampleCount = 0", source)
        self.assertIn("local peakSampleCoverage = peakSampleCount / sampleCount", source)
        self.assertIn("local normalizedPeakCoverage =", source)
        self.assertIn(
            "local heightRangeFactor = math.clamp(heightRange / TERRAIN_WRITE_RESOLUTION, 0, 1)",
            source,
        )
        self.assertIn("local peakCoverageBias = math.max(peakSampleCoverage, normalizedPeakCoverage)", source)
        self.assertIn(
            "local surfaceHeightBias = heightRangeFactor * peakCoverageBias",
            source,
        )
        self.assertIn("surfaceHeight = averageHeight + (maxHeight - averageHeight) * surfaceHeightBias", source)
        self.assertIn("local sparsePeakPlaneDamping =", source)
        self.assertRegex(
            source,
            r"surfaceFillDepth = if heightRange > 0\s+then math\.max\(1, TERRAIN_THICKNESS \* math\.clamp\(normalizedPeakCoverage \+ peakCoverageBias \* 0\.25, 0, 1\)\)",
        )
        self.assertIn("* sparsePeakPlaneDamping", source)
        self.assertIn("normalizedPeakCoverage", source)
        self.assertIn("local worldBotY = worldSurfY - columnProfile.surfaceFillDepth", source)
        self.assertIn("local worldSurfY = origin.y + columnProfile.surfaceHeight", source)

    def test_terrain_builder_supports_neighbor_aware_chunk_edge_sampling(self) -> None:
        terrain_source = TERRAIN_BUILDER.read_text(encoding="utf-8")
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        streaming_source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("local function resolveNeighborHeightSample(terrainGrid, terrainNeighbors, gridW, gridD, cellX, cellZ)", terrain_source)
        self.assertIn("local function buildTerrainNeighborContextByChunkId(chunks)", import_service_source)
        self.assertIn("local function buildTerrainNeighborContextSignature(neighbors)", import_service_source)
        self.assertIn("terrainNeighborContext = terrainNeighborContextByChunkId[chunk.id]", import_service_source)
        self.assertIn("perChunkOptions.terrainNeighbors = terrainNeighborContext.neighbors", import_service_source)
        self.assertIn("perChunkOptions.terrainNeighborSignature = terrainNeighborContext.signature", import_service_source)
        self.assertIn("local function buildStreamingTerrainNeighborContext(chunkRef)", streaming_source)
        self.assertIn("local function buildStreamingTerrainNeighborContextSignature(neighbors)", streaming_source)
        self.assertIn("importOptions.terrainNeighbors = terrainNeighborContext.neighbors", streaming_source)
        self.assertIn("importOptions.terrainNeighborSignature = terrainNeighborContext.signature", streaming_source)

    def test_terrain_plan_cache_derives_neighbor_signature_from_neighbor_context(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local function buildDerivedTerrainNeighborSignature(terrainNeighbors)", source)
        self.assertIn("local function buildTerrainNeighborDescriptorSignature(direction, descriptor)", source)
        self.assertIn("local terrainIdentityToken = tostring(neighborTerrain)", source)
        self.assertIn(
            'local heightsIdentityToken = if type(neighborTerrain) == "table" then tostring(neighborTerrain.heights) else "none"',
            source,
        )
        self.assertIn("local function resolveTerrainNeighborSignature(options)", source)
        self.assertIn("terrainNeighborSignature = resolveTerrainNeighborSignature(options)", source)
        self.assertIn("options == nil", source)
        self.assertIn("return cachedPlan", source)

    def test_terrain_material_richness_flows_from_builder_to_preview_hotspot_summary(self) -> None:
        terrain_source = TERRAIN_BUILDER.read_text(encoding="utf-8")
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        preview_builder_source = AUSTIN_PREVIEW_BUILDER.read_text(encoding="utf-8")

        self.assertIn("terrainStats", terrain_source)
        self.assertIn("materialKindCount", terrain_source)
        self.assertIn("dominantMaterial", terrain_source)
        self.assertIn("dominantMaterialCellCount", terrain_source)
        self.assertIn("nonGrassCellCount", terrain_source)

        self.assertIn("terrainMaterialKindCount", import_service_source)
        self.assertIn("terrainDominantMaterial", import_service_source)
        self.assertIn("terrainDominantMaterialCellCount", import_service_source)
        self.assertIn("terrainNonGrassCellCount", import_service_source)

        self.assertIn("terrainMaterialKindCount", preview_builder_source)
        self.assertIn("terrainDominantMaterial", preview_builder_source)
        self.assertIn("terrainDominantMaterialCellCount", preview_builder_source)
        self.assertIn("terrainNonGrassCellCount", preview_builder_source)

    def test_play_bootstrap_reuses_shared_world_state_application(self) -> None:
        source = BOOTSTRAP_AUSTIN.read_text(encoding="utf-8")

        self.assertIn(
            "local WorldStateApplier = require(script.Parent.ImportService.WorldStateApplier)",
            source,
        )
        self.assertIn("WorldStateApplier.Apply(", source)
        self.assertIn('worldRootName = "GeneratedWorld_Austin"', source)
        self.assertNotIn('Instance.new("Atmosphere")', source)
        self.assertNotIn('Instance.new("BloomEffect")', source)
        self.assertNotIn('Instance.new("SunRaysEffect")', source)
        self.assertNotIn('Instance.new("ColorCorrectionEffect")', source)

    def test_startup_structure_truth_accepts_merged_shell_readable_cues(self) -> None:
        source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("local function isStartupReadableFacadeCuePart(part)", source)
        self.assertIn('or name == "MergedShellWindowPaneCue"', source)
        self.assertIn("local function isStartupRoofCuePart(part)", source)
        self.assertIn('name == "MergedShellRooflineCue"', source)
        self.assertIn('or name == "MergedShellPerimeterCue"', source)
        self.assertIn("nearbyReadableFacadeCueParts", source)
        self.assertIn("coherentEnvelopeNearbyReadableFacadeCueParts", source)
        self.assertIn("local hasDirectWallEnvelope =", source)
        self.assertIn("local hasMergedReadableEnvelope =", source)
        self.assertIn("envelopeTelemetry.nearbyMergedBuildingMeshParts > 0", source)
        self.assertIn("and envelopeTelemetry.nearbyReadableFacadeCueParts > 0", source)
        self.assertIn("local hasRoofEnvelope = envelopeTelemetry.nearbyRoofParts > 0", source)
        self.assertIn("and envelopeTelemetry.overheadRoofParts > 0", source)

    def test_scene_audit_treats_merged_shell_readable_cues_as_visible_wall_evidence(self) -> None:
        source = SCENE_AUDIT.read_text(encoding="utf-8")

        self.assertIn("local function isVisibleWallCuePart(descendant)", source)
        self.assertIn('instance.Name == "MergedShellWindowPaneCue"', source)
        self.assertIn("visibleWallCueParts = 0", source)
        self.assertIn("summary.visibleWallCueParts += 1", source)
        self.assertIn(
            "if structureSummary.visibleShellWallParts > 0 or structureSummary.visibleWallCueParts > 0 then",
            source,
        )

    def test_runtime_import_threads_chunk_ownership_into_layer_folders(self) -> None:
        source = IMPORT_SERVICE.read_text(encoding="utf-8")

        self.assertIn("setImportAuditAttributes(layerFolder, chunk.id, options.importRunId)", source)
        self.assertIn("setImportAuditAttributes(subplanFolder, chunk.id, options.importRunId)", source)
        self.assertIn("setImportAuditAttributes(folder, chunk.id, options.importRunId)", source)

    def test_preview_hotspot_summary_threads_chunk_shape_context(self) -> None:
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        preview_builder_source = AUSTIN_PREVIEW_BUILDER.read_text(encoding="utf-8")
        preview_summary_source = (ROOT / "scripts" / "preview_telemetry_summary.py").read_text(encoding="utf-8")

        self.assertIn("terrainCellCount", import_service_source)
        self.assertIn("terrainSubsampleCount", import_service_source)
        self.assertIn("buildingFeatureCount", import_service_source)

        self.assertIn("terrainCellCount", preview_builder_source)
        self.assertIn("terrainSubsampleCount", preview_builder_source)
        self.assertIn("buildingFeatureCount", preview_builder_source)

        self.assertIn("dominantCostCenter", preview_summary_source)
        self.assertIn("dominantCostRatio", preview_summary_source)
        self.assertIn("terrainSignalStatus", preview_summary_source)

    def test_preview_hotspot_summary_surfaces_building_split_costs(self) -> None:
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        preview_builder_source = AUSTIN_PREVIEW_BUILDER.read_text(encoding="utf-8")
        preview_summary_source = (ROOT / "scripts" / "preview_telemetry_summary.py").read_text(encoding="utf-8")

        self.assertIn("buildingShellDetailMs", import_service_source)
        self.assertIn("buildingInteriorMs", import_service_source)

        self.assertIn("buildingShellDetailMs", preview_builder_source)
        self.assertIn("buildingInteriorMs", preview_builder_source)

        self.assertIn("buildingShellDetailMs", preview_summary_source)
        self.assertIn("buildingInteriorMs", preview_summary_source)

    def test_preview_hotspot_summary_surfaces_building_detail_subphases(self) -> None:
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        preview_builder_source = AUSTIN_PREVIEW_BUILDER.read_text(encoding="utf-8")
        preview_summary_source = (ROOT / "scripts" / "preview_telemetry_summary.py").read_text(encoding="utf-8")

        for field_name in (
            "buildingRoofBuildMs",
            "buildingFacadeDetailMs",
            "buildingPerimeterDetailMs",
            "buildingTerrainFillMs",
            "buildingRooftopDetailMs",
            "buildingNameLabelMs",
        ):
            self.assertIn(field_name, import_service_source)
            self.assertIn(field_name, preview_builder_source)
            self.assertIn(field_name, preview_summary_source)

        self.assertIn("buildingShellDominantDetailPhase", preview_summary_source)
        self.assertIn("buildingShellDominantDetailMs", preview_summary_source)

    def test_simple_shells_keep_bounded_corner_accents_for_vertical_readability(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertIn('detailFolder:SetAttribute("ArnisCornerAccentCount", 0)', source)
        self.assertIn("local function buildCornerAccents(parent, worldPts, baseY, height)", source)
        self.assertIn("local function addCornerAccentsToAccumulator(acc, worldPts, baseY, height)", source)
        self.assertIn('accent.Name = "CornerAccent"', source)
        self.assertRegex(
            source,
            r"if\s+preferSimpleShellDetail\s+then[\s\S]*ArnisCornerAccentCount",
            "expected bounded corner accents to be limited to the simple-shell detail path",
        )

    def test_shell_mesh_bounded_low_medium_buildings_keep_explicit_shell_walls_for_play_visibility(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")
        scene_audit_source = SCENE_AUDIT.read_text(encoding="utf-8")

        self.assertIn("local function shouldPreferPlayVisibleShellWalls", source)
        self.assertRegex(
            source,
            r"function\s+BuildingBuilder\.MeshBuildAll[\s\S]*local\s+preferPlayVisibleShellWalls\s*=\s*[\s\S]*shouldPreferPlayVisibleShellWalls[\s\S]*if\s+preferPlayVisibleShellWalls\s+then[\s\S]*buildWallLoopParts\(shellFolder,\s*bldgName,\s*worldPts,\s*baseY,\s*height,\s*mat,\s*color,\s*\"outer\"",
            "expected bounded low/medium shellMesh buildings to keep explicit shell wall parts instead of only merged wall meshes",
        )
        self.assertIn('part:SetAttribute("ArnisShellWallEvidence", true)', source)
        self.assertIn('ArnisShellWallEvidence', scene_audit_source)
        self.assertRegex(
            scene_audit_source,
            r"ArnisShellWallEvidence[\s\S]*visibleShellWallParts",
            "expected SceneAudit to count explicit shell wall evidence when classifying visible shell walls",
        )

    def test_shell_mesh_bounded_fallback_expands_explicit_wall_visibility_by_shape_and_size(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local function shouldPreferPlayVisibleShellWalls", source)
        self.assertIn("PLAY_VISIBLE_SHELL_ROOF_SHAPES", source)
        self.assertIn("levels > 6 or height > 34", source)
        self.assertIn("return footprintPointCount <= 10", source)
        self.assertIn("preferPlayVisibleShellWalls", source)
        self.assertRegex(
            source,
            r"if\s+preferPlayVisibleShellWalls\s+then[\s\S]*buildWallLoopParts\(shellFolder,\s*bldgName,\s*worldPts,\s*baseY,\s*height,\s*mat,\s*color,\s*\"outer\"",
            "expected bounded shellMesh fallback to keep explicit shell wall parts for medium-complexity buildings",
        )
        self.assertRegex(
            source,
            r"if\s+preferPlayVisibleShellWalls\s+then[\s\S]*buildPlayVisibleShellReadableCues\(detailFolder,\s*worldPts,\s*baseY,\s*height\)",
            "expected bounded shellMesh fallback to add cheap roofline and beltline readability cues",
        )
        self.assertIn('detailFolder:SetAttribute("ArnisCornerAccentCount", playVisibleCornerAccentCount)', source)
        self.assertIn('detailFolder:SetAttribute("ArnisMergedShellDoorCueCount", playVisibleDoorCueCount)', source)
        self.assertIn('detailFolder:SetAttribute("ArnisMergedShellStreetFacadeCueCount", playVisibleStreetFacadeCueCount)', source)
        self.assertIn('detailFolder:SetAttribute("ArnisMergedShellWindowPaneCueCount", playVisibleWindowPaneCueCount)', source)
        self.assertIn("if boundedHoleLoopCount == 1 then", source)
        self.assertIn("return levels <= 5 and height <= 28 and footprintPointCount <= 12", source)

    def test_shell_mesh_courtyard_truth_spec_exercises_bounded_hole_support(self) -> None:
        spec_source = SHELLMESH_COURTYARD_TRUTH.read_text(encoding="utf-8")

        self.assertIn('BuildingMode = "shellMesh"', spec_source)
        self.assertIn("holes = {", spec_source)
        self.assertIn('shellWallEvidenceCount >= 8', spec_source)
        self.assertIn("expected shellMesh courtyard void to remain open", spec_source)

    def test_scene_audit_surfaces_closure_only_roof_gaps_separately_from_generic_roofless_buildings(self) -> None:
        source = SCENE_AUDIT.read_text(encoding="utf-8")

        self.assertIn('descendant.Name == "MergedShellRooflineCue"', source)
        self.assertIn('descendant.Name == "MergedShellPerimeterCue"', source)
        self.assertIn("roofCueParts = 0", source)
        self.assertIn("local hasMergedRoofCue = roofCueParts > 0", source)
        self.assertIn("buildingModelsWithClosureOnlyRoofGap", source)
        self.assertIn("buildingClosureOnlyRoofGapDetails", source)
        self.assertIn("buildingModelsWithVisibleShellWalls", source)
        self.assertIn("buildingModelsWithoutVisibleShellWalls", source)
        self.assertIn("buildingVisibleDetailPartCount", source)
        self.assertIn("buildingVisibleFacadePartCount", source)
        self.assertIn("buildingRoofCoverageByUsage", source)
        self.assertIn("buildingRoofCoverageByShape", source)
        self.assertRegex(
            source,
            r"roofClosureParts\s*>\s*0[\s\S]*evidenceKind == \"none\"[\s\S]*buildingModelsWithClosureOnlyRoofGap",
            "expected SceneAudit to classify closure-only shaped roofs as a dedicated roof gap instead of burying them in generic no-roof counts",
        )

    def test_world_probe_surfaces_compact_structure_and_terrain_convergence_metrics(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")
        terrain_probe_source = WORLD_PROBE_TERRAIN.read_text(encoding="utf-8")

        self.assertIn("localSupport = nil", world_probe_source)
        self.assertIn("localTerrain = nil", world_probe_source)
        self.assertIn("localEnclosure = nil", world_probe_source)
        self.assertIn("localRoofCover = nil", world_probe_source)
        self.assertIn("sampleCount", terrain_probe_source)
        self.assertIn("coverageRatio", terrain_probe_source)
        self.assertIn("edgeCoverageRatio", terrain_probe_source)
        self.assertIn("convergenceStatus", terrain_probe_source)
        self.assertIn("missingEdgeSampleCount", terrain_probe_source)
        self.assertIn("edgeTerrainYRangeStuds", terrain_probe_source)
        self.assertIn("centerEdgeMaxDeltaStuds", terrain_probe_source)
        self.assertIn("local function countSampleSlots(samples)", terrain_probe_source)

    def test_world_probe_counts_merged_shell_roof_cues_as_local_roof_cover(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")

        self.assertIn("local function isRoofCuePart(part)", world_probe_source)
        self.assertIn('name == "MergedShellRooflineCue"', world_probe_source)
        self.assertIn('or name == "MergedShellPerimeterCue"', world_probe_source)
        self.assertIn("local isRoofCue = isRoofCuePart(descendant)", world_probe_source)
        self.assertIn("if not isRoofPart and not isRoofCue then", world_probe_source)
        self.assertIn("nearbyRoofParts += 1", world_probe_source)
        self.assertIn("overheadRoofParts += 1", world_probe_source)

    def test_world_probe_counts_merged_shell_readable_facade_cues_in_local_enclosure(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")

        self.assertIn("local function isReadableFacadeCuePart(part)", world_probe_source)
        self.assertIn('name == "MergedShellWallPresenceCue"', world_probe_source)
        self.assertIn('or name == "MergedShellStreetFacadeCue"', world_probe_source)
        self.assertIn('or name == "MergedShellWindowPaneCue"', world_probe_source)
        self.assertIn("local nearbyReadableFacadeCueParts = 0", world_probe_source)
        self.assertIn("nearbyReadableFacadeCueParts += 1", world_probe_source)
        self.assertIn("readableFacadeCueParts = nearbyReadableFacadeCueParts", world_probe_source)

    def test_irregular_shaped_roofs_fall_back_to_visible_roof_geometry_not_only_closure_decks(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local function buildFallbackFlatVisibleRoof(", source)
        self.assertRegex(
            source,
            r'if roofShape == "gabled" or roofShape == "gambrel" then[\s\S]*if not rectangularFootprint then[\s\S]*buildFallbackFlatVisibleRoof\(',
            "expected irregular gabled roofs to fall back to visible roof geometry instead of a transparent closure-only deck",
        )
        self.assertRegex(
            source,
            r'elseif roofShape == "pyramidal" or roofShape == "hipped" then[\s\S]*buildFallbackFlatVisibleRoof\(',
            "expected irregular hipped roofs to keep visible roof evidence in fallback mode",
        )

    def test_shellmesh_roof_truth_specs_exercise_shellmesh_mode(self) -> None:
        hipped_source = HIPPED_ROOF_TRUTH.read_text(encoding="utf-8")
        gabled_source = GABLED_IRREGULAR_FOOTPRINT_TRUTH.read_text(encoding="utf-8")

        self.assertIn(
            'BuildingMode = "shellMesh"',
            hipped_source,
            "expected hipped roof truth to use shellMesh play mode",
        )
        self.assertIn(
            'BuildingMode = "shellMesh"',
            gabled_source,
            "expected irregular gabled roof truth to use shellMesh play mode",
        )

    def test_player_local_terrain_telemetry_carries_material_richness(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")
        terrain_probe_source = WORLD_PROBE_TERRAIN.read_text(encoding="utf-8")

        self.assertIn("terrainMaterial = if terrainResult then terrainResult.Material.Name else nil", world_probe_source)
        self.assertIn("materialKindCount", terrain_probe_source)
        self.assertIn("dominantMaterial", terrain_probe_source)
        self.assertIn("dominantMaterialSampleCount", terrain_probe_source)
        self.assertIn("nonGrassSampleCount", terrain_probe_source)
        self.assertIn("sampleCount", terrain_probe_source)
        self.assertIn("status = status", terrain_probe_source)
        self.assertIn("missingEdgeSampleCount", terrain_probe_source)
        self.assertIn("edgeTerrainYRangeStuds", terrain_probe_source)
        self.assertIn("centerEdgeMaxDeltaStuds", terrain_probe_source)
        self.assertIn("edgeIndices = LOCAL_TERRAIN_EDGE_INDICES", world_probe_source)
        self.assertIn("LOCAL_TERRAIN_EDGE_INDICES = {", world_probe_source)

    def test_world_probe_refreshes_faster_while_the_avatar_is_moving(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")

        self.assertIn("local IDLE_SAMPLE_INTERVAL = 1.5", world_probe_source)
        self.assertIn("local MOVING_SAMPLE_INTERVAL = 0.5", world_probe_source)
        self.assertIn("local MOVING_RESAMPLE_DISTANCE = 8", world_probe_source)
        self.assertIn("local MOVING_SPEED_THRESHOLD = 4", world_probe_source)
        self.assertIn("local function resolveMovementAwareSampleCadence(rootPart)", world_probe_source)
        self.assertIn("local sampleInterval, resampleDistance, isMoving =", world_probe_source)
        self.assertIn("if isMoving and not lastSampleWasMoving then", world_probe_source)
        self.assertIn("lastSampleWasMoving = isMoving", world_probe_source)
        self.assertIn("rootPart.AssemblyLinearVelocity.Magnitude", world_probe_source)

    def test_streaming_service_uses_live_avatar_motion_before_the_first_focal_delta(self) -> None:
        streaming_source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("local LIVE_PLAYER_ROOT_MOTION_THRESHOLD = 4", streaming_source)
        self.assertIn("resolveLivePlayerRootMotion = function()", streaming_source)
        self.assertIn("rootPart.AssemblyLinearVelocity", streaming_source)
        self.assertIn("movementDeltaStuds < 1", streaming_source)
        self.assertIn("movementLookaheadStuds = math.min(maxLookaheadStuds, liveMotionSpeed * lookaheadSeconds)", streaming_source)
        self.assertIn("predictedFocalPoint = playerPos + liveMotionForward.Unit * movementLookaheadStuds", streaming_source)

    def test_streaming_engine_uses_explicit_ring_budgets_not_only_distance_radii(self) -> None:
        streaming_source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("resolveStreamingRings(config)", streaming_source)
        self.assertIn("ArnisStreamingRingNearResidentEstimatedCost", streaming_source)
        self.assertIn("ArnisStreamingRingMidResidentEstimatedCost", streaming_source)
        self.assertIn("ArnisStreamingRingFarResidentEstimatedCost", streaming_source)
        self.assertIn("ArnisStreamingQueuedEstimatedCost", streaming_source)
        self.assertIn("ArnisStreamingLastPrefetchReason", streaming_source)
        self.assertIn("ArnisStreamingLastEvictionReason", streaming_source)
        self.assertIn("ArnisStreamingPredictedFocalX", streaming_source)
        self.assertIn("ArnisStreamingPredictedFocalZ", streaming_source)
        self.assertIn("ArnisStreamingMovementDeltaStuds", streaming_source)
        self.assertIn("ArnisStreamingMovementLookaheadStuds", streaming_source)
        self.assertIn("movement_lookahead", streaming_source)

    def test_terrain_builder_prefers_satellite_materials_over_slope_classification(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        # getMat must check terrainGrid.materials as the primary source
        self.assertIn("terrainGrid.materials", source)
        self.assertIn("hasExplicitCellMaterial", source)
        self.assertRegex(
            source,
            r"if\s+hasExplicitCellMaterial\s+then\s*\n\s*return\s+baseMat",
            "expected getMat to return satellite-derived material immediately when present, skipping slope logic",
        )

        # Slope-based fallback must only apply when satellite material is absent
        self.assertRegex(
            source,
            r"if\s+not\s+baseMat\s+then[\s\S]*computeSlope",
            "expected slope-based classification to be a fallback only when satellite material is absent",
        )

        # The expanded satellite material palette must be documented in the resolver
        self.assertIn("SATELLITE_MATERIAL_PALETTE", source)
        for material_name in ("Sand", "Mud", "Pavement", "Limestone", "Sandstone", "Slate", "Asphalt", "Concrete"):
            self.assertIn(
                material_name,
                source,
                f"expected TerrainBuilder to include {material_name} in the satellite material palette",
            )

        # Satellite material resolution must use pcall for safe Enum lookup
        self.assertRegex(
            source,
            r"pcall\(function\(\)\s*\n?\s*return\s+Enum\.Material\[name\]",
            "expected satellite material resolution to use pcall for safe Enum.Material lookup",
        )


if __name__ == "__main__":
    unittest.main()
