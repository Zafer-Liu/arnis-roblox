#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua"
RUN_AUSTIN_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "RunAustin.lua"
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
CHUNK_PRIORITY_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ChunkPriority.lua"
IMPORT_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "init.lua"
SIGNATURES_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ImportSignatures.lua"
WORLD_PROBE_PATH = ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "WorldProbe.client.lua"
WORLD_PROBE_SUPPORT_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeSupport.lua"
WORLD_PROBE_FLAGS_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTelemetryFlags.lua"
WORLD_PROBE_TERRAIN_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTerrain.lua"
WORLD_PROBE_TERRAIN_SPEC_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "WorldProbeTerrain.spec.lua"
WORLD_PROBE_FLAGS_SPEC_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "WorldProbeTelemetryFlags.spec.lua"
WORLD_STATE_APPLIER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "WorldStateApplier.lua"
MINIMAP_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "MinimapService.lua"
PREVIEW_BUILDER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
BUILDING_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "BuildingBuilder.lua"
)
SCENE_AUDIT_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "SceneAudit.lua"
WORLD_CONFIG_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldConfig.lua"


class AustinRuntimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bootstrap_text = BOOTSTRAP_PATH.read_text(encoding="utf-8")
        cls.run_austin_text = RUN_AUSTIN_PATH.read_text(encoding="utf-8")
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.chunk_priority_text = CHUNK_PRIORITY_PATH.read_text(encoding="utf-8")
        cls.import_service_text = IMPORT_SERVICE_PATH.read_text(encoding="utf-8")
        cls.signatures_text = SIGNATURES_PATH.read_text(encoding="utf-8") if SIGNATURES_PATH.exists() else ""
        cls.world_probe_text = WORLD_PROBE_PATH.read_text(encoding="utf-8") if WORLD_PROBE_PATH.exists() else ""
        cls.world_probe_support_text = (
            WORLD_PROBE_SUPPORT_PATH.read_text(encoding="utf-8") if WORLD_PROBE_SUPPORT_PATH.exists() else ""
        )
        cls.world_probe_flags_text = WORLD_PROBE_FLAGS_PATH.read_text(encoding="utf-8") if WORLD_PROBE_FLAGS_PATH.exists() else ""
        cls.world_probe_terrain_spec_text = (
            WORLD_PROBE_TERRAIN_SPEC_PATH.read_text(encoding="utf-8")
            if WORLD_PROBE_TERRAIN_SPEC_PATH.exists()
            else ""
        )
        cls.world_probe_flags_spec_text = (
            WORLD_PROBE_FLAGS_SPEC_PATH.read_text(encoding="utf-8") if WORLD_PROBE_FLAGS_SPEC_PATH.exists() else ""
        )
        cls.world_probe_terrain_text = (
            WORLD_PROBE_TERRAIN_PATH.read_text(encoding="utf-8") if WORLD_PROBE_TERRAIN_PATH.exists() else ""
        )
        cls.world_state_applier_text = WORLD_STATE_APPLIER_PATH.read_text(encoding="utf-8")
        cls.minimap_service_text = MINIMAP_SERVICE_PATH.read_text(encoding="utf-8")
        cls.preview_builder_text = PREVIEW_BUILDER_PATH.read_text(encoding="utf-8")
        cls.building_builder_text = BUILDING_BUILDER_PATH.read_text(encoding="utf-8")
        cls.scene_audit_text = SCENE_AUDIT_PATH.read_text(encoding="utf-8")
        cls.world_config_text = WORLD_CONFIG_PATH.read_text(encoding="utf-8")

    def test_bootstrap_guards_against_duplicate_runtime_execution(self) -> None:
        self.assertIn(
            "local BootstrapStateMachine = require(script.Parent.ImportService.BootstrapStateMachine)",
            self.bootstrap_text,
        )
        self.assertIn("local BOOTSTRAP_STATE_ATTR = BootstrapStateMachine.STATE_ATTR", self.bootstrap_text)
        self.assertIn("local BOOTSTRAP_STATE_TRACE_ATTR = BootstrapStateMachine.STATE_TRACE_ATTR", self.bootstrap_text)
        self.assertIn(
            "local BOOTSTRAP_DUPLICATE_COUNT_ATTR = BootstrapStateMachine.DUPLICATE_COUNT_ATTR",
            self.bootstrap_text,
        )
        self.assertIn("local BOOTSTRAP_ENTRY_COUNT_ATTR = BootstrapStateMachine.ENTRY_COUNT_ATTR", self.bootstrap_text)
        self.assertIn(
            "local BOOTSTRAP_LAST_SCRIPT_PATH_ATTR = BootstrapStateMachine.LAST_SCRIPT_PATH_ATTR",
            self.bootstrap_text,
        )
        self.assertIn(
            'local BOOTSTRAP_ATTEMPT_ID_ATTR = "ArnisAustinBootstrapAttemptId"',
            self.bootstrap_text,
        )
        self.assertIn("local bootstrapMachine, duplicateAttempt = BootstrapStateMachine.begin(", self.bootstrap_text)
        self.assertIn("BootstrapStateMachine.transition(bootstrapMachine, state)", self.bootstrap_text)
        self.assertIn("BootstrapStateMachine.fail(bootstrapMachine)", self.bootstrap_text)
        self.assertIn('reportPhase(options, "loading_manifest")', self.run_austin_text)
        self.assertIn('reportPhase(options, "importing_startup")', self.run_austin_text)
        self.assertIn('setBootstrapState("world_ready")', self.bootstrap_text)
        self.assertIn('setBootstrapState("streaming_ready")', self.bootstrap_text)
        self.assertIn('setBootstrapState("minimap_ready")', self.bootstrap_text)
        self.assertIn('setBootstrapState("gameplay_ready")', self.bootstrap_text)
        self.assertIn("BootstrapStateMachine.fail(bootstrapMachine)", self.bootstrap_text)
        self.assertIn(
            'Workspace:GetAttribute(BOOTSTRAP_ATTEMPT_ID_ATTR)',
            self.bootstrap_text,
        )
        self.assertIn('"[BootstrapAustin] Duplicate bootstrap attempt ignored. state="', self.bootstrap_text)

    def test_bootstrap_lifts_characters_above_spawn_surface_before_pivoting(self) -> None:
        self.assertIn("local function getCharacterSpawnCFrame(character)", self.bootstrap_text)
        self.assertIn("local extents = character:GetExtentsSize()", self.bootstrap_text)
        self.assertIn("local spawnLift = math.max(6, extents.Y * 0.5 + 0.5)", self.bootstrap_text)
        self.assertIn("local elevatedPosition = basePosition + Vector3.new(0, spawnLift, 0)", self.bootstrap_text)
        self.assertIn("local characterSpawnCFrame = getCharacterSpawnCFrame(character)", self.bootstrap_text)
        self.assertIn("character:PivotTo(characterSpawnCFrame)", self.bootstrap_text)

    def test_bootstrap_hides_respawn_pad_and_uses_ground_surface_not_double_lift(self) -> None:
        self.assertIn("local function isDecorativeRoadDetailDescendant(hitInstance, worldRoot)", self.bootstrap_text)
        self.assertIn('if isDecorativeRoadDetailDescendant(hitInstance, worldRoot) then', self.bootstrap_text)
        self.assertIn("return hit.Position.Y", self.bootstrap_text)
        self.assertIn("spawn.Transparency = 1", self.bootstrap_text)
        self.assertIn("spawn.CanCollide = false", self.bootstrap_text)
        self.assertIn("local spawnSurfaceY = findGroundYNear(worldRoot, spawnPoint, holdingPad, spawn)", self.bootstrap_text)
        self.assertIn("local spawnCenterY = spawnSurfaceY + spawn.Size.Y * 0.5", self.bootstrap_text)
        self.assertIn("local lookTarget = Vector3.new(preferredLookTarget.X, spawnSurfaceY, preferredLookTarget.Z)", self.bootstrap_text)
        self.assertIn("spawn.CFrame = CFrame.new(spawnPoint.X, spawnCenterY, spawnPoint.Z)", self.bootstrap_text)
        self.assertIn("spawnCFrame = CFrame.lookAt(Vector3.new(spawnPoint.X, spawnSurfaceY, spawnPoint.Z), lookTarget)", self.bootstrap_text)

    def test_bootstrap_waits_for_near_ring_streaming_settlement_before_gameplay_ready(self) -> None:
        self.assertIn('local STARTUP_STREAMING_TIMEOUT_SECONDS = 10', self.bootstrap_text)
        self.assertIn("local function waitForStartupStreamingReady(spawnPoint)", self.bootstrap_text)
        self.assertIn(
            'local startupResidency = StreamingService.GetStartupResidencySnapshot(spawnPoint, "GeneratedWorld_Austin")',
            self.bootstrap_text,
        )
        self.assertIn("startupResidency.ready", self.bootstrap_text)
        self.assertIn("StreamingService.Update(spawnPoint)", self.bootstrap_text)
        self.assertIn("local streamingStartupReady = waitForStartupStreamingReady(spawnPoint)", self.bootstrap_text)
        self.assertIn("if not streamingStartupReady then", self.bootstrap_text)
        self.assertIn('warn("[BootstrapAustin] Startup streaming did not settle the near ring before gameplay readiness.")', self.bootstrap_text)

    def test_run_austin_publishes_runtime_world_root_telemetry(self) -> None:
        self.assertIn('setPerfAttribute("WorldRootName", "GeneratedWorld_Austin")', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootChildCount", #worldRoot:GetChildren())', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootDescendantCount", #worldRoot:GetDescendants())', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootExists", 1)', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootExists", 0)', self.run_austin_text)

    def test_shell_mesh_bounded_wall_fallback_stays_shape_limited(self) -> None:
        self.assertIn("local function shouldPreferPlayVisibleShellWalls", self.building_builder_text)
        self.assertIn("PLAY_VISIBLE_SHELL_ROOF_SHAPES", self.building_builder_text)
        self.assertIn("levels > 6 or height > 34", self.building_builder_text)
        self.assertIn("return footprintPointCount <= 10", self.building_builder_text)
        self.assertIn("if boundedHoleLoopCount > 1 then", self.building_builder_text)
        self.assertIn("if boundedHoleLoopCount == 1 then", self.building_builder_text)
        self.assertIn("return levels <= 5 and height <= 28 and footprintPointCount <= 12", self.building_builder_text)
        self.assertIn("preferPlayVisibleShellWalls", self.building_builder_text)
        self.assertIn("if preferPlayVisibleShellWalls then", self.building_builder_text)

    def test_streaming_service_publishes_startup_residency_telemetry(self) -> None:
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLoadedChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingDesiredChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingCandidateChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingProcessedWorkItems", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastFocalX", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastFocalZ", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingPredictedFocalX", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingPredictedFocalZ", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingMovementDeltaStuds", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingMovementLookaheadStuds", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLoadedChunkCount"', self.streaming_text)
        self.assertIn('#ChunkLoader.ListLoadedChunks(worldRootName)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingCandidateChunkCount", #candidateChunkEntries)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingDesiredChunkCount", desiredChunkCount)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingProcessedWorkItems", processedWorkItems)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearResidentChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidResidentChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarResidentChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearDesiredChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidDesiredChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarDesiredChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearResidentEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidResidentEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarResidentEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearDesiredEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidDesiredEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarDesiredEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearBudgetBytes", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidBudgetBytes", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarBudgetBytes", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingNearMaxChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingMidMaxChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingRingFarMaxChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingQueuedEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingQueuedWorkItemCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingEvictedEstimatedCost", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingEvictedChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingSchedulerState", "idle")', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastPrefetchReason", "")', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastEvictionReason", "")', self.streaming_text)
        self.assertIn("function StreamingService.GetStartupResidencySnapshot", self.streaming_text)
        self.assertIn('model:GetAttribute("ArnisChunkId")', self.streaming_text)
        self.assertIn('chunkEntryHasStartupOwnership(chunkEntry, chunkId)', self.streaming_text)
        self.assertIn("nearbyBuildingModels", self.streaming_text)
        self.assertIn("nearbyWallParts", self.streaming_text)
        self.assertIn("nearbyRoofParts", self.streaming_text)
        self.assertIn("collidableWallPartsNearby", self.streaming_text)
        self.assertIn("overheadRoofParts", self.streaming_text)
        self.assertIn("coherentEnvelopeNearbyBuildingModels", self.streaming_text)
        self.assertIn("coherentEnvelopeNearbyWallParts", self.streaming_text)
        self.assertIn("coherentEnvelopeNearbyRoofParts", self.streaming_text)
        self.assertIn("coherentEnvelopeOverheadRoofParts", self.streaming_text)
        self.assertIn("coherentEnvelopeCollidableWallPartsNearby", self.streaming_text)
        self.assertIn("coherentEnvelopeSourceId", self.streaming_text)
        self.assertIn("coherentEnvelopeCandidateCount", self.streaming_text)
        self.assertIn("coherentEnvelopeReady", self.streaming_text)
        self.assertIn("local function isStartupEnvelopeReady(envelopeTelemetry)", self.streaming_text)
        self.assertIn('or name == "MergedShellWindowPaneCue"', self.streaming_text)
        self.assertIn('name == "MergedShellRooflineCue"', self.streaming_text)
        self.assertIn('or name == "MergedShellPerimeterCue"', self.streaming_text)
        self.assertIn("local function selectStartupEnvelopeTelemetry(candidateTelemetryBySourceId)", self.streaming_text)
        self.assertIn("local function countStartupEnvelopeCandidates(candidateTelemetryBySourceId)", self.streaming_text)
        self.assertIn("local coherentEnvelopeReady = isStartupEnvelopeReady({", self.streaming_text)
        self.assertIn("local hasRoofEnvelope = envelopeTelemetry.nearbyRoofParts > 0", self.streaming_text)
        self.assertIn("and coherentEnvelopeReady", self.streaming_text)

    def test_streaming_lod_visibility_uses_group_footprint_not_only_chunk_center(self) -> None:
        self.assertIn("local lodGroupFootprintBoundsCache = setmetatable({}, { __mode = \"k\" })", self.streaming_text)
        self.assertIn("local function getLodGroupFootprintBounds(group, fallbackPosition)", self.streaming_text)
        self.assertIn("local function getLodGroupFootprintDistanceSq(group, fallbackPosition, point)", self.streaming_text)
        self.assertIn("local highDetailRadiusSq = highDetailRadius * highDetailRadius", self.streaming_text)
        self.assertIn("local interiorRadiusSq = interiorRadius * interiorRadius", self.streaming_text)
        self.assertIn("local cameraFocusPos = primaryFocusPos", self.streaming_text)
        self.assertIn("local avatarFocusPos = secondaryFocusPos", self.streaming_text)
        self.assertIn("local detailVisible = isLodGroupVisibleForFocus(group, chunkCenter, cameraFocusPos, highDetailRadiusSq)", self.streaming_text)
        self.assertIn(
            "detailVisible = isLodGroupVisibleForFocus(group, chunkCenter, avatarFocusPos, highDetailRadiusSq)",
            self.streaming_text,
        )
        self.assertIn(
            "local interiorVisible = isLodGroupVisibleForFocus(group, chunkCenter, avatarFocusPos, interiorRadiusSq)",
            self.streaming_text,
        )

    def test_streaming_start_applies_immediate_lod_sync_for_seeded_chunks(self) -> None:
        self.assertIn("seedLoadedChunkLods(streamingChunkOptionsByLod, streamingOptions.worldRootName)", self.streaming_text)
        self.assertIn("updateLOD()", self.streaming_text)
        self.assertIn("local function resolveCurrentCameraFocusPosition()", self.streaming_text)
        self.assertIn("local immediateCameraFocusPos = resolveCurrentCameraFocusPosition()", self.streaming_text)
        self.assertIn("local function shouldForceMovementLodRefresh()", self.streaming_text)
        self.assertIn("lastLODUpdate = LOD_UPDATE_INTERVAL", self.streaming_text)
        self.assertIn("local importedChunkEntry = ChunkLoader.GetChunkEntry(chunkRef.id, streamingOptions.worldRootName)", self.streaming_text)

    def test_streaming_residency_uses_chunk_footprint_distance_not_only_chunk_center(self) -> None:
        self.assertIn("function ChunkPriority.GetChunkFootprintBounds", self.chunk_priority_text)
        self.assertIn("function ChunkPriority.GetChunkFootprintDistanceSq", self.chunk_priority_text)
        self.assertIn("local chunkFootprintBounds = ChunkPriority.GetChunkFootprintBounds(chunkRef)", self.streaming_text)
        self.assertIn("chunkRef.streamingFootprintBounds = chunkFootprintBounds", self.streaming_text)
        self.assertIn(
            "ChunkPriority.GetChunkFootprintDistanceSq(chunkRef, playerPos, streamingOptions.config.ChunkSizeStuds)",
            self.streaming_text,
        )
        self.assertIn("local actualDistSq = getChunkDistanceSqToPoint(chunkEntry, playerPos)", self.streaming_text)
        self.assertIn("local ringName = getChunkRingName(chunkFootprintDistanceSq, resolvedRings)", self.streaming_text)
        self.assertIn("if chunkFootprintDistanceSq > targetExitRadiusSq then", self.streaming_text)

    def test_streaming_service_requires_registered_chunks_for_startup_structure_telemetry(self) -> None:
        self.assertIn("for _, chunkId in ipairs(ChunkLoader.ListLoadedChunks(resolvedWorldRootName)) do", self.streaming_text)
        self.assertIn("local chunkEntry = ChunkLoader.GetChunkEntry(chunkId, resolvedWorldRootName)", self.streaming_text)
        self.assertIn("if not chunkEntryBelongsToWorldRoot(chunkEntry, resolvedWorldRootName) then", self.streaming_text)
        self.assertIn("local chunkFolder = chunkEntry.folder", self.streaming_text)
        self.assertIn('if not chunkFolder or chunkFolder:GetAttribute("ArnisChunkId") ~= chunkId then', self.streaming_text)
        self.assertIn("local buildingsFolder = chunkFolder:FindFirstChild(\"Buildings\")", self.streaming_text)

    def test_streaming_service_bypasses_subplan_rollout_for_high_detail_building_chunks(self) -> None:
        self.assertIn("local hasPendingBuildingSubplans", self.streaming_text)
        self.assertIn("local shouldBypassHighDetailSubplanRollout", self.streaming_text)
        self.assertIn("hasPendingBuildingSubplans = function(chunkRef, config)", self.streaming_text)
        self.assertIn(
            "shouldBypassHighDetailSubplanRollout = function(chunkEntry, chunkOptions, targetLod)",
            self.streaming_text,
        )
        self.assertIn("if shouldBypassHighDetailSubplanRollout(chunkEntry, chunkOptions, targetLod) then", self.streaming_text)
        self.assertIn("workItems[#workItems + 1] = {", self.streaming_text)
        self.assertIn("targetLod == LOD_HIGH", self.streaming_text)
        self.assertIn('subplan.layer == "buildings"', self.streaming_text)
        self.assertIn("highDetailWholeChunkPriority = true", self.streaming_text)

    def test_world_config_declares_explicit_runtime_streaming_rings(self) -> None:
        self.assertIn("StreamingRings = {", self.world_config_text)
        self.assertIn("near = {", self.world_config_text)
        self.assertIn("mid = {", self.world_config_text)
        self.assertIn("far = {", self.world_config_text)
        self.assertIn("StreamingLookaheadSeconds", self.world_config_text)
        self.assertIn("StreamingMaxLookaheadStuds", self.world_config_text)
        self.assertIn("EstimatedBudgetBytes", self.world_config_text)
        self.assertIn("MaxChunkCount", self.world_config_text)

    def test_streaming_service_treats_estimated_memory_cost_as_authoritative_ring_budget(self) -> None:
        self.assertIn("local function resolveStreamingRings(config)", self.streaming_text)
        self.assertIn("EstimatedBudgetBytes", self.streaming_text)
        self.assertIn("MaxChunkCount", self.streaming_text)
        self.assertIn("ArnisStreamingQueuedEstimatedCost", self.streaming_text)
        self.assertIn("ArnisStreamingEvictedEstimatedCost", self.streaming_text)
        self.assertIn("ArnisStreamingEvictedChunkCount", self.streaming_text)
        self.assertIn("ArnisStreamingRingNearBudgetBytes", self.streaming_text)
        self.assertIn("ArnisStreamingRingNearMaxChunkCount", self.streaming_text)
        self.assertIn("ArnisStreamingRingNearDesiredChunkCount", self.streaming_text)
        self.assertIn("ArnisStreamingRingNearDesiredEstimatedCost", self.streaming_text)
        self.assertIn("ArnisStreamingSchedulerState", self.streaming_text)
        self.assertIn("ArnisStreamingLastPrefetchReason", self.streaming_text)
        self.assertIn("ArnisStreamingLastEvictionReason", self.streaming_text)
        self.assertIn("ArnisStreamingPredictedFocalX", self.streaming_text)
        self.assertIn("ArnisStreamingMovementLookaheadStuds", self.streaming_text)
        self.assertIn('lastPrefetchReason = "movement_lookahead"', self.streaming_text)

    def test_startup_import_and_streaming_share_chunk_signature_contract(self) -> None:
        self.assertIn("local ImportSignatures = require(script.Parent.ImportSignatures)", self.streaming_text)
        self.assertIn("local ImportSignatures = require(script.ImportSignatures)", self.import_service_text)
        self.assertIn("ImportSignatures.GetChunkSignature(chunkRef)", self.streaming_text)
        self.assertIn("perChunkOptions.chunkSignature = ImportSignatures.GetChunkSignature(", self.import_service_text)
        self.assertIn("configSignature = ImportSignatures.GetConfigSignature(config)", self.import_service_text)
        self.assertIn("layerSignatures = ImportSignatures.GetLayerSignatures(config)", self.import_service_text)
        self.assertIn("function ImportSignatures.GetChunkSignature(chunkRef)", self.signatures_text)
        self.assertIn("function ImportSignatures.GetConfigSignature(config)", self.signatures_text)
        self.assertIn("function ImportSignatures.GetLayerSignatures(config)", self.signatures_text)

    def test_runtime_startup_import_registers_canonical_chunk_refs(self) -> None:
        self.assertIn("local startupChunkRefsById = {}", self.run_austin_text)
        self.assertIn("startupChunkRefsById[chunkId] = manifestSource:ResolveChunkRef(chunkId)", self.run_austin_text)
        self.assertIn("registrationChunksById = startupChunkRefsById", self.run_austin_text)
        self.assertIn("local registrationChunksById = options.registrationChunksById", self.import_service_text)
        self.assertIn("local registrationChunk = registrationChunksById and registrationChunksById[chunk.id] or nil", self.import_service_text)
        self.assertIn("perChunkOptions.registrationChunk = registrationChunk", self.import_service_text)
        self.assertIn("perChunkOptions.chunkSignature = ImportSignatures.GetChunkSignature(registrationChunk or chunk)", self.import_service_text)
        self.assertIn("setImportAuditAttributes(layerFolder, chunk.id, options.importRunId)", self.import_service_text)
        self.assertIn("setImportAuditAttributes(subplanFolder, chunk.id, options.importRunId)", self.import_service_text)
        self.assertIn("setImportAuditAttributes(folder, chunk.id, options.importRunId)", self.import_service_text)

    def test_client_world_probe_publishes_nearby_building_and_overhead_roof_telemetry(self) -> None:
        self.assertIn('local payloadJson = HttpService:JSONEncode(payload)', self.world_probe_text)
        self.assertIn('print("ARNIS_CLIENT_WORLD " .. payloadJson)', self.world_probe_text)
        self.assertIn('print("ARNIS_CLIENT_WORLD_COMPACT " .. compactPayloadJson)', self.world_probe_text)
        self.assertIn('print("ARNIS_CLIENT_BOOTSTRAP " .. bootstrapPayloadJson)', self.world_probe_text)
        self.assertIn('local WORLD_ROOT_ATTR = "ArnisWorldRootName"', self.world_probe_text)
        self.assertIn('local worldRootName = Workspace:GetAttribute(WORLD_ROOT_ATTR)', self.world_probe_text)
        self.assertIn('local worldRoot = Workspace:FindFirstChild(worldRootName)', self.world_probe_text)
        self.assertIn('model:GetAttribute("ArnisImportSourceId")', self.world_probe_text)
        self.assertIn('model:GetAttribute("ArnisImportRoofShape")', self.world_probe_text)
        self.assertIn('model:GetAttribute("ArnisImportBuildingTopY")', self.world_probe_text)
        self.assertIn('local mergedMeshes = buildingsFolder:FindFirstChild("MergedMeshes")', self.world_probe_text)
        self.assertIn("nearbyMergedBuildingMeshParts", self.world_probe_text)
        self.assertIn("local rayResult = Workspace:Raycast(", self.world_probe_text)
        self.assertIn("groundMaterial =", self.world_probe_text)
        self.assertIn('overheadRoofParts', self.world_probe_text)
        self.assertIn('nearbyBuildingModels', self.world_probe_text)
        self.assertIn('bootstrapAttemptId = Workspace:GetAttribute("ArnisAustinBootstrapAttemptId")', self.world_probe_text)
        self.assertIn('bootstrapState = Workspace:GetAttribute("ArnisAustinBootstrapState")', self.world_probe_text)
        self.assertIn('bootstrapStateTrace = Workspace:GetAttribute("ArnisAustinBootstrapStateTrace")', self.world_probe_text)
        self.assertIn('bootstrapDuplicateCount = Workspace:GetAttribute("ArnisAustinBootstrapDuplicateCount")', self.world_probe_text)
        self.assertIn('bootstrapLastScriptPath = Workspace:GetAttribute("ArnisAustinBootstrapLastScriptPath")', self.world_probe_text)
        self.assertIn("local WorldProbeSupport = require(ReplicatedStorage.Shared.WorldProbeSupport)", self.world_probe_text)
        self.assertIn("WorldProbeSupport.shouldIgnoreGroundHit(rayResult.Instance, worldRoot, ignore)", self.world_probe_text)
        self.assertIn("local function isDecorativeRoadDetailDescendant(hitInstance, worldRoot)", self.bootstrap_text)

    def test_world_root_publication_is_owned_outside_minimap_startup(self) -> None:
        self.assertIn('local WORLD_ROOT_ATTR = "ArnisWorldRootName"', self.world_state_applier_text)
        self.assertIn('Workspace:SetAttribute(WORLD_ROOT_ATTR, resolvedOptions.worldRootName or "GeneratedWorld")', self.world_state_applier_text)
        self.assertIn('local WORLD_ROOT_ATTR = "ArnisWorldRootName"', self.minimap_service_text)
        self.assertIn('Workspace:GetAttribute(WORLD_ROOT_ATTR)', self.minimap_service_text)
        self.assertIn('Workspace:SetAttribute(MINIMAP_WORLD_ROOT_ATTR, Workspace:GetAttribute(WORLD_ROOT_ATTR)', self.minimap_service_text)
        self.assertNotIn('Workspace:SetAttribute(WORLD_ROOT_ATTR, resolvedOptions.worldRootName or Workspace:GetAttribute(WORLD_ROOT_ATTR) or "GeneratedWorld")', self.minimap_service_text)
        self.assertNotIn('Workspace:SetAttribute(WORLD_ROOT_ATTR, "GeneratedWorld_Austin")', self.run_austin_text)

    def test_runtime_contract_exposes_bootstrap_state_trace_for_ordered_readiness_assertions(self) -> None:
        self.assertIn("local BOOTSTRAP_STATE_TRACE_ATTR = BootstrapStateMachine.STATE_TRACE_ATTR", self.bootstrap_text)
        self.assertIn('reportPhase(options, "loading_manifest")', self.run_austin_text)
        self.assertIn('reportPhase(options, "importing_startup")', self.run_austin_text)
        self.assertIn('setBootstrapState("world_ready")', self.bootstrap_text)
        self.assertIn('setBootstrapState("streaming_ready")', self.bootstrap_text)
        self.assertIn('if runtimeWorldConfig.EnableMinimap ~= false and Workspace:GetAttribute("ArnisMinimapEnabled") ~= true then', self.bootstrap_text)
        self.assertIn('setBootstrapState("minimap_ready")', self.bootstrap_text)
        self.assertIn('setBootstrapState("gameplay_ready")', self.bootstrap_text)

    def test_bootstrap_emits_authoritative_play_scene_marker_from_live_runtime(self) -> None:
        self.assertIn("local SceneAudit = require(script.Parent.ImportService.SceneAudit)", self.bootstrap_text)
        self.assertIn("local CanonicalWorldContract = require(script.Parent.ImportService.CanonicalWorldContract)", self.bootstrap_text)
        self.assertIn("local SceneMarkerEmitter = require(script.Parent.ImportService.SceneMarkerEmitter)", self.bootstrap_text)
        self.assertIn("task.defer(function()", self.bootstrap_text)
        self.assertRegex(
            self.bootstrap_text,
            r'SceneMarkerEmitter\.emitSceneMarkers\(\s*"ARNIS_SCENE_PLAY",\s*"play",',
        )
        self.assertIn('worldIdentity = CanonicalWorldContract.resolveCanonicalManifestFamily("play")', self.bootstrap_text)
        self.assertIn('chunkEnvelopeKind = "runtime_resident"', self.bootstrap_text)
        self.assertIn('manifestSourceKind = result.manifestSourceKind or "canonical_manifest"', self.bootstrap_text)
        self.assertIn('manifestSourceName = result.resolvedManifestName or RunAustin.getManifestName()', self.bootstrap_text)
        self.assertIn('local sceneSummary = SceneAudit.summarizeWorld(worldRoot)', self.bootstrap_text)

    def test_preview_and_play_share_one_resolved_world_config_contract(self) -> None:
        self.assertIn(
            "local StreamingRuntimeConfig = require(ReplicatedStorage.Shared.StreamingRuntimeConfig)",
            self.preview_builder_text,
        )
        self.assertIn(
            "local previewWorldConfig = StreamingRuntimeConfig.Resolve(DefaultWorldConfig)",
            self.preview_builder_text,
        )
        self.assertIn(
            "worldStateApplier.Apply(resolvedManifestSource, previewWorldConfig, {",
            self.preview_builder_text,
        )
        self.assertIn(
            "local runtimeWorldConfig = options.config",
            self.run_austin_text,
        )
        self.assertIn(
            "config = runtimeWorldConfig,",
            self.run_austin_text,
        )
        self.assertIn(
            "local result = RunAustin.run({",
            self.bootstrap_text,
        )
        self.assertIn(
            "config = runtimeWorldConfig,",
            self.bootstrap_text,
        )
        self.assertIn('RunAustin.ROUTE_CATALOG_ATTR = "VertigoRouteCatalogName"', self.run_austin_text)
        self.assertIn('RunAustin.ROUTE_LANE_ATTR = "VertigoRouteLane"', self.run_austin_text)
        self.assertIn('RunAustin.ROUTE_STEP_INDEX_ATTR = "VertigoRouteStepIndex"', self.run_austin_text)
        self.assertIn("local function resolveRouteSelectionOptions(options)", self.run_austin_text)
        self.assertIn("Workspace:GetAttribute(RunAustin.ROUTE_CATALOG_ATTR)", self.run_austin_text)
        self.assertIn("Workspace:GetAttribute(RunAustin.ROUTE_LANE_ATTR)", self.run_austin_text)
        self.assertIn("Workspace:GetAttribute(RunAustin.ROUTE_STEP_INDEX_ATTR)", self.run_austin_text)
        self.assertIn("local routeSelectionOptions = resolveRouteSelectionOptions(options)", self.run_austin_text)
        self.assertIn("routeCatalogName = routeSelectionOptions.routeCatalogName", self.run_austin_text)
        self.assertIn("routeLane = routeSelectionOptions.routeLane", self.run_austin_text)
        self.assertIn("routeStepIndex = routeSelectionOptions.routeStepIndex", self.run_austin_text)
        self.assertIn('setPerfAttribute("RouteCatalogName", routeSelectionOptions.routeCatalogName or "")', self.run_austin_text)
        self.assertIn('setPerfAttribute("RouteLane", routeSelectionOptions.routeLane or "")', self.run_austin_text)
        self.assertIn('setPerfAttribute("RouteStepIndex", routeSelectionOptions.routeStepIndex or -1)', self.run_austin_text)
        self.assertIn('"ManifestSourceKind"', self.run_austin_text)
        self.assertIn('routeSelectionOptions.routeCatalogName and "route_catalog" or "canonical_manifest"', self.run_austin_text)
        self.assertIn('setPerfAttribute("ManifestSourceName", resolvedManifestName or RunAustin.getManifestName())', self.run_austin_text)
        self.assertIn("resolvedManifestName = resolvedManifestName,", self.run_austin_text)
        self.assertIn('manifestSourceKind = Workspace:GetAttribute("VertigoAustinManifestSourceKind"),', self.run_austin_text)
        self.assertIn('RouteCatalogName = normalizedRequest and normalizedRequest.routeCatalogName or ""', self.preview_builder_text)
        self.assertIn('RouteLane = normalizedRequest and normalizedRequest.routeLane or ""', self.preview_builder_text)
        self.assertIn('RouteStepIndex = normalizedRequest and normalizedRequest.routeStepIndex or -1', self.preview_builder_text)
        self.assertIn("ManifestSourceKind = manifestSourceKind", self.preview_builder_text)
        self.assertIn("ManifestSourceName = resolvedManifestName or \"\"", self.preview_builder_text)
        self.assertIn("cachedPreviewManifestName = resolvedManifestName", self.preview_builder_text)
        self.assertIn("cachedFullManifestName = resolvedManifestName", self.preview_builder_text)
        self.assertIn('local manifestSourceKind =', self.preview_builder_text)
        self.assertIn('normalizedRequest and normalizedRequest.routeCatalogName and "route_catalog"', self.preview_builder_text)
        self.assertIn('[BootstrapAustin] Manifest source kind=%s name=%s', self.bootstrap_text)
        self.assertIn("result.manifestSourceKind or \"canonical_manifest\"", self.bootstrap_text)
        self.assertIn("result.resolvedManifestName or RunAustin.getManifestName()", self.bootstrap_text)

    def test_client_world_probe_exposes_support_wall_and_roof_cover_observability(self) -> None:
        self.assertIn("supportSurfaceRole =", self.world_probe_text)
        self.assertIn("supportY =", self.world_probe_text)
        self.assertIn("terrainY =", self.world_probe_text)
        self.assertIn("supportMinusTerrainYStuds =", self.world_probe_text)
        self.assertIn("nearbyWallParts =", self.world_probe_text)
        self.assertIn("nearbyReadableFacadeCueParts =", self.world_probe_text)
        self.assertIn("collidableWallPartsNearby =", self.world_probe_text)
        self.assertIn("nearestWallDistanceStuds =", self.world_probe_text)
        self.assertIn("overheadRoofMinClearanceStuds =", self.world_probe_text)
        self.assertIn("localSupport =", self.world_probe_text)
        self.assertIn("localEnclosure =", self.world_probe_text)
        self.assertIn("localRoofCover =", self.world_probe_text)
        self.assertIn('local shellFolder = model:FindFirstChild("Shell")', self.world_probe_text)
        self.assertIn("local isRoofClosureDeck = descendant:GetAttribute(\"ArnisRoofClosureDeck\") == true", self.world_probe_text)
        self.assertIn('name == "MergedShellRooflineCue"', self.world_probe_text)
        self.assertIn('or name == "MergedShellPerimeterCue"', self.world_probe_text)
        self.assertIn("local isRoofCue = isRoofCuePart(descendant)", self.world_probe_text)
        self.assertIn("local function isReadableFacadeCuePart(part)", self.world_probe_text)
        self.assertIn("if descendant:IsA(\"MeshPart\") and shellFolder and descendant:IsDescendantOf(shellFolder)", self.world_probe_text)
        self.assertRegex(
            self.world_probe_text,
            r"if\s+shellFolder\s+and\s+descendant:IsDescendantOf\(shellFolder\)\s+and\s+not\s+isRoofPart\s+and\s+not\s+isRoofCue\s+and\s+not\s+isRoofClosureDeck\s+then",
        )
        self.assertIn("WorldProbeSupport.shouldIgnoreGroundHit", self.world_probe_text)
        self.assertIn("WorldProbeSupport.classifySupportSurfaceRole", self.world_probe_text)
        self.assertIn("function WorldProbeSupport.classifySupportSurfaceRole(hitInstance)", self.world_probe_support_text)
        self.assertIn("function WorldProbeSupport.shouldIgnoreGroundHit(hitInstance, worldRoot, ignoredRoots)", self.world_probe_support_text)
        self.assertIn('if hitInstance:IsA("SpawnLocation") then', self.world_probe_support_text)
        self.assertIn('local surfaceRole = node:GetAttribute("ArnisRoadSurfaceRole")', self.world_probe_support_text)

    def test_client_world_probe_exposes_local_terrain_roughness_metrics(self) -> None:
        self.assertIn("local WorldProbeTelemetryFlags = require(ReplicatedStorage.Shared.WorldProbeTelemetryFlags)", self.world_probe_text)
        self.assertIn("local WorldProbeTerrain = require(ReplicatedStorage.Shared.WorldProbeTerrain)", self.world_probe_text)
        self.assertIn('local telemetryFamilies = Workspace:GetAttribute(WorldProbeTelemetryFlags.WORKSPACE_ATTR)', self.world_probe_text)
        self.assertIn("local telemetryFlags = WorldProbeTelemetryFlags.parseTelemetryFamilies(telemetryFamilies)", self.world_probe_text)
        self.assertIn("WorldProbeTelemetryFlags.annotateMarkerPayload(bootstrapPayload, telemetryFlags)", self.world_probe_text)
        self.assertIn("sampleCount", self.world_probe_terrain_text)
        self.assertIn("missingEdgeSampleCount", self.world_probe_terrain_text)
        self.assertIn("edgeTerrainYRangeStuds", self.world_probe_terrain_text)
        self.assertIn("centerEdgeMaxDeltaStuds", self.world_probe_terrain_text)
        self.assertIn("local function countSampleSlots(samples)", self.world_probe_terrain_text)
        self.assertIn("sampleCount", self.world_probe_terrain_spec_text)
        self.assertIn("missingEdgeSampleCount", self.world_probe_terrain_spec_text)
        self.assertIn("edgeTerrainYRangeStuds", self.world_probe_terrain_spec_text)
        self.assertIn("centerEdgeMaxDeltaStuds", self.world_probe_terrain_spec_text)
        self.assertIn("localTerrain = nil", self.world_probe_text)
        self.assertIn("localExperiencePayload.localTerrain = payload.localTerrain", self.world_probe_text)
        self.assertIn("WorldProbeTelemetryFlags.annotateMarkerPayload(compactPayload, telemetryFlags)", self.world_probe_text)
        self.assertIn("WorldProbeTelemetryFlags.annotateMarkerPayload(payload, telemetryFlags)", self.world_probe_text)
        self.assertIn(
            "WorldProbeTelemetryFlags.shapeLocalExperiencePayload(",
            self.world_probe_text,
        )
        self.assertIn(
            "WorldProbeTelemetryFlags.shapeLocalExperiencePayload(",
            self.world_probe_text,
        )
        self.assertIn('if WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "terrain") then', self.world_probe_text)
        self.assertIn('local playerLocalTelemetryEnabled = WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "player_local")', self.world_probe_text)
        self.assertIn('if WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "structures") then', self.world_probe_text)
        self.assertIn("localTerrain = localTerrain", self.world_probe_text)
        self.assertIn("local function sampleLocalTerrain(rootPart, worldRoot)", self.world_probe_text)
        self.assertIn("local samples = table.create(#LOCAL_TERRAIN_OFFSETS)", self.world_probe_text)
        self.assertIn("samples[index] = {", self.world_probe_text)
        self.assertIn("WorldProbeTerrain.summarizeTerrainSamples", self.world_probe_text)
        self.assertIn("samplePattern = samplePattern", self.world_probe_terrain_text)
        self.assertIn("sampleRadiusStuds = sampleRadiusStuds", self.world_probe_terrain_text)
        self.assertIn("sampleCount = sampleCount", self.world_probe_terrain_text)
        self.assertIn("missingSampleCount = missingSampleCount", self.world_probe_terrain_text)
        self.assertIn("missingEdgeSampleCount = missingEdgeSampleCount", self.world_probe_terrain_text)
        self.assertIn("status = status", self.world_probe_terrain_text)
        self.assertIn("sampleCount", self.world_probe_terrain_text)
        self.assertIn("local totalSlots = countSampleSlots(samples)", self.world_probe_terrain_text)
        self.assertIn("centerTerrainY = roundTenths(centerTerrainY)", self.world_probe_terrain_text)
        self.assertIn("edgeMeanTerrainY =", self.world_probe_terrain_text)
        self.assertIn("heightRangeStuds =", self.world_probe_terrain_text)
        self.assertIn("maxStepStuds =", self.world_probe_terrain_text)
        self.assertIn("meanAbsStepStuds =", self.world_probe_terrain_text)
        self.assertIn("edgeTerrainYRangeStuds =", self.world_probe_terrain_text)
        self.assertIn("centerMinusEdgeMeanStuds =", self.world_probe_terrain_text)
        self.assertIn("centerEdgeMaxDeltaStuds =", self.world_probe_terrain_text)
        self.assertIn("WorldProbeTerrain.summarizeTerrainSamples", self.world_probe_terrain_spec_text)
        self.assertIn("edgeMeanTerrainY", self.world_probe_terrain_spec_text)
        self.assertIn("centerMinusEdgeMeanStuds", self.world_probe_terrain_spec_text)
        self.assertIn("edgeTerrainYRangeStuds", self.world_probe_terrain_spec_text)
        self.assertIn("missingEdgeSampleCount", self.world_probe_terrain_spec_text)

    def test_client_world_probe_emits_dedicated_local_experience_marker(self) -> None:
        self.assertIn('print("ARNIS_CLIENT_LOCAL_EXPERIENCE " .. localExperiencePayloadJson)', self.world_probe_text)
        self.assertIn('local playerLocalTelemetryEnabled = WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "player_local")', self.world_probe_text)
        self.assertIn('playerLocalTelemetryEnabled = false', self.world_probe_text)
        self.assertIn("localExperiencePayload.localSupport = payload.localSupport", self.world_probe_text)
        self.assertIn("localExperiencePayload.localTerrain = payload.localTerrain", self.world_probe_text)
        self.assertIn("localExperiencePayload.localEnclosure = payload.localEnclosure", self.world_probe_text)
        self.assertIn("localExperiencePayload.localRoofCover = payload.localRoofCover", self.world_probe_text)
        self.assertIn("bootstrapAttemptId = bootstrapPayload.bootstrapAttemptId,", self.world_probe_text)

    def test_client_world_probe_resamples_moving_players_more_aggressively(self) -> None:
        self.assertIn("local IDLE_SAMPLE_INTERVAL = 1.5", self.world_probe_text)
        self.assertIn("local MOVING_SAMPLE_INTERVAL = 0.5", self.world_probe_text)
        self.assertIn("local MOVING_RESAMPLE_DISTANCE = 8", self.world_probe_text)
        self.assertIn("local MOVING_SPEED_THRESHOLD = 4", self.world_probe_text)
        self.assertIn("local lastSampleWasMoving = false", self.world_probe_text)
        self.assertIn("local function resolveMovementAwareSampleCadence(rootPart)", self.world_probe_text)
        self.assertIn("rootPart.AssemblyLinearVelocity.Magnitude", self.world_probe_text)
        self.assertIn("if isMoving and not lastSampleWasMoving then", self.world_probe_text)
        self.assertIn("local sampleInterval, resampleDistance, isMoving =", self.world_probe_text)
        self.assertIn("if displacement < resampleDistance then", self.world_probe_text)

    def test_world_probe_telemetry_flags_contract_is_explicit_and_stable(self) -> None:
        self.assertIn('WorldProbeTelemetryFlags.WORKSPACE_ATTR = "ArnisTelemetryFamilies"', self.world_probe_flags_text)
        self.assertIn("function WorldProbeTelemetryFlags.parseTelemetryFamilies(value)", self.world_probe_flags_text)
        self.assertIn("function WorldProbeTelemetryFlags.isEnabled(telemetryFlags, family)", self.world_probe_flags_text)
        self.assertIn("function WorldProbeTelemetryFlags.annotateMarkerPayload(payload, telemetryFlags)", self.world_probe_flags_text)
        self.assertIn("function WorldProbeTelemetryFlags.shapeLocalExperiencePayload(payload, telemetryFlags, playerLocalTelemetryEnabled)", self.world_probe_flags_text)
        self.assertIn("payload.telemetryFamilies = enabledFamilies", self.world_probe_flags_text)
        self.assertIn("SUPPORTED_FAMILY_ORDER", self.world_probe_flags_text)
        self.assertIn("SUPPORTED_FAMILIES[familyName] = true", self.world_probe_flags_text)
        self.assertIn("annotateMarkerPayload", self.world_probe_flags_spec_text)
        self.assertIn("shapeLocalExperiencePayload", self.world_probe_flags_spec_text)
        self.assertIn("defaultLocalExperiencePayload.playerLocalTelemetryEnabled", self.world_probe_flags_spec_text)
        self.assertIn("requestedLocalExperiencePayload.playerLocalTelemetryEnabled", self.world_probe_flags_spec_text)
        self.assertIn("disabledAfterEnabledPayload.playerLocalTelemetryEnabled", self.world_probe_flags_spec_text)
        self.assertIn("telemetryFamilies", self.world_probe_flags_spec_text)

    def test_shaped_roof_closure_decks_are_marked_internal_support_not_visible_roof_truth(self) -> None:
        self.assertIn("local function applyRoofPartOptions(part, partOptions)", self.building_builder_text)
        self.assertIn("ArnisRoofClosureDeck = true", self.building_builder_text)
        self.assertIn("ArnisShellWallEvidence", self.building_builder_text)
        self.assertIn("ArnisShellWallEvidence", self.scene_audit_text)
        self.assertIn('descendant.Name == "MergedShellRooflineCue"', self.scene_audit_text)
        self.assertIn('descendant.Name == "MergedShellPerimeterCue"', self.scene_audit_text)
        self.assertIn("part.Transparency = partOptions.transparency", self.building_builder_text)

    def test_scene_audit_tracks_visible_detail_and_facade_truth_separately_from_total_parts(self) -> None:
        self.assertIn('instance.Name == "MergedShellWindowPaneCue"', self.scene_audit_text)
        self.assertIn("buildingVisibleDetailPartCount", self.scene_audit_text)
        self.assertIn("buildingVisibleFacadePartCount", self.scene_audit_text)
        self.assertIn("visibleDetailParts = 0", self.scene_audit_text)
        self.assertIn("visibleFacadeParts = 0", self.scene_audit_text)
        self.assertIn("scene.buildingVisibleDetailPartCount += visibleDetailParts", self.scene_audit_text)
        self.assertIn("scene.buildingVisibleFacadePartCount += visibleFacadeParts", self.scene_audit_text)


if __name__ == "__main__":
    unittest.main()
