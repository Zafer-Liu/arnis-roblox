#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua"
RUN_AUSTIN_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "RunAustin.lua"
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
IMPORT_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "init.lua"
SIGNATURES_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ImportSignatures.lua"
WORLD_PROBE_PATH = ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "WorldProbe.client.lua"
WORLD_PROBE_SUPPORT_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeSupport.lua"
WORLD_PROBE_TERRAIN_PATH = ROOT / "roblox" / "src" / "ReplicatedStorage" / "Shared" / "WorldProbeTerrain.lua"
WORLD_STATE_APPLIER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "WorldStateApplier.lua"
MINIMAP_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "MinimapService.lua"
PREVIEW_BUILDER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
BUILDING_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "BuildingBuilder.lua"
)


class AustinRuntimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bootstrap_text = BOOTSTRAP_PATH.read_text(encoding="utf-8")
        cls.run_austin_text = RUN_AUSTIN_PATH.read_text(encoding="utf-8")
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.import_service_text = IMPORT_SERVICE_PATH.read_text(encoding="utf-8")
        cls.signatures_text = SIGNATURES_PATH.read_text(encoding="utf-8") if SIGNATURES_PATH.exists() else ""
        cls.world_probe_text = WORLD_PROBE_PATH.read_text(encoding="utf-8") if WORLD_PROBE_PATH.exists() else ""
        cls.world_probe_support_text = (
            WORLD_PROBE_SUPPORT_PATH.read_text(encoding="utf-8") if WORLD_PROBE_SUPPORT_PATH.exists() else ""
        )
        cls.world_probe_terrain_text = (
            WORLD_PROBE_TERRAIN_PATH.read_text(encoding="utf-8") if WORLD_PROBE_TERRAIN_PATH.exists() else ""
        )
        cls.world_state_applier_text = WORLD_STATE_APPLIER_PATH.read_text(encoding="utf-8")
        cls.minimap_service_text = MINIMAP_SERVICE_PATH.read_text(encoding="utf-8")
        cls.preview_builder_text = PREVIEW_BUILDER_PATH.read_text(encoding="utf-8")
        cls.building_builder_text = BUILDING_BUILDER_PATH.read_text(encoding="utf-8")

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

    def test_run_austin_publishes_runtime_world_root_telemetry(self) -> None:
        self.assertIn('setPerfAttribute("WorldRootName", "GeneratedWorld_Austin")', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootChildCount", #worldRoot:GetChildren())', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootDescendantCount", #worldRoot:GetDescendants())', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootExists", 1)', self.run_austin_text)
        self.assertIn('setPerfAttribute("WorldRootExists", 0)', self.run_austin_text)

    def test_streaming_service_publishes_startup_residency_telemetry(self) -> None:
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLoadedChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingDesiredChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingCandidateChunkCount", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingProcessedWorkItems", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastFocalX", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLastFocalZ", 0)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingLoadedChunkCount"', self.streaming_text)
        self.assertIn('#ChunkLoader.ListLoadedChunks(streamingOptions.worldRootName)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingCandidateChunkCount", #candidateChunkEntries)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingDesiredChunkCount", desiredChunkCount)', self.streaming_text)
        self.assertIn('Workspace:SetAttribute("ArnisStreamingProcessedWorkItems", processedWorkItems)', self.streaming_text)

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

    def test_client_world_probe_publishes_nearby_building_and_overhead_roof_telemetry(self) -> None:
        self.assertIn('print("ARNIS_CLIENT_WORLD " .. HttpService:JSONEncode(', self.world_probe_text)
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

    def test_client_world_probe_exposes_support_wall_and_roof_cover_observability(self) -> None:
        self.assertIn("supportSurfaceRole =", self.world_probe_text)
        self.assertIn("supportY =", self.world_probe_text)
        self.assertIn("terrainY =", self.world_probe_text)
        self.assertIn("supportMinusTerrainYStuds =", self.world_probe_text)
        self.assertIn("nearbyWallParts =", self.world_probe_text)
        self.assertIn("collidableWallPartsNearby =", self.world_probe_text)
        self.assertIn("nearestWallDistanceStuds =", self.world_probe_text)
        self.assertIn("overheadRoofMinClearanceStuds =", self.world_probe_text)
        self.assertIn("localSupport =", self.world_probe_text)
        self.assertIn("localEnclosure =", self.world_probe_text)
        self.assertIn("localRoofCover =", self.world_probe_text)
        self.assertIn('local shellFolder = model:FindFirstChild("Shell")', self.world_probe_text)
        self.assertIn("local isRoofClosureDeck = descendant:GetAttribute(\"ArnisRoofClosureDeck\") == true", self.world_probe_text)
        self.assertIn("if descendant:IsA(\"MeshPart\") and shellFolder and descendant:IsDescendantOf(shellFolder)", self.world_probe_text)
        self.assertIn(
            "if shellFolder and descendant:IsDescendantOf(shellFolder) and not isRoofPart and not isRoofClosureDeck then",
            self.world_probe_text,
        )
        self.assertIn("WorldProbeSupport.shouldIgnoreGroundHit", self.world_probe_text)
        self.assertIn("WorldProbeSupport.classifySupportSurfaceRole", self.world_probe_text)
        self.assertIn("function WorldProbeSupport.classifySupportSurfaceRole(hitInstance)", self.world_probe_support_text)
        self.assertIn("function WorldProbeSupport.shouldIgnoreGroundHit(hitInstance, worldRoot, ignoredRoots)", self.world_probe_support_text)
        self.assertIn('if hitInstance:IsA("SpawnLocation") then', self.world_probe_support_text)
        self.assertIn('local surfaceRole = node:GetAttribute("ArnisRoadSurfaceRole")', self.world_probe_support_text)

    def test_client_world_probe_exposes_local_terrain_roughness_metrics(self) -> None:
        self.assertIn("local WorldProbeTerrain = require(ReplicatedStorage.Shared.WorldProbeTerrain)", self.world_probe_text)
        self.assertIn("localTerrain = localTerrain", self.world_probe_text)
        self.assertIn("local function sampleLocalTerrain(rootPart, worldRoot)", self.world_probe_text)
        self.assertIn("local samples = table.create(#LOCAL_TERRAIN_OFFSETS)", self.world_probe_text)
        self.assertIn("samples[index] = {", self.world_probe_text)
        self.assertIn("WorldProbeTerrain.summarizeTerrainSamples", self.world_probe_text)
        self.assertIn("samplePattern = samplePattern", self.world_probe_terrain_text)
        self.assertIn("sampleRadiusStuds = sampleRadiusStuds", self.world_probe_terrain_text)
        self.assertIn("sampleCount = sampleCount", self.world_probe_terrain_text)
        self.assertIn("missingSampleCount = missingSampleCount", self.world_probe_terrain_text)
        self.assertIn("centerTerrainY = roundTenths(centerTerrainY)", self.world_probe_terrain_text)
        self.assertIn("heightRangeStuds =", self.world_probe_terrain_text)
        self.assertIn("maxStepStuds =", self.world_probe_terrain_text)
        self.assertIn("meanAbsStepStuds =", self.world_probe_terrain_text)

    def test_client_world_probe_emits_dedicated_local_experience_marker(self) -> None:
        self.assertIn('print("ARNIS_CLIENT_LOCAL_EXPERIENCE " .. localExperiencePayloadJson)', self.world_probe_text)
        self.assertIn("localExperiencePayload.localSupport = payload.localSupport", self.world_probe_text)
        self.assertIn("localExperiencePayload.localTerrain = payload.localTerrain", self.world_probe_text)
        self.assertIn("localExperiencePayload.localEnclosure = payload.localEnclosure", self.world_probe_text)
        self.assertIn("localExperiencePayload.localRoofCover = payload.localRoofCover", self.world_probe_text)
        self.assertIn("bootstrapAttemptId = bootstrapPayload.bootstrapAttemptId,", self.world_probe_text)

    def test_shaped_roof_closure_decks_are_marked_internal_support_not_visible_roof_truth(self) -> None:
        self.assertIn("local function applyRoofPartOptions(part, partOptions)", self.building_builder_text)
        self.assertIn("ArnisRoofClosureDeck = true", self.building_builder_text)
        self.assertIn("part.Transparency = partOptions.transparency", self.building_builder_text)


if __name__ == "__main__":
    unittest.main()
