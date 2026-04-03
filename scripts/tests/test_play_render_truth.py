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
AUSTIN_PREVIEW_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
IMPORT_SIGNATURES = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ImportSignatures.lua"
STREAMING_SERVICE = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
WORLD_PROBE = ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "WorldProbe.client.lua"
WORLD_PROBE_TERRAIN = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTerrain.lua"
BOOTSTRAP_AUSTIN = ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua"


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
        self.assertIn("surfaceHeight = if maxHeight - minHeight >= TERRAIN_WRITE_RESOLUTION then maxHeight else averageHeight", source)
        self.assertIn("surfaceFillDepth = if heightRange >= TERRAIN_WRITE_RESOLUTION", source)
        self.assertIn("local worldBotY = worldSurfY - columnProfile.surfaceFillDepth", source)
        self.assertIn("local worldSurfY = origin.y + columnProfile.surfaceHeight", source)

    def test_terrain_builder_supports_neighbor_aware_chunk_edge_sampling(self) -> None:
        terrain_source = TERRAIN_BUILDER.read_text(encoding="utf-8")
        import_service_source = IMPORT_SERVICE.read_text(encoding="utf-8")
        streaming_source = STREAMING_SERVICE.read_text(encoding="utf-8")

        self.assertIn("local function resolveNeighborHeightSample(plan, cellX, cellZ)", terrain_source)
        self.assertIn("local function buildTerrainNeighborContextByChunkId(chunks)", import_service_source)
        self.assertIn("terrainNeighborContext = terrainNeighborContextByChunkId[chunk.id]", import_service_source)
        self.assertIn("perChunkOptions.terrainNeighbors = terrainNeighborContext.neighbors", import_service_source)
        self.assertIn("perChunkOptions.terrainNeighborSignature = terrainNeighborContext.signature", import_service_source)
        self.assertIn("local function buildStreamingTerrainNeighborContext(chunkRef)", streaming_source)
        self.assertIn("importOptions.terrainNeighbors = terrainNeighborContext.neighbors", streaming_source)
        self.assertIn("importOptions.terrainNeighborSignature = terrainNeighborContext.signature", streaming_source)

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

    def test_shell_mesh_simple_low_rise_buildings_keep_explicit_shell_walls_for_play_visibility(self) -> None:
        source = BUILDING_BUILDER.read_text(encoding="utf-8")

        self.assertRegex(
            source,
            r"function\s+BuildingBuilder\.MeshBuildAll[\s\S]*if\s+preferSimpleShellDetail\s+then[\s\S]*buildWallLoopParts\(shellFolder,\s*bldgName,\s*worldPts,\s*baseY,\s*height,\s*mat,\s*color,\s*\"outer\"",
            "expected simple low-rise shellMesh buildings to keep explicit shell wall parts instead of only merged wall meshes",
        )

    def test_scene_audit_surfaces_closure_only_roof_gaps_separately_from_generic_roofless_buildings(self) -> None:
        source = SCENE_AUDIT.read_text(encoding="utf-8")

        self.assertIn("buildingModelsWithClosureOnlyRoofGap", source)
        self.assertIn("buildingClosureOnlyRoofGapDetails", source)
        self.assertRegex(
            source,
            r"roofClosureParts\s*>\s*0[\s\S]*buildingModelsWithClosureOnlyRoofGap",
            "expected SceneAudit to classify closure-only shaped roofs as a dedicated roof gap instead of burying them in generic no-roof counts",
        )

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

    def test_player_local_terrain_telemetry_carries_material_richness(self) -> None:
        world_probe_source = WORLD_PROBE.read_text(encoding="utf-8")
        terrain_probe_source = WORLD_PROBE_TERRAIN.read_text(encoding="utf-8")

        self.assertIn("terrainMaterial = if terrainResult then terrainResult.Material.Name else nil", world_probe_source)
        self.assertIn("materialKindCount", terrain_probe_source)
        self.assertIn("dominantMaterial", terrain_probe_source)
        self.assertIn("dominantMaterialSampleCount", terrain_probe_source)
        self.assertIn("nonGrassSampleCount", terrain_probe_source)
        self.assertIn("missingEdgeSampleCount", terrain_probe_source)
        self.assertIn("edgeTerrainYRangeStuds", terrain_probe_source)
        self.assertIn("centerEdgeMaxDeltaStuds", terrain_probe_source)
        self.assertIn("edgeIndices = LOCAL_TERRAIN_EDGE_INDICES", world_probe_source)
        self.assertIn("LOCAL_TERRAIN_EDGE_INDICES = {", world_probe_source)

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


if __name__ == "__main__":
    unittest.main()
