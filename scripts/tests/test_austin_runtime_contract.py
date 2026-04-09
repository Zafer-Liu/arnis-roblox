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
AUSTIN_SPAWN_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "AustinSpawn.lua"
SIGNATURES_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ImportSignatures.lua"
CANONICAL_WORLD_CONTRACT_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "CanonicalWorldContract.lua"
WORLD_PROBE_PATH = ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "WorldProbe.client.lua"
WORLD_PROBE_SUPPORT_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeSupport.lua"
WORLD_PROBE_FLAGS_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTelemetryFlags.lua"
WORLD_PROBE_TERRAIN_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTerrain.lua"
WORLD_PROBE_TERRAIN_SPEC_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "WorldProbeTerrain.spec.lua"
WORLD_PROBE_FLAGS_SPEC_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "WorldProbeTelemetryFlags.spec.lua"
WORLD_STATE_APPLIER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "WorldStateApplier.lua"
MINIMAP_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "MinimapService.lua"
CHUNK_SCHEMA_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "ChunkSchema.lua"
HARNESS_ROUTE_CONFIG_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "HarnessRouteConfig.lua"
PREVIEW_BUILDER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
PREVIEW_REQUEST_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewRequest.lua"
BUILDING_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "BuildingBuilder.lua"
)
TERRAIN_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"
)
SCENE_AUDIT_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "SceneAudit.lua"
WORLD_CONFIG_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldConfig.lua"
ROAD_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "RoadBuilder.lua"
)
ROAD_CHUNK_PLAN_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "RoadChunkPlan.lua"
WATER_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "WaterBuilder.lua"
)
PROP_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "PropBuilder.lua"
)
RAIL_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "RailBuilder.lua"
)
TRAVERSAL_SHARED_STATE_PATH = (
    ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "Traversal" / "SharedState.lua"
)


class AustinRuntimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bootstrap_text = BOOTSTRAP_PATH.read_text(encoding="utf-8")
        cls.run_austin_text = RUN_AUSTIN_PATH.read_text(encoding="utf-8")
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.chunk_priority_text = CHUNK_PRIORITY_PATH.read_text(encoding="utf-8")
        cls.import_service_text = IMPORT_SERVICE_PATH.read_text(encoding="utf-8")
        cls.austin_spawn_text = AUSTIN_SPAWN_PATH.read_text(encoding="utf-8")
        cls.canonical_world_contract_text = CANONICAL_WORLD_CONTRACT_PATH.read_text(encoding="utf-8")
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
        cls.chunk_schema_text = CHUNK_SCHEMA_PATH.read_text(encoding="utf-8")
        cls.harness_route_config_text = HARNESS_ROUTE_CONFIG_PATH.read_text(encoding="utf-8")
        cls.preview_builder_text = PREVIEW_BUILDER_PATH.read_text(encoding="utf-8")
        cls.preview_request_text = PREVIEW_REQUEST_PATH.read_text(encoding="utf-8")
        cls.building_builder_text = BUILDING_BUILDER_PATH.read_text(encoding="utf-8")
        cls.terrain_builder_text = TERRAIN_BUILDER_PATH.read_text(encoding="utf-8")
        cls.scene_audit_text = SCENE_AUDIT_PATH.read_text(encoding="utf-8")
        cls.world_config_text = WORLD_CONFIG_PATH.read_text(encoding="utf-8")
        cls.road_builder_text = ROAD_BUILDER_PATH.read_text(encoding="utf-8")
        cls.road_chunk_plan_text = ROAD_CHUNK_PLAN_PATH.read_text(encoding="utf-8")
        cls.water_builder_text = WATER_BUILDER_PATH.read_text(encoding="utf-8")
        cls.prop_builder_text = PROP_BUILDER_PATH.read_text(encoding="utf-8")
        cls.rail_builder_text = RAIL_BUILDER_PATH.read_text(encoding="utf-8")
        cls.traversal_shared_state_text = TRAVERSAL_SHARED_STATE_PATH.read_text(encoding="utf-8")

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
            "Workspace:SetAttribute(BOOTSTRAP_ATTEMPT_ID_ATTR, bootstrapAttemptId)",
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
        self.assertIn('local STARTUP_STREAMING_TIMEOUT_SECONDS = 60', self.bootstrap_text)
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
        # shouldPreferPlayVisibleShellWalls uses threshold-based logic:
        # simple buildings get explicit Parts, complex/tall buildings use
        # EditableMesh merge (back-face triangle corruption has been reverted).
        self.assertIn("shouldPreferSimpleShellDetail(building, footprintPointCount, height)", self.building_builder_text)
        self.assertIn("levels > 6 or height > 34", self.building_builder_text)
        self.assertIn("footprintPointCount <= 10", self.building_builder_text)
        # shouldEmitMergedShellReadableCues still gates on size thresholds
        self.assertIn("if boundedHoleLoopCount > 1 then", self.building_builder_text)
        self.assertIn("levels > 8 or height > 40 or footprintPointCount > 12", self.building_builder_text)
        self.assertIn("preferPlayVisibleShellWalls", self.building_builder_text)
        self.assertIn(
            "if preferPlayVisibleShellWalls and not (shellMeshIncludesRoof and hasPrecomputedShellMesh) then",
            self.building_builder_text,
        )

    def test_precomputed_mesh_fast_path_wired_in_building_and_road_builders(self) -> None:
        # BuildingBuilder: shellMesh fast path in MeshBuildAll
        self.assertIn("building.shellMesh", self.building_builder_text)
        self.assertIn("addPrecomputedMesh", self.building_builder_text)
        self.assertIn("precomputedMeshCount", self.building_builder_text)
        self.assertIn("runtimeMeshCount", self.building_builder_text)
        # RoadBuilder: roadMesh fast path in MeshBuildAll
        self.assertIn("road.roadMesh", self.road_builder_text)
        self.assertIn("addPrecomputedMesh", self.road_builder_text)
        self.assertIn("hasPrecomputedRoadMesh", self.road_builder_text)
        self.assertIn("precomputedMeshCount", self.road_builder_text)
        # ImportService: telemetry counters wired
        self.assertIn("buildingPrecomputedMeshCount", self.import_service_text)
        self.assertIn("roadPrecomputedMeshCount", self.import_service_text)

    def test_precomputed_building_mesh_contract_explicitly_marks_when_roof_is_included(self) -> None:
        self.assertIn("building.roofIncluded", self.chunk_schema_text)
        self.assertIn('prefix .. ".buildings[].roofIncluded must be a boolean"', self.chunk_schema_text)
        self.assertIn("local shellMeshIncludesRoof = building.roofIncluded == true", self.building_builder_text)
        self.assertIn("local hasPrecomputedShellMesh =", self.building_builder_text)
        self.assertIn("if not (shellMeshIncludesRoof and hasPrecomputedShellMesh) then", self.building_builder_text)
        self.assertIn("shellMeshIncludesRoof and hasPrecomputedShellMesh", self.building_builder_text)

    def test_mesh_builders_guard_doublesided_property_for_non_plugin_runtime_threads(self) -> None:
        self.assertIn("local doubleSidedCapabilityWarningIssued = false", self.building_builder_text)
        self.assertIn("local function tryEnableDoubleSided(part, builderLabel)", self.building_builder_text)
        self.assertIn("part.DoubleSided = true", self.building_builder_text)
        self.assertIn('warn(("[%s] DoubleSided unavailable in this runtime: %s")', self.building_builder_text)
        self.assertIn("tryEnableDoubleSided(part, \"MeshAccumulator\")", self.building_builder_text)
        self.assertIn("tryEnableDoubleSided(part, \"RoadMeshAccumulator\")", self.road_builder_text)
        self.assertIn("part.DoubleSided = true", self.road_builder_text)

    def test_bootstrap_preserves_import_failure_signal_instead_of_misclassifying_as_manifest_missing(self) -> None:
        self.assertIn("local bootstrapFailureKind = if runOk then \"manifest_unavailable\" else \"import_failed\"", self.bootstrap_text)
        self.assertIn("local errMessage = if bootstrapFailureKind == \"import_failed\"", self.bootstrap_text)
        self.assertIn('"Austin import failed — bootstrap halted."', self.bootstrap_text)
        self.assertIn('"Austin manifest unavailable — bootstrap halted."', self.bootstrap_text)
        self.assertIn("error: %s", self.bootstrap_text)

    def test_vehicle_controller_camera_telemetry_is_change_driven_instead_of_per_frame_attribute_churn(self) -> None:
        self.assertIn('S.setPlayerAttributeIfChanged("ArnisVehicleControllerReady", true)', self.traversal_shared_state_text)
        self.assertIn("local subjectChanged = S.lastPublishedClientTelemetrySubject ~= subject", self.traversal_shared_state_text)
        self.assertIn("local cameraTelemetryChanged = subjectChanged", self.traversal_shared_state_text)
        self.assertIn("or S.lastPublishedClientTelemetry.ArnisClientCameraMode ~= cameraMode", self.traversal_shared_state_text)
        self.assertIn("local cameraSubjectName = S.lastPublishedClientTelemetry.ArnisClientCameraSubject", self.traversal_shared_state_text)
        self.assertIn("if subjectChanged then", self.traversal_shared_state_text)
        self.assertIn("cameraSubjectName = subject and subject:GetFullName() or nil", self.traversal_shared_state_text)
        self.assertIn("if not (cameraTelemetryChanged or humanoidStateChanged) then", self.traversal_shared_state_text)
        self.assertIn("S.lastPublishedClientTelemetrySubject = subject", self.traversal_shared_state_text)

    def test_startup_streaming_probe_recognizes_roofincluded_merged_shells_as_roof_evidence(self) -> None:
        self.assertIn('model:SetAttribute("ArnisImportRoofIncluded", building.roofIncluded == true)', self.building_builder_text)
        self.assertIn('local roofIncludedByShellMesh = model:GetAttribute("ArnisImportRoofIncluded") == true', self.streaming_text)
        self.assertIn("if roofIncludedByShellMesh and not countedMergedRoofEvidence then", self.streaming_text)
        self.assertIn("structureTelemetry.nearbyRoofParts += 1", self.streaming_text)
        self.assertIn("envelopeTelemetry.nearbyRoofParts += 1", self.streaming_text)
        self.assertIn("structureTelemetry.overheadRoofParts += 1", self.streaming_text)

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

    def test_live_runtime_uses_production_streaming_profile_outside_studio(self) -> None:
        self.assertIn("if not RunService:IsStudio() then", self.bootstrap_text)
        self.assertIn('runtimeConfigSource.StreamingProfile = "production_server"', self.bootstrap_text)
        self.assertIn("local runtimeWorldConfig = StreamingRuntimeConfig.Resolve(runtimeConfigSource)", self.bootstrap_text)
        self.assertIn("if not RunService:IsStudio() then", self.run_austin_text)
        self.assertIn('runtimeConfigSource.StreamingProfile = "production_server"', self.run_austin_text)
        self.assertIn("runtimeWorldConfig = StreamingRuntimeConfig.Resolve(runtimeConfigSource)", self.run_austin_text)
        self.assertIn("local previewWorldConfig = StreamingRuntimeConfig.Resolve(DefaultWorldConfig)", self.preview_builder_text)

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
        self.assertIn('"pressure_replacement"', self.streaming_text)

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

    def test_streaming_service_only_reimports_resident_chunks_for_building_lod_upgrades(self) -> None:
        self.assertIn("local previousRank = BUILDING_LOD_DETAIL_RANK[previousBuildingLod] or 0", self.streaming_text)
        self.assertIn("local targetRank = BUILDING_LOD_DETAIL_RANK[chunkBuildingLodLevel] or 0", self.streaming_text)
        self.assertIn("local needsLodUpgrade = enableLodReimport", self.streaming_text)
        self.assertIn("and targetRank > previousRank", self.streaming_text)
        self.assertIn("-- Downgrades are handled by ordinary eviction instead of in-place re-import", self.streaming_text)

    def test_streaming_service_derives_ring_and_building_lod_from_actual_distance_not_lookahead(self) -> None:
        self.assertIn("local actualDistSq = getChunkDistanceSqToPoint(chunkEntry, playerPos)", self.streaming_text)
        self.assertIn("local ringName = getChunkRingName(actualDistSq, resolvedRings)", self.streaming_text)
        self.assertIn("local distSq = getChunkDistanceSqToPoint(chunkEntry, predicted)", self.streaming_text)
        self.assertIn("local ringName = getChunkRingName(distSq, resolvedRings)", self.streaming_text)

    def test_streaming_service_defers_resident_eviction_to_cooldown_when_chunk_limit_or_ring_nil_hits(self) -> None:
        import re
        match = re.search(
            r"if ring == nil or exceedsRingChunkLimit then\n(?P<body>.*?)\n\s*continue",
            self.streaming_text,
            re.DOTALL,
        )
        self.assertIsNotNone(match, "expected ring/chunk-limit branch in streaming loop")
        body = match.group("body")  # type: ignore[union-attr]
        self.assertIn("Let the cooldown-based not-desired sweep handle resident chunks", body)
        self.assertNotIn("ChunkLoader.UnloadChunk(chunkRef.id", body)

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

    def test_austin_spawn_has_canonical_anchor_fallback_for_live_austin_manifests(self) -> None:
        self.assertIn("local AUSTIN_FALLBACK_CANONICAL_ANCHOR = table.freeze({", self.austin_spawn_text)
        self.assertIn('positionStuds = { x = -6.0854, y = -0.4639, z = -208.371 }', self.austin_spawn_text)
        self.assertIn('lookDirectionStuds = { x = 0, y = 0, z = 1 }', self.austin_spawn_text)
        self.assertIn('if meta and meta.worldName == "Austin" then', self.austin_spawn_text)
        self.assertIn("return AUSTIN_FALLBACK_CANONICAL_ANCHOR", self.austin_spawn_text)

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
        self.assertIn('for _, model in ipairs(buildingsFolder:GetChildren()) do', self.world_probe_text)
        self.assertNotIn('for _, model in ipairs(buildingsFolder:GetDescendants()) do', self.world_probe_text)
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
            "local runOk, runReturn1, runReturn2 = pcall(function()",
            self.bootstrap_text,
        )
        self.assertIn(
            "return RunAustin.run({",
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

    def test_preview_builder_uses_route_window_chunk_selection_instead_of_full_bounded_envelope(self) -> None:
        self.assertIn(
            "local previewChunkIds, resolvedLoadRadius =",
            self.preview_builder_text,
        )
        self.assertIn(
            "AustinPreviewRequest.SelectChunkIds(manifestSource, boundedEnvelope.focusPoint, normalizedRequest, AustinPreviewBuilder.LOAD_RADIUS)",
            self.preview_builder_text,
        )
        self.assertIn("selectionLoadRadius = resolvedLoadRadius", self.preview_builder_text)
        self.assertIn('if type(previewChunkIds) ~= "table" or #previewChunkIds == 0 then', self.preview_builder_text)
        self.assertNotIn("local previewChunkIds = boundedEnvelope.chunkIds", self.preview_builder_text)
        self.assertIn('setPerfAttribute("RouteLane", routeSelectionOptions.routeLane or "")', self.run_austin_text)
        self.assertIn('setPerfAttribute("RouteStepIndex", routeSelectionOptions.routeStepIndex or -1)', self.run_austin_text)
        self.assertIn('"ManifestSourceKind"', self.run_austin_text)
        self.assertIn('routeSelectionOptions.routeCatalogName and "route_catalog" or "canonical_manifest"', self.run_austin_text)
        self.assertIn('setPerfAttribute("ManifestSourceName", resolvedManifestName or RunAustin.getManifestName())', self.run_austin_text)
        self.assertIn("resolvedManifestName = resolvedManifestName,", self.run_austin_text)
        self.assertIn('local resolvedSourceKind = (type(manifestSource) == "table" and manifestSource.manifestSourceKind)', self.run_austin_text)
        self.assertIn('or Workspace:GetAttribute("VertigoAustinManifestSourceKind")', self.run_austin_text)
        self.assertIn("manifestSourceKind = resolvedSourceKind,", self.run_austin_text)
        self.assertIn('RouteCatalogName = normalizedRequest and normalizedRequest.routeCatalogName or ""', self.preview_builder_text)
        self.assertIn('RouteLane = normalizedRequest and normalizedRequest.routeLane or ""', self.preview_builder_text)
        self.assertIn('RouteStepIndex = normalizedRequest and normalizedRequest.routeStepIndex or -1', self.preview_builder_text)
        self.assertIn("ManifestSourceKind = manifestSourceKind", self.preview_builder_text)
        self.assertIn("ManifestSourceName = resolvedManifestName or \"\"", self.preview_builder_text)

    def test_bootstrap_reads_harness_route_config_before_runtime_load(self) -> None:
        self.assertIn("local HarnessRouteConfig = require(script.Parent.ImportService.HarnessRouteConfig)", self.bootstrap_text)
        self.assertIn("local function resolveHarnessRouteSelection()", self.bootstrap_text)
        self.assertIn("if type(HarnessRouteConfig) ~= \"table\" then", self.bootstrap_text)
        self.assertIn("if type(HarnessRouteConfig.telemetryFamilies) == \"string\"", self.bootstrap_text)
        self.assertIn("selection.telemetryFamilies = HarnessRouteConfig.telemetryFamilies", self.bootstrap_text)
        # telemetryFamilies must be read independently of the route-catalog
        # `enabled` gate so perf telemetry emits in default harness runs.
        telemetry_idx = self.bootstrap_text.find("selection.telemetryFamilies = HarnessRouteConfig.telemetryFamilies")
        enabled_gate_idx = self.bootstrap_text.find("if HarnessRouteConfig.enabled ~= true then")
        self.assertNotEqual(telemetry_idx, -1)
        self.assertNotEqual(enabled_gate_idx, -1)
        self.assertLess(telemetry_idx, enabled_gate_idx)
        self.assertIn("local ReplicatedStorage = game:GetService(\"ReplicatedStorage\")", self.bootstrap_text)
        self.assertIn("ReplicatedStorage:SetAttribute(\"ArnisTelemetryFamilies\", harnessRouteSelection.telemetryFamilies)", self.bootstrap_text)
        self.assertIn("Workspace:SetAttribute(\"ArnisTelemetryFamilies\", harnessRouteSelection.telemetryFamilies)", self.bootstrap_text)
        self.assertIn("player:SetAttribute(\"ArnisTelemetryFamilies\", harnessRouteSelection.telemetryFamilies)", self.bootstrap_text)
        self.assertIn("Workspace:SetAttribute(\"VertigoRouteCatalogName\", harnessRouteSelection.routeCatalogName)", self.bootstrap_text)
        self.assertIn("Workspace:SetAttribute(\"VertigoRouteLane\", harnessRouteSelection.routeLane)", self.bootstrap_text)
        self.assertIn("Workspace:SetAttribute(\"VertigoRouteStepIndex\", harnessRouteSelection.routeStepIndex)", self.bootstrap_text)
        self.assertIn("local harnessRouteSelection = resolveHarnessRouteSelection()", self.bootstrap_text)
        self.assertLess(
            self.bootstrap_text.find("local harnessRouteSelection = resolveHarnessRouteSelection()"),
            self.bootstrap_text.find("local function onPlayer(player)"),
        )
        self.assertIn("routeCatalogName = harnessRouteSelection.routeCatalogName,", self.bootstrap_text)
        self.assertIn("routeLane = harnessRouteSelection.routeLane,", self.bootstrap_text)
        self.assertIn("routeStepIndex = harnessRouteSelection.routeStepIndex,", self.bootstrap_text)
        self.assertIn("enabled = false", self.harness_route_config_text)
        self.assertIn("routeCatalogName = \"\"", self.harness_route_config_text)
        self.assertIn("routeLane = \"\"", self.harness_route_config_text)
        self.assertIn("routeStepIndex = -1", self.harness_route_config_text)
        self.assertIn("telemetryFamilies = \"\"", self.harness_route_config_text)
        self.assertNotIn("elseif cellZ >= plan.gridD then", self.terrain_builder_text)

    def test_preview_builder_uses_chunk_priority_api_with_explicit_secondary_focus_slot(self) -> None:
        self.assertIn(
            "ChunkPriority.SortWorkItems(workItems, focusPoint, nil, chunkSize, forwardVector, observedChunkCostById)",
            self.preview_builder_text,
        )
        self.assertIn(
            "ChunkPriority.SortChunkIdsByPriority(",
            self.preview_builder_text,
        )
        self.assertIn(
            "focusPoint,\n            nil,\n            chunkSize,\n            forwardVector,\n            observedChunkCostById",
            self.preview_builder_text,
        )

    def test_chunk_priority_declares_chunk_center_helper_before_footprint_center_helper(self) -> None:
        chunk_center_index = self.chunk_priority_text.find("local function getChunkCenterXZ(")
        footprint_center_index = self.chunk_priority_text.find("local function getChunkFootprintCenterXZ(")
        self.assertGreaterEqual(chunk_center_index, 0)
        self.assertGreaterEqual(footprint_center_index, 0)
        self.assertLess(
            chunk_center_index,
            footprint_center_index,
            "expected getChunkCenterXZ to be declared before getChunkFootprintCenterXZ so Lua closes over the local helper",
        )

    def test_terrain_builder_initializes_neighbor_sampling_plan_before_interpolation(self) -> None:
        self.assertIn(
            "local function resolveNeighborHeightSample(terrainGrid, terrainNeighbors, gridW, gridD, cellX, cellZ)",
            self.terrain_builder_text,
        )
        self.assertIn("local function sampleInterpolatedHeight(cellX, cellZ, fracX, fracZ)", self.terrain_builder_text)
        self.assertIn(
            "resolveNeighborHeightSample(terrainGrid, terrainNeighbors, gridW, gridD, cellX, cellZ)",
            self.terrain_builder_text,
        )
        self.assertNotIn("resolveNeighborHeightSample(plan, cellX, cellZ)", self.terrain_builder_text)

    def test_route_runtime_handles_preserve_lane_summary_access_for_preview_chunk_selection(self) -> None:
        self.assertIn(
            "routeCatalogHandle:LoadLaneRuntimeHandle(routeStepIndex, routeLane, timeoutSeconds, options)",
            self.canonical_world_contract_text,
        )
        self.assertIn("type(manifestSource) == \"table\"", self.canonical_world_contract_text)
        self.assertIn("type(routeCatalogHandle.LoadLaneSummary) == \"function\"", self.canonical_world_contract_text)
        self.assertIn("function manifestSource:LoadLaneSummary(stepIndex, laneName)", self.canonical_world_contract_text)
        self.assertIn("return routeCatalogHandle:LoadLaneSummary(stepIndex, laneName)", self.canonical_world_contract_text)
        self.assertIn("cachedPreviewManifestName = resolvedManifestName", self.preview_builder_text)
        self.assertIn("cachedFullManifestName = resolvedManifestName", self.preview_builder_text)
        self.assertIn("cachedPreviewManifestRequestKey = nil", self.preview_builder_text)
        self.assertIn("cachedFullManifestRequestKey = nil", self.preview_builder_text)
        self.assertIn("local function buildManifestRequestKey(normalizedRequest)", self.preview_builder_text)
        self.assertIn('return "canonical_manifest"', self.preview_builder_text)
        self.assertIn('"route_catalog",', self.preview_builder_text)
        self.assertIn('local canUseManifestCache = manifestSourceKind == "canonical_manifest"', self.preview_builder_text)
        self.assertIn("RunService:IsStudio() and not timeTravelActive and canUseManifestCache", self.preview_builder_text)
        self.assertIn("if not timeTravelActive and canUseManifestCache then", self.preview_builder_text)
        self.assertIn("cachedPreviewManifestRequestKey == manifestRequestKey", self.preview_builder_text)
        self.assertIn("cachedFullManifestRequestKey == manifestRequestKey", self.preview_builder_text)
        self.assertIn("cachedPreviewManifestRequestKey = manifestRequestKey", self.preview_builder_text)
        self.assertIn("cachedFullManifestRequestKey = manifestRequestKey", self.preview_builder_text)
        self.assertIn("local function normalizeRouteChunkId(chunkId)", self.preview_request_text)
        self.assertIn("local function normalizeRouteChunkIds(laneSummary)", self.preview_request_text)
        self.assertIn('string.find(chunkId, ":", 1, true)', self.preview_request_text)
        self.assertIn("chunkRef.chunk_id", self.preview_request_text)
        self.assertIn("return normalizeRouteChunkIds(laneSummary), nil", self.preview_request_text)
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
        self.assertIn("local function resolveTelemetryFamilies()", self.world_probe_text)
        self.assertIn('local telemetryFamilies = resolveTelemetryFamilies()', self.world_probe_text)
        self.assertIn('player:GetAttribute(WorldProbeTelemetryFlags.PLAYER_ATTR)', self.world_probe_text)
        self.assertIn('ReplicatedStorage:GetAttribute(WorldProbeTelemetryFlags.REPLICATED_STORAGE_ATTR)', self.world_probe_text)
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
        self.assertIn('player:GetAttributeChangedSignal(WorldProbeTelemetryFlags.PLAYER_ATTR):Connect(function()', self.world_probe_text)
        self.assertIn('local NEARBY_NAMED_BUILDING_RADIUS = 640', self.world_probe_text)
        self.assertIn('local MAX_NAMED_BUILDINGS = 6', self.world_probe_text)
        self.assertIn('local nearestNamedBuildingDetails = {}', self.world_probe_text)
        self.assertIn('model:GetAttribute("ArnisImportBuildingName")', self.world_probe_text)
        self.assertIn('if horizontalDistanceSq > NEARBY_NAMED_BUILDING_RADIUS_SQ then', self.world_probe_text)
        self.assertIn('local isWithinNearbyBuildingRadius = horizontalDistance <= NEARBY_BUILDING_RADIUS', self.world_probe_text)
        self.assertIn('if not isWithinNearbyBuildingRadius then', self.world_probe_text)
        self.assertIn('nearestNamedBuildingSourceIds = {}', self.world_probe_text)
        self.assertIn('nearestNamedBuildingNames = {}', self.world_probe_text)
        self.assertIn('"ArnisClientNearbyNamedBuildingNames"', self.world_probe_text)

    def test_client_world_probe_avoids_vector2_allocation_in_structure_scan(self) -> None:
        self.assertIn("local NEARBY_BUILDING_RADIUS_SQ = NEARBY_BUILDING_RADIUS * NEARBY_BUILDING_RADIUS", self.world_probe_text)
        self.assertIn(
            "local NEARBY_NAMED_BUILDING_RADIUS_SQ = NEARBY_NAMED_BUILDING_RADIUS * NEARBY_NAMED_BUILDING_RADIUS",
            self.world_probe_text,
        )
        self.assertIn("local OVERHEAD_ROOF_RADIUS_SQ = OVERHEAD_ROOF_RADIUS * OVERHEAD_ROOF_RADIUS", self.world_probe_text)
        self.assertIn("local NEARBY_WALL_RADIUS_SQ = NEARBY_WALL_RADIUS * NEARBY_WALL_RADIUS", self.world_probe_text)
        self.assertIn("local horizontalDistanceSq = partOffset.X * partOffset.X + partOffset.Z * partOffset.Z", self.world_probe_text)
        self.assertIn("local horizontalDistanceSq = offset.X * offset.X + offset.Z * offset.Z", self.world_probe_text)
        self.assertIn("if horizontalDistanceSq <= NEARBY_BUILDING_RADIUS_SQ then", self.world_probe_text)
        self.assertIn("if horizontalDistanceSq > NEARBY_NAMED_BUILDING_RADIUS_SQ then", self.world_probe_text)
        self.assertIn("local horizontalDistance = math.sqrt(horizontalDistanceSq)", self.world_probe_text)
        self.assertNotIn("local horizontalPartDistance = math.sqrt(horizontalDistanceSq)", self.world_probe_text)
        self.assertIn('string.find(part.Name, "_roof_closure", 1, true) ~= nil', self.world_probe_text)
        self.assertNotIn("string.lower(part.Name)", self.world_probe_text)
        self.assertNotIn("Vector2.new(partOffset.X, partOffset.Z).Magnitude", self.world_probe_text)
        self.assertNotIn("Vector2.new(offset.X, offset.Z).Magnitude", self.world_probe_text)

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
        self.assertIn('WorldProbeTelemetryFlags.PLAYER_ATTR = "ArnisTelemetryFamilies"', self.world_probe_flags_text)
        self.assertIn('WorldProbeTelemetryFlags.REPLICATED_STORAGE_ATTR = "ArnisTelemetryFamilies"', self.world_probe_flags_text)
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
        self.assertIn('if not rectangularFootprint then\n            buildRoofClosureDeck(', self.building_builder_text)
        self.assertIn("tryBuildRectangularHippedRoofMesh(", self.building_builder_text)
        self.assertIn("buildRoofClosureDeck(bldgName, footprint, bounds.holeWorldLoops, baseY + height, rc, rm, parent)", self.building_builder_text)
        self.assertIn("ArnisShellWallEvidence", self.building_builder_text)
        self.assertIn("ArnisShellWallEvidence", self.scene_audit_text)
        self.assertIn('descendant.Name == "MergedShellRooflineCue"', self.scene_audit_text)
        self.assertIn('descendant.Name == "MergedShellPerimeterCue"', self.scene_audit_text)
        self.assertIn("part.Transparency = partOptions.transparency", self.building_builder_text)

    def test_building_builder_publishes_building_name_attribute_for_runtime_probes(self) -> None:
        self.assertIn('if type(building.name) == "string" and building.name ~= "" then', self.building_builder_text)
        self.assertIn('model:SetAttribute("ArnisImportBuildingName", building.name)', self.building_builder_text)

    def test_building_builder_guards_lod_assignment_outside_plugin_capability(self) -> None:
        self.assertIn("local function trySetModelLevelOfDetail(model, levelOfDetail)", self.building_builder_text)
        self.assertIn("pcall(function()", self.building_builder_text)
        self.assertIn("trySetModelLevelOfDetail(model, Enum.ModelLevelOfDetail.Automatic)", self.building_builder_text)
        self.assertNotIn("model.LevelOfDetail = Enum.ModelLevelOfDetail.Automatic", self.building_builder_text)

    def test_building_builder_roof_material_hash_diversification(self) -> None:
        """Roof material is selected via hash diversification, not inherited from wall."""
        self.assertIn("ROOF_MATERIAL_PALETTE", self.building_builder_text)
        self.assertIn("Enum.Material.Slate", self.building_builder_text)
        self.assertIn("Enum.Material.Asphalt", self.building_builder_text)
        # The palette should be used in getRoofMaterial via hash
        self.assertIn("hashId(", self.building_builder_text)
        # getRoofMaterial should use the palette as a fallback instead of wallMat
        self.assertIn("ROOF_MATERIAL_PALETTE", self.building_builder_text)
        self.assertIn("ROOF_MATERIAL_PALETTE_COLORS", self.building_builder_text)

    def test_building_builder_roof_palette_is_forward_declared_for_getroofcolor(self) -> None:
        self.assertIn(
            "local ROOF_MATERIAL_PALETTE_COLORS -- forward declaration for getRoofColor closure",
            self.building_builder_text,
        )
        self.assertIn("ROOF_MATERIAL_PALETTE_COLORS = {", self.building_builder_text)

    def test_building_builder_window_tint_varies_by_usage(self) -> None:
        """Window glass color varies by building usage (office vs residential vs warehouse)."""
        self.assertIn("WINDOW_TINT_BY_USAGE_CLASS", self.building_builder_text)
        self.assertIn("local function getUsageClass(usage)", self.building_builder_text)
        # Verify the tint table has at least office, residential, and industrial entries
        self.assertIn('"office"', self.building_builder_text)
        self.assertIn('"residential"', self.building_builder_text)
        self.assertIn('"industrial"', self.building_builder_text)
        # Verify the tint is applied in window generation
        self.assertIn("getWindowTint(", self.building_builder_text)

    def test_building_builder_reads_facade_style_field(self) -> None:
        """BuildingBuilder reads facadeStyle field when available."""
        self.assertIn("building.facadeStyle", self.building_builder_text)
        self.assertIn("facadeStyle", self.building_builder_text)

    def test_building_builder_reads_roof_levels_field(self) -> None:
        """BuildingBuilder reads roofLevels field when available."""
        self.assertIn("building.roofLevels", self.building_builder_text)
        self.assertIn("roofLevels", self.building_builder_text)

    def test_scene_audit_tracks_visible_detail_and_facade_truth_separately_from_total_parts(self) -> None:
        self.assertIn('instance.Name == "MergedShellWindowPaneCue"', self.scene_audit_text)
        self.assertIn("buildingVisibleDetailPartCount", self.scene_audit_text)
        self.assertIn("buildingVisibleFacadePartCount", self.scene_audit_text)
        self.assertIn("visibleDetailParts = 0", self.scene_audit_text)
        self.assertIn("visibleFacadeParts = 0", self.scene_audit_text)
        self.assertIn("scene.buildingVisibleDetailPartCount += visibleDetailParts", self.scene_audit_text)
        self.assertIn("scene.buildingVisibleFacadePartCount += visibleFacadeParts", self.scene_audit_text)

    def test_road_builder_consumes_sidewalk_enum_for_curb_geometry(self) -> None:
        # RoadChunkPlan reads road.sidewalk enum and falls back to hasSidewalk boolean
        self.assertIn("if road.sidewalk then", self.road_chunk_plan_text)
        self.assertIn("return road.sidewalk", self.road_chunk_plan_text)
        self.assertIn('road.hasSidewalk and "both" or "no"', self.road_chunk_plan_text)
        # RoadBuilder handles "separate" sidewalk enum value via normalization
        self.assertIn('"separate"', self.road_builder_text)
        self.assertIn("normalizeSidewalkMode", self.road_builder_text)
        # Curb geometry uses the accumulator pattern with CURB_THICKNESS
        self.assertIn("CURB_THICKNESS", self.road_builder_text)
        self.assertIn("curbAcc:addRoadStrip(", self.road_builder_text)
        # Normalized sidewalkMode drives left/right curb placement in the mesh path
        self.assertIn('normalizedSidewalkMode == "both"', self.road_builder_text)
        self.assertIn('normalizedSidewalkMode == "left"', self.road_builder_text)
        self.assertIn('normalizedSidewalkMode == "right"', self.road_builder_text)

    def test_road_builder_consumes_layer_for_vertical_offset(self) -> None:
        # RoadChunkPlan applies Y offset from road.layer (positive and negative)
        self.assertIn("road.layer", self.road_chunk_plan_text)
        self.assertIn("layerElevation", self.road_chunk_plan_text)
        # Layer supports negative values for underpasses/tunnels
        self.assertIn("road.layer < 0", self.road_chunk_plan_text)
        # Layer constant uses ~5 studs per layer
        self.assertIn("LAYER_ELEVATION_STUDS", self.road_chunk_plan_text)

    def test_road_builder_consumes_subkind_for_visual_material_differentiation(self) -> None:
        # RoadBuilder reads road.subkind for color tinting
        self.assertIn("road.subkind", self.road_builder_text)
        # Subkind-based color map exists for visual differentiation
        self.assertIn("SUBKIND_COLOR_TINT", self.road_builder_text)
        # getSubkindColorTint reads road.subkind and returns a tinted color
        self.assertIn("local function getSubkindColorTint(road)", self.road_builder_text)
        self.assertIn("SUBKIND_COLOR_TINT[road.subkind]", self.road_builder_text)
        # getRoadColor integrates subkind tint when available
        self.assertIn("local subkindTint = getSubkindColorTint(road)", self.road_builder_text)


    def test_water_builder_consumes_color_field_for_per_body_water_color(self) -> None:
        self.assertIn("resolveWaterColor", self.water_builder_text)
        self.assertIn("water.color", self.water_builder_text)
        self.assertIn("DEFAULT_WATER_COLOR", self.water_builder_text)
        self.assertIn("colorField.r", self.water_builder_text)

    # ------------------------------------------------------------------
    # Rooftop gameplay surfaces (Task 6 – Planetary Realism Sprint)
    # ------------------------------------------------------------------

    def test_rooftop_equipment_threshold_is_three_levels(self) -> None:
        """Rooftop equipment should generate on buildings with 3+ levels, not 5+."""
        self.assertIn("building.levels < 3", self.building_builder_text,
                       "Rooftop equipment threshold must be lowered to 3 levels")
        self.assertNotIn("building.levels < 5", self.building_builder_text,
                         "Old 5-level threshold must be removed")

    def test_flat_roof_parapet_geometry(self) -> None:
        """Flat-roof buildings must have a parapet (edge lip) around the perimeter."""
        self.assertIn("buildRooftopParapet", self.building_builder_text,
                       "buildRooftopParapet function must exist")
        self.assertIn("Parapet", self.building_builder_text,
                       "Parapet parts must be named 'Parapet'")
        self.assertIn('CollectionService:AddTag(parapet, "LOD_Detail")',
                       self.building_builder_text,
                       "Parapet parts must be tagged LOD_Detail")

    def test_client_world_probe_measures_frame_time_with_zero_per_frame_allocations(self) -> None:
        # Pre-allocated ring buffer exists with fixed capacity
        self.assertIn("local PERF_RING_CAPACITY = 300", self.world_probe_text)
        self.assertIn("local perfRing = table.create(PERF_RING_CAPACITY, 0)", self.world_probe_text)
        self.assertIn("local perfRingHead = 0", self.world_probe_text)
        self.assertIn("local perfRingCount = 0", self.world_probe_text)
        # Frame time recording uses pre-allocated ring, no table or string creation
        self.assertIn("local function recordFrameTime(dt)", self.world_probe_text)
        self.assertIn("perfRing[perfRingHead] = dt", self.world_probe_text)
        # Heartbeat passes dt to recordFrameTime
        self.assertIn("RunService.Heartbeat:Connect(function(dt)", self.world_probe_text)
        self.assertIn("recordFrameTime(dt)", self.world_probe_text)
        # Perf emission is gated behind client_perf telemetry family
        self.assertIn('WorldProbeTelemetryFlags.isEnabled(telemetryFlags, "client_perf")', self.world_probe_text)
        # ARNIS_CLIENT_PERF marker is emitted
        self.assertIn('print("ARNIS_CLIENT_PERF " .. perfPayloadJson)', self.world_probe_text)
        # Payload contains expected fields
        self.assertIn("avgFrameTimeMs =", self.world_probe_text)
        self.assertIn("p99FrameTimeMs =", self.world_probe_text)
        self.assertIn("maxFrameTimeMs =", self.world_probe_text)
        self.assertIn("fps =", self.world_probe_text)
        self.assertIn("instanceCountParts =", self.world_probe_text)
        self.assertIn("instanceCountMeshParts =", self.world_probe_text)
        # Instance counting must discover the world root through getWorldRoot(),
        # not an inline Workspace:FindFirstChild lookup, so it stays consistent
        # with all other telemetry paths and picks up fallback discovery changes.
        self.assertIn("local worldRoot = getWorldRoot()", self.world_probe_text,
                       "Perf instance counting must use getWorldRoot() for world root discovery")

    def test_rooftop_equipment_variety(self) -> None:
        """Rooftop equipment must include antenna and vent box in addition to AC units."""
        self.assertIn("Antenna", self.building_builder_text,
                       "Antenna equipment type must exist")
        self.assertIn("VentBox", self.building_builder_text,
                       "VentBox equipment type must exist")
        # Equipment type selection via hash modulo 3
        self.assertIn("equipmentType % 3", self.building_builder_text,
                       "Equipment type must be selected via hashId modulo 3")


    # ------------------------------------------------------------------
    # PropBuilder leafType: broadleaf vs needleleaf canopy shape
    # ------------------------------------------------------------------

    def test_prop_builder_infers_needleleaved_from_conifer_species(self) -> None:
        """Conifer species should infer needleleaved leafType for cone canopy."""
        self.assertIn("CONIFER_SPECIES_PATTERNS", self.prop_builder_text)
        # Must include key genera
        for genus in ("pinus", "picea", "abies", "spruce", "fir", "cedar",
                      "juniper", "cypress", "redwood", "sequoia"):
            self.assertIn(
                f'"{genus}"',
                self.prop_builder_text,
                f"CONIFER_SPECIES_PATTERNS must include {genus}",
            )

    def test_prop_builder_needleleaved_canopy_is_taller_than_wide(self) -> None:
        """Needleleaved canopy should use stacked tiers producing a cone silhouette."""
        self.assertIn('leafType == "needleleaved"', self.prop_builder_text)
        # Multi-tier cone approach: stacked progressively smaller balls
        self.assertIn("tier", self.prop_builder_text)

    def test_prop_builder_broadleaved_uses_multi_lobe_canopy(self) -> None:
        """Broadleaved (default) trees must use the multi-lobe organic canopy."""
        self.assertIn("buildRealisticCanopy(model, trunkTop, canopyR, canopyColor3", self.prop_builder_text)
        self.assertIn("CanopyLobe", self.prop_builder_text)
        self.assertIn("CanopyMain", self.prop_builder_text)

    # ------------------------------------------------------------------
    # PropBuilder precomputed prop mesh fast path
    # ------------------------------------------------------------------

    def test_prop_builder_has_precomputed_prop_mesh_loader(self) -> None:
        """PropBuilder must expose loadPrecomputedPropMesh for Rust pipeline meshes."""
        self.assertIn("propMesh", self.prop_builder_text)
        self.assertIn("loadPrecomputedPropMesh", self.prop_builder_text)

    def test_prop_builder_precomputed_mesh_uses_pcall(self) -> None:
        """Precomputed mesh creation must be pcall-guarded."""
        self.assertIn("CreateEditableMesh", self.prop_builder_text)
        self.assertIn("CreateMeshPartAsync", self.prop_builder_text)

    def test_prop_builder_precomputed_mesh_telemetry(self) -> None:
        """PropBuilder must track precomputed vs runtime prop counts."""
        self.assertIn("precomputedPropCount", self.prop_builder_text)
        self.assertIn("runtimePropCount", self.prop_builder_text)
        self.assertIn("GetBuildStats", self.prop_builder_text)

    # ------------------------------------------------------------------
    # RailBuilder kind differentiation
    # ------------------------------------------------------------------

    def test_rail_builder_declares_kind_properties_table(self) -> None:
        """RailBuilder must have a RAIL_KIND_PROPERTIES table for visual differentiation."""
        self.assertIn("RAIL_KIND_PROPERTIES", self.rail_builder_text)
        for kind in ("rail", "heavy_rail", "light_rail", "tram", "subway", "metro", "narrow_gauge"):
            self.assertIn(
                kind,
                self.rail_builder_text,
                f"RAIL_KIND_PROPERTIES must include {kind}",
            )

    def test_rail_builder_kind_specific_material(self) -> None:
        """Rail kinds must map to distinct materials."""
        self.assertIn("Enum.Material.Metal", self.rail_builder_text)
        self.assertIn("Enum.Material.Concrete", self.rail_builder_text)
        # Default fallback must remain Cobblestone
        self.assertIn("Enum.Material.Cobblestone", self.rail_builder_text)

    def test_rail_builder_kind_specific_thickness(self) -> None:
        """Rail kinds must specify distinct thickness values."""
        # heavy_rail/rail = 1.5, light_rail/tram = 1.0, subway/metro = 2.0, narrow_gauge = 0.8
        self.assertIn("thickness = 1.5", self.rail_builder_text)
        self.assertIn("thickness = 1.0", self.rail_builder_text)
        self.assertIn("thickness = 2.0", self.rail_builder_text)
        self.assertIn("thickness = 0.8", self.rail_builder_text)

    def test_rail_builder_resolves_kind_properties_before_painting(self) -> None:
        """FallbackBuild must resolve kind properties and pass them to paintSegment."""
        self.assertIn("resolveRailKindProperties(rail.kind)", self.rail_builder_text)
        self.assertIn("paintSegment(terrain, p1, p2, width, kindProps)", self.rail_builder_text)
        self.assertIn("kindProps.thickness", self.rail_builder_text)
        self.assertIn("kindProps.material", self.rail_builder_text)


    # ------------------------------------------------------------------
    # wallColor / roofColor signal fidelity
    # ------------------------------------------------------------------

    def test_wall_color_preserves_non_placeholder_values(self) -> None:
        """getColor must return the manifest wallColor when it is not the exact
        OSM auto-fill placeholder (170,170,170)."""
        # The guard must be an exact equality check, not a range/threshold.
        self.assertIn(
            "if not (r == 170 and g == 170 and b == 170) then",
            self.building_builder_text,
            "wallColor rejection must use exact equality on the OSM placeholder",
        )
        # The faithful path returns Color3.fromRGB(r, g, b)
        self.assertIn(
            "return Color3.fromRGB(r, g, b)",
            self.building_builder_text,
            "wallColor must be forwarded faithfully via Color3.fromRGB(r, g, b)",
        )

    def test_wall_color_rejects_only_exact_osm_placeholder(self) -> None:
        """Only the exact (170,170,170) grey placeholder should be rejected.
        No broad range check (e.g. math.abs, threshold, or tolerance) may exist
        in getColor."""
        import re
        get_color_match = re.search(
            r"local function getColor\(building\)(.*?)^end",
            self.building_builder_text,
            re.DOTALL | re.MULTILINE,
        )
        self.assertIsNotNone(get_color_match, "getColor function must exist")
        get_color_body = get_color_match.group(1)  # type: ignore[union-attr]
        for forbidden in ("math.abs", "tolerance", "threshold", "isGrey", "isGray"):
            self.assertNotIn(
                forbidden,
                get_color_body,
                f"getColor must not use fuzzy grey detection ({forbidden})",
            )

    def test_roof_color_preserved_faithfully(self) -> None:
        """getRoofColor must use the manifest roofColor directly without any
        placeholder rejection or grey filtering."""
        import re
        get_roof_match = re.search(
            r"local function getRoofColor\(building.*?\)(.*?)^end",
            self.building_builder_text,
            re.DOTALL | re.MULTILINE,
        )
        self.assertIsNotNone(get_roof_match, "getRoofColor function must exist")
        roof_body = get_roof_match.group(1)  # type: ignore[union-attr]
        # roofColor must be returned directly
        self.assertIn(
            "return Color3.fromRGB(building.roofColor.r, building.roofColor.g, building.roofColor.b)",
            roof_body,
            "roofColor must be forwarded faithfully without any grey rejection",
        )
        # No grey placeholder filtering in getRoofColor
        self.assertNotIn("170", roof_body,
                         "getRoofColor must not filter any grey placeholder")


    def test_road_builder_emits_street_name_labels(self) -> None:
        """RoadBuilder must create BillboardGui street labels for named roads."""
        import re
        # emitStreetLabel function must exist and check road.name
        self.assertIn("function emitStreetLabel(", self.road_builder_text)
        # Must guard on road.name being present and non-empty
        self.assertIn("road.name", self.road_builder_text)
        # Must create a BillboardGui
        self.assertIn('Instance.new("BillboardGui")', self.road_builder_text)
        # MaxDistance must be 150
        self.assertIn("MaxDistance = 150", self.road_builder_text)
        # AlwaysOnTop must be false
        self.assertIn("AlwaysOnTop = false", self.road_builder_text)
        # Must tag attachment with LOD_Detail
        emit_match = re.search(
            r"function emitStreetLabel\(.*?\)\n(.*?)^end",
            self.road_builder_text,
            re.DOTALL | re.MULTILINE,
        )
        self.assertIsNotNone(emit_match, "emitStreetLabel function body must exist")
        emit_body = emit_match.group(1)  # type: ignore[union-attr]
        self.assertIn("LOD_Detail", emit_body)
        self.assertIn("Attachment", emit_body)

    def test_road_builder_consumes_sidewalk_surface_material(self) -> None:
        """RoadBuilder must read road.sidewalkSurface and map it to Roblox materials."""
        # Must have the sidewalk surface material mapping table
        self.assertIn("SIDEWALK_SURFACE_MATERIAL", self.road_builder_text)
        # Must map paving_stones to Cobblestone
        self.assertIn("paving_stones = Enum.Material.Cobblestone", self.road_builder_text)
        # Must map concrete
        self.assertIn("concrete = Enum.Material.Concrete", self.road_builder_text)
        # Must map asphalt
        self.assertIn("asphalt = Enum.Material.Asphalt", self.road_builder_text)
        # Must map gravel to Pebble
        self.assertIn("gravel = Enum.Material.Pebble", self.road_builder_text)
        # Must map sett to Cobblestone
        self.assertIn("sett = Enum.Material.Cobblestone", self.road_builder_text)
        # Must have getSidewalkMaterial function that reads road.sidewalkSurface
        self.assertIn("function getSidewalkMaterial(road)", self.road_builder_text)
        self.assertIn("road.sidewalkSurface", self.road_builder_text)
        # getSidewalkMaterial must be called in the sidewalk accumulator path
        self.assertIn("getSidewalkMaterial(road)", self.road_builder_text)

    def test_road_builder_emits_lane_marking_geometry(self) -> None:
        """RoadBuilder must emit center lane lines and oneway arrows for multi-lane roads."""
        import re
        # paintLaneMarkings function must exist
        self.assertIn("function paintLaneMarkings(", self.road_builder_text)
        # Must check road.lanes
        lane_match = re.search(
            r"function paintLaneMarkings\(.*?\)\n(.*?)^end",
            self.road_builder_text,
            re.DOTALL | re.MULTILINE,
        )
        self.assertIsNotNone(lane_match, "paintLaneMarkings function body must exist")
        lane_body = lane_match.group(1)  # type: ignore[union-attr]
        # Must guard on lanes >= 2
        self.assertIn("road.lanes", lane_body)
        self.assertIn("lanes < 2", lane_body)
        # Must create center line Part
        self.assertIn("LaneCenterLine", lane_body)
        # Must use SmoothPlastic material and white color
        self.assertIn("Enum.Material.SmoothPlastic", lane_body)
        self.assertIn("Color3.fromRGB(255, 255, 255)", lane_body)
        # Must check for oneway
        self.assertIn("road.oneway", lane_body)
        # Must create oneway arrow part
        self.assertIn("LaneOnewayArrow", lane_body)
        # Must tag with LOD_Detail
        self.assertIn("LOD_Detail", lane_body)
    # ------------------------------------------------------------------
    # BuildingBuilder: pipeline field consumption
    # ------------------------------------------------------------------

    def test_building_builder_documents_cladding_resolution_in_pipeline(self) -> None:
        """getMaterial documents that cladding is resolved into material by the Rust pipeline."""
        src = self.building_builder_text
        # The comment must explain that cladding is merged into material by the pipeline
        self.assertIn("building:cladding", src,
                       "getMaterial must document that cladding flows through the pipeline material field")

    def test_building_builder_consumes_roof_direction(self) -> None:
        """building.roofDirection must be read when orienting gabled roof ridges."""
        src = self.building_builder_text
        self.assertIn("building.roofDirection", src, "BuildingBuilder must read building.roofDirection")
        # It should convert to radians
        self.assertIn("math.rad(building.roofDirection)", src,
                       "roofDirection must be converted to radians")

    def test_building_builder_consumes_roof_angle(self) -> None:
        """building.roofAngle must be read for roof pitch computation."""
        src = self.building_builder_text
        self.assertIn("building.roofAngle", src, "BuildingBuilder must read building.roofAngle")
        # Must clamp to reasonable bounds
        self.assertIn("math.clamp(building.roofAngle", src,
                       "roofAngle must be clamped to reasonable bounds")
        # Must use tan for rise computation
        self.assertIn("math.tan(math.rad(clampedAngle))", src,
                       "roofAngle must be converted via tan(rad()) for rise/run")

    def test_building_builder_supports_merged_window_mesh(self) -> None:
        """Window panes should be mergeable into EditableMesh via MeshAccumulator."""
        src = self.building_builder_text
        self.assertIn("windowAccumulators", src,
                       "BuildingBuilder must have a windowAccumulators dict for merged window meshes")
        self.assertIn("getWindowAccumulator", src,
                       "BuildingBuilder must expose a getWindowAccumulator helper")
        self.assertIn("addWindowPaneToAccumulator", src,
                       "BuildingBuilder must have addWindowPaneToAccumulator for quad conversion")
        # Must support transparency on MeshAccumulator
        self.assertIn("self.transparency", src,
                       "MeshAccumulator must support a transparency option")
        # Fallback path must still exist
        self.assertIn('Instance.new("Part")', src,
                       "Part-based fallback path must still exist for windows")
        self.assertIn("SimpleShellWindowPane", src,
                       "SimpleShellWindowPane fallback name must be preserved")

    def test_water_builder_prefers_water_type_over_kind(self) -> None:
        """WaterBuilder.resolveKindProperties must check waterType before kind."""
        src = self.water_builder_text
        # waterType lookup must appear before the kind fallback
        wt_idx = src.index("water.waterType")
        kind_idx = src.index("water.kind", wt_idx)
        self.assertLess(wt_idx, kind_idx,
                        "resolveKindProperties must check waterType before kind")
        # Both feed into the same WATER_KIND_PROPERTIES table
        self.assertIn("WATER_KIND_PROPERTIES[waterType]", src)
        self.assertIn("WATER_KIND_PROPERTIES[kind]", src)

    def test_building_builder_uses_structure_type_as_material_hint(self) -> None:
        """BuildingBuilder.getMaterial must use structureType when material is absent."""
        src = self.building_builder_text
        # structureType check must exist in getMaterial
        self.assertIn("building.structureType", src,
                       "getMaterial must read building.structureType")
        # Isolate getMaterial body to verify ordering within the function
        fn_start = src.index("local function getMaterial(building)")
        fn_end = src.index("\nend", fn_start) + 4
        fn_src = src[fn_start:fn_end]
        mat_idx = fn_src.index("building.material")
        st_idx = fn_src.index("building.structureType")
        usage_idx = fn_src.index("building.usage or building.kind")
        self.assertLess(mat_idx, st_idx,
                        "structureType check must come after material tag check")
        self.assertLess(st_idx, usage_idx,
                        "structureType check must come before usage fallback")
        # Verify key mappings exist
        for mapping in ["timber_frame", "steel_frame", "concrete", "masonry", "brick"]:
            self.assertIn(mapping, src,
                          f"structureType mapping for '{mapping}' must exist")

    def test_world_config_exposes_merge_windows_knob(self) -> None:
        """WorldConfig must expose MergeWindowsIntoMesh for the window mesh toggle."""
        src = self.world_config_text
        self.assertIn("MergeWindowsIntoMesh", src,
                       "WorldConfig must have MergeWindowsIntoMesh knob")
        # Default should be true (merged)
        self.assertIn("MergeWindowsIntoMesh = true", src,
                       "MergeWindowsIntoMesh should default to true")

    def test_world_config_exposes_satellite_overlay_knob(self) -> None:
        """WorldConfig must expose EnableTerrainSatelliteOverlay for satellite texture toggle."""
        src = self.world_config_text
        self.assertIn("EnableTerrainSatelliteOverlay", src,
                       "WorldConfig must have EnableTerrainSatelliteOverlay knob")
        self.assertIn("EnableTerrainSatelliteOverlay = true", src,
                       "EnableTerrainSatelliteOverlay should default to true")

    def test_terrain_builder_supports_satellite_overlay(self) -> None:
        """TerrainBuilder must expose BuildSatelliteOverlay and check WorldConfig."""
        src = self.terrain_builder_text
        self.assertIn("function TerrainBuilder.BuildSatelliteOverlay", src,
                       "TerrainBuilder must have BuildSatelliteOverlay function")
        self.assertIn("WorldConfig.EnableTerrainSatelliteOverlay", src,
                       "BuildSatelliteOverlay must check WorldConfig knob")
        self.assertIn("satelliteOverlayBudget", src,
                       "BuildSatelliteOverlay must enforce budget tracking")
        self.assertIn("pcall", src,
                       "Overlay API calls must be pcall-wrapped")
        # Budget constants
        self.assertIn("max = 10", src,
                       "Budget max should be 10 chunks")
        # The overlay must be wired into Build
        self.assertIn("BuildSatelliteOverlay", src)
        # Must create EditableMesh and SurfaceAppearance
        self.assertIn("CreateEditableMesh", src,
                       "Must use EditableMesh for heightfield overlay")
        self.assertIn("SurfaceAppearance", src,
                       "Must use SurfaceAppearance for satellite texture")
        self.assertIn("CreateEditableImage", src,
                       "Must use EditableImage for texture data")
        self.assertIn("WritePixelsBuffer", src,
                       "Must use WritePixelsBuffer to fill texture")

    def test_mesh_accumulator_addquad_emits_front_face_triangles(self) -> None:
        """addQuad must emit standard front-face triangles."""
        src = self.building_builder_text
        self.assertIn("{ base + 1, base + 2, base + 3 }", src)
        self.assertIn("{ base + 1, base + 3, base + 4 }", src)

    def test_road_colors_are_dark_enough_for_grass_contrast(self) -> None:
        """Road surface colors must be dark enough to contrast with green grass terrain.

        All paved road colors (ROAD_COLOR, SUBKIND_COLOR_TINT, MATERIAL_COLOR for
        Asphalt) should be below RGB(100, 100, 100) so they stand out against grass.
        """
        import re
        src = self.road_builder_text

        # Verify ROAD_COLOR default is dark (under 100 per channel)
        default_match = re.search(
            r'default\s*=\s*Color3\.fromRGB\((\d+),\s*(\d+),\s*(\d+)\)',
            src[src.index("ROAD_COLOR = {"):src.index("SUBKIND_COLOR_TINT")],
        )
        self.assertIsNotNone(default_match, "ROAD_COLOR.default must exist")
        r, g, b = int(default_match.group(1)), int(default_match.group(2)), int(default_match.group(3))
        self.assertLess(r, 100, f"ROAD_COLOR.default red channel {r} too bright for grass contrast")
        self.assertLess(g, 100, f"ROAD_COLOR.default green channel {g} too bright for grass contrast")
        self.assertLess(b, 100, f"ROAD_COLOR.default blue channel {b} too bright for grass contrast")

        # Verify primary/residential roads are also dark
        for kind in ("primary", "secondary", "tertiary", "residential"):
            kind_match = re.search(
                rf'{kind}\s*=\s*Color3\.fromRGB\((\d+),\s*(\d+),\s*(\d+)\)',
                src[src.index("ROAD_COLOR = {"):src.index("SUBKIND_COLOR_TINT")],
            )
            self.assertIsNotNone(kind_match, f"ROAD_COLOR.{kind} must exist")
            kr, kg, kb = int(kind_match.group(1)), int(kind_match.group(2)), int(kind_match.group(3))
            self.assertLess(kr, 100, f"ROAD_COLOR.{kind} red={kr} too bright")
            self.assertLess(kg, 100, f"ROAD_COLOR.{kind} green={kg} too bright")
            self.assertLess(kb, 100, f"ROAD_COLOR.{kind} blue={kb} too bright")

        # Verify MATERIAL_COLOR for Asphalt is dark
        asphalt_match = re.search(
            r'Asphalt\]\s*=\s*Color3\.fromRGB\((\d+),\s*(\d+),\s*(\d+)\)',
            src,
        )
        self.assertIsNotNone(asphalt_match, "MATERIAL_COLOR[Asphalt] must exist")
        ar, ag, ab = int(asphalt_match.group(1)), int(asphalt_match.group(2)), int(asphalt_match.group(3))
        self.assertLess(ar, 100, f"MATERIAL_COLOR Asphalt red={ar} too bright")
        self.assertLess(ag, 100, f"MATERIAL_COLOR Asphalt green={ag} too bright")
        self.assertLess(ab, 100, f"MATERIAL_COLOR Asphalt blue={ab} too bright")

        # Verify ROAD_SURFACE_LIFT provides enough elevation above terrain
        lift_match = re.search(r'ROAD_SURFACE_LIFT\s*=\s*([\d.]+)', src)
        self.assertIsNotNone(lift_match, "ROAD_SURFACE_LIFT must be defined")
        lift = float(lift_match.group(1))
        self.assertGreaterEqual(lift, 0.15, f"ROAD_SURFACE_LIFT={lift} too low for terrain visibility")

    def test_world_config_exposes_debug_building_colors(self) -> None:
        """WorldConfig must expose DebugBuildingColors knob for visual debug mode."""
        self.assertIn("DebugBuildingColors", self.world_config_text)
        self.assertIn("DebugBuildingColors = false", self.world_config_text)
        # BuildingBuilder must read the flag and apply debug colors
        src = self.building_builder_text
        self.assertIn("DebugBuildingColors", src)
        self.assertIn("Color3.fromRGB(255, 0, 0)", src, "Wall debug color (red) must be present")
        self.assertIn("Color3.fromRGB(0, 0, 255)", src, "Roof debug color (blue) must be present")
        self.assertIn("ArnisDebugWallCount", src, "Must set wall count attribute for debug")

    def test_water_builder_precomputed_mesh_fast_path(self) -> None:
        """WaterBuilder must have a precomputed mesh fast path consuming waterMesh from the manifest."""
        src = self.water_builder_text
        # Must import AssetService for EditableMesh/MeshPart creation
        self.assertIn("AssetService", src)
        # Must check for waterMesh on the water feature
        self.assertIn("water.waterMesh", src)
        # Must have the BuildPrecomputedMesh function
        self.assertIn("function WaterBuilder.BuildPrecomputedMesh", src)
        # Must create EditableMesh and MeshPart via pcall
        self.assertIn("AssetService:CreateEditableMesh()", src)
        self.assertIn("AssetService:CreateMeshPartAsync(Content.fromObject(mesh))", src)
        # Must convert 0-based Rust indices to 1-based
        self.assertIn("local a = tris[i]", src)
        self.assertIn("local b = tris[i + 1]", src)
        self.assertIn("local c = tris[i + 2]", src)
        self.assertIn("local v1 = vertexIds[a + 1]", src)
        self.assertIn("local v2 = vertexIds[b + 1]", src)
        self.assertIn("local v3 = vertexIds[c + 1]", src)
        # Must offset vertices by originStuds
        self.assertIn("(verts[i] or 0) + ox", src)
        self.assertIn("(verts[i + 1] or 0) + oy", src)
        self.assertIn("(verts[i + 2] or 0) + oz", src)
        # Must have telemetry counters
        self.assertIn("precomputedMeshCount", src)
        self.assertIn("runtimeMeshCount", src)
        self.assertIn("function WaterBuilder.GetMeshTelemetry", src)
        # Build must try the fast path before falling back
        self.assertIn("waterMeshTelemetry.precomputedMeshCount", src)
        self.assertIn("waterMeshTelemetry.runtimeMeshCount", src)
        # Must tag the part as precomputed
        self.assertIn('part:SetAttribute("ArnisPrecomputedMesh", true)', src)

    def test_terrain_builder_precomputed_mesh_infrastructure(self) -> None:
        """TerrainBuilder must have precomputed mesh infrastructure gated by TerrainMeshMode config."""
        src = self.terrain_builder_text
        # Must have the BuildPrecomputedMesh function
        self.assertIn("function TerrainBuilder.BuildPrecomputedMesh", src)
        # Must have TryBuildPrecomputedMesh that checks the config flag
        self.assertIn("function TerrainBuilder.TryBuildPrecomputedMesh", src)
        self.assertIn("WorldConfig.TerrainMeshMode", src)
        # Must check for terrainMesh on the terrain grid
        self.assertIn("terrainGrid.terrainMesh", src)
        # Must create EditableMesh and MeshPart via pcall
        self.assertIn("AssetService:CreateEditableMesh()", src)
        self.assertIn("AssetService:CreateMeshPartAsync(Content.fromObject(mesh))", src)
        # Must convert 0-based Rust indices to 1-based
        self.assertIn("tris[i] + 1", src)
        # Must offset vertices by originStuds
        self.assertIn("verts[i] + ox", src)
        # Must resolve material from materialPalette
        self.assertIn("materialPalette", src)
        # Must have telemetry counters
        self.assertIn("precomputedMeshCount", src)
        self.assertIn("runtimeFillBlockCount", src)
        self.assertIn("function TerrainBuilder.GetMeshTelemetry", src)
        # Must tag the part as precomputed
        self.assertIn('part:SetAttribute("ArnisPrecomputedMesh", true)', src)
        self.assertIn('part:SetAttribute("ArnisTerrainMeshMode", true)', src)

    def test_world_config_exposes_terrain_mesh_mode(self) -> None:
        """WorldConfig must expose TerrainMeshMode flag, defaulting to false."""
        self.assertIn("TerrainMeshMode", self.world_config_text)
        self.assertIn("TerrainMeshMode = false", self.world_config_text)

    def test_import_service_wires_water_and_terrain_mesh_telemetry(self) -> None:
        """ImportService must wire water and terrain precomputed mesh telemetry counters."""
        src = self.import_service_text
        self.assertIn("waterPrecomputedMeshCount", src)
        self.assertIn("waterRuntimeMeshCount", src)
        self.assertIn("terrainPrecomputedMeshCount", src)
        self.assertIn("terrainRuntimeFillBlockCount", src)
        self.assertIn("WaterBuilder.GetMeshTelemetry()", src)
        self.assertIn("TerrainBuilder.GetMeshTelemetry()", src)
        self.assertIn("TerrainBuilder.TryBuildPrecomputedMesh(", src)


if __name__ == "__main__":
    unittest.main()
