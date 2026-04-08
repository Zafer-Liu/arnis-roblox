--[[
  TelemetryReporter.lua
  Captures bootstrap phase timings and chunk-fetch telemetry, then POSTs a
  consolidated payload to the Cloudflare planetary worker so the run can be
  inspected server-side without being in-game.

  Contract: the payload schema produced by Report() is consumed by the
  Cloudflare worker (`/telemetry/run`) and by the Python audit script. Do not
  reorder or rename fields without coordinating both consumers.

  Operational guarantees:
  * Report() never throws to the caller — pcall-wrapped, fire-and-forget.
  * No new require cycles: this module does NOT require BootstrapAustin.
  * Works even if HttpService is disabled — the RequestAsync call is pcall'd
    and only emits a warning on failure.
--]]

local HttpService = game:GetService("HttpService")
local Workspace = game:GetService("Workspace")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local WorldConfig = require(ReplicatedStorage.Shared.WorldConfig)

local TelemetryReporter = {}

local DEFAULT_ENDPOINT = "https://planetary.adpena.workers.dev"
local USER_AGENT = "arnis-planetary-runtime/0.5"

local configuredEndpoint = DEFAULT_ENDPOINT
local phaseTimestamps = {}
local phaseOrder = {}

function TelemetryReporter.Configure(endpointUrl)
    if type(endpointUrl) == "string" and endpointUrl ~= "" then
        configuredEndpoint = endpointUrl
    end
end

function TelemetryReporter.RecordPhase(phaseName)
    if type(phaseName) ~= "string" or phaseName == "" then
        return
    end
    if phaseTimestamps[phaseName] == nil then
        table.insert(phaseOrder, phaseName)
    end
    phaseTimestamps[phaseName] = os.clock()
end

local function resolveSourceUrl()
    local manifestSourceConfig = WorldConfig.ManifestSource or {}
    if manifestSourceConfig.mode == "external_url" then
        return tostring(manifestSourceConfig.externalUrl or "")
    elseif manifestSourceConfig.mode == "roblox_asset" then
        return ("roblox_asset://%s"):format(tostring(manifestSourceConfig.robloxAssetId or 0))
    end
    return tostring(manifestSourceConfig.mode or "embedded")
end

local function buildPhaseDeltas()
    local deltas = {}
    if #phaseOrder == 0 then
        return deltas, 0
    end
    local firstPhase = phaseOrder[1]
    local firstTs = phaseTimestamps[firstPhase] or 0
    local lastTs = firstTs
    for _, name in ipairs(phaseOrder) do
        local ts = phaseTimestamps[name] or firstTs
        deltas[name] = ts - firstTs
        if ts > lastTs then
            lastTs = ts
        end
    end
    return deltas, lastTs - firstTs
end

local function buildPayload(payloadOverrides)
    payloadOverrides = payloadOverrides or {}

    local phases, totalElapsed = buildPhaseDeltas()

    local runId
    local okGuid, guidOrErr = pcall(function()
        return HttpService:GenerateGUID(false)
    end)
    if okGuid then
        runId = guidOrErr
    else
        runId = string.format("local-%d-%04x", math.floor(os.clock() * 1000), math.random(0, 0xFFFF))
    end

    local payload = {
        runId = runId,
        timestamp = os.time(),
        place = {
            placeId = game.PlaceId,
            universeId = game.GameId,
            serverJobId = game.JobId,
        },
        bootstrap = {
            status = payloadOverrides.status or "success",
            totalElapsedSeconds = totalElapsed,
            errorMessage = payloadOverrides.errorMessage,
            errorDetail = payloadOverrides.errorDetail,
            phases = phases,
        },
        chunkFetch = {
            sourceUrl = Workspace:GetAttribute("ArnisChunkFetchSourceUrl") or "",
            fetchCount = Workspace:GetAttribute("ArnisChunkFetchCount") or 0,
            failureCount = Workspace:GetAttribute("ArnisChunkFetchFailures") or 0,
            totalBytes = Workspace:GetAttribute("ArnisChunkFetchBytes") or 0,
            avgLatencyMs = Workspace:GetAttribute("ArnisChunkFetchAvgLatencyMs") or 0,
            slowestLatencyMs = Workspace:GetAttribute("ArnisChunkFetchSlowestLatencyMs") or 0,
            slowestChunkId = Workspace:GetAttribute("ArnisChunkFetchSlowestChunkId") or "",
        },
        import = {
            chunksImported = payloadOverrides.chunksImported or 0,
            totalInstances = payloadOverrides.totalInstances or 0,
            totalFeatures = payloadOverrides.totalFeatures or 0,
        },
        environment = {
            placeVersion = game.PlaceVersion,
            sourceUrl = resolveSourceUrl(),
        },
    }

    return payload
end

local function postPayload(payload)
    local okEncode, body = pcall(function()
        return HttpService:JSONEncode(payload)
    end)
    if not okEncode then
        warn(("[TelemetryReporter] JSONEncode failed: %s"):format(tostring(body)))
        return
    end

    local url = ("%s/telemetry/run"):format(configuredEndpoint)
    local request = {
        Url = url,
        Method = "POST",
        -- Restricted headers (User-Agent, Accept-Encoding, Host, Origin, etc.)
        -- are blocked by HttpService:RequestAsync with the error "Header X is
        -- not allowed!". Only set Content-Type here; the engine supplies its
        -- own User-Agent. The USER_AGENT constant is still exposed for tests
        -- and documentation but is NOT sent on the wire.
        Headers = {
            ["Content-Type"] = "application/json",
        },
        Body = body,
    }

    local okRequest, response = pcall(function()
        return HttpService:RequestAsync(request)
    end)
    if not okRequest then
        warn(("[TelemetryReporter] RequestAsync failed: %s"):format(tostring(response)))
        return
    end
    if type(response) == "table" and response.Success ~= true then
        warn(
            ("[TelemetryReporter] POST %s returned status=%s %s"):format(
                url,
                tostring(response.StatusCode),
                tostring(response.StatusMessage)
            )
        )
    end
end

function TelemetryReporter.Report(payloadOverrides)
    -- Fire-and-forget: never block or throw to the bootstrap caller. We
    -- pcall everything inside the spawned task as well so a failure to even
    -- assemble the payload (e.g. attribute read race) cannot crash the
    -- background task chain.
    task.spawn(function()
        local okBuild, payloadOrErr = pcall(buildPayload, payloadOverrides)
        if not okBuild then
            warn(("[TelemetryReporter] buildPayload failed: %s"):format(tostring(payloadOrErr)))
            return
        end
        local okPost, postErr = pcall(postPayload, payloadOrErr)
        if not okPost then
            warn(("[TelemetryReporter] postPayload failed: %s"):format(tostring(postErr)))
        end
    end)
end

return TelemetryReporter
