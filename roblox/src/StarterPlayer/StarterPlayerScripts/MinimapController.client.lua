local AssetService = game:GetService("AssetService")
local HttpService = game:GetService("HttpService")
local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local RunService = game:GetService("RunService")
local TweenService = game:GetService("TweenService")
local UserInputService = game:GetService("UserInputService")
local Workspace = game:GetService("Workspace")

local EditableImageCompat = require(ReplicatedStorage.Shared.EditableImageCompat)

local player = Players.LocalPlayer
local playerGui = player:WaitForChild("PlayerGui")

local MAP_SIZE = 200
local MAP_DISPLAY_SIZE = 180
local MAP_FULLSCREEN_SIZE = 600
local MAP_RADIUS = 400
local MAP_RADIUS_FULL = 1600
local UPDATE_INTERVAL = 0.2
local MIN_RENDER_MOVE_STUDS = 4
local HEADING_BUCKET_DEGREES = 6
local BORDER_WIDTH = 2
local WORLD_ROOT_ATTR = "ArnisMinimapWorldRootName"
local ENABLED_ATTR = "ArnisMinimapEnabled"
local CHUNK_JSON_ATTR = "ArnisMinimapChunkJson"
local TWEEN_INFO = TweenInfo.new(0.35, Enum.EasingStyle.Quint, Enum.EasingDirection.Out)

local COLORS = table.freeze({
    background = { 30, 35, 45, 255 },
    road = { 255, 255, 255, 255 },
    road_minor = { 220, 220, 220, 255 },
    building = { 210, 200, 185, 255 },
    water = { 170, 210, 240, 255 },
    park = { 180, 220, 170, 255 },
    forest = { 140, 190, 140, 255 },
    parking = { 230, 225, 215, 255 },
    player = { 65, 130, 240, 255 },
    player_dir = { 65, 130, 240, 200 },
    border = { 50, 55, 65, 255 },
})

local MAJOR_ROAD_KINDS = table.freeze({
    primary = true,
    secondary = true,
    tertiary = true,
    trunk = true,
    motorway = true,
})

local COMPASS_DIRS = table.freeze({ "N", "NE", "E", "SE", "S", "SW", "W", "NW" })

-- Telemetry attribute set; module-level so publishMinimapTelemetry is not
-- rebuilding this table every call.
local TELEMETRY_ATTR_NAMES = table.freeze({
    "ArnisMinimapEnabled",
    "ArnisMinimapGuiReady",
    "ArnisMinimapWorldRootName",
    "ArnisMinimapSnapshotCount",
    "ArnisMinimapFullscreen",
    "ArnisMinimapError",
})

-- Scratch buffers reused across renders so we don't churn the allocator at
-- the minimap's render cadence. Never hand these out across yields.
local scanlineIntersections = table.create(16)
local snapshotIterBuffer = {}

local screenGui = nil
local imageLabel = nil
local mapLabel = nil
local editableImage = nil
local pixelBuffer = nil
local lastUpdate = 0
local isFullscreen = false
local currentWorldRoot = nil
local chunkConnections = {}
local chunkSnapshotsByFolder = {}
local worldRootConnections = {}
local lastTelemetry = {}
local chunkSnapshotRevision = 0
local lastRenderedSnapshotRevision = -1
local lastRenderedFullscreen = nil
local lastRenderedCamX = nil
local lastRenderedCamZ = nil
local lastRenderedHeadingBucket = nil

local function markSnapshotsDirty()
    chunkSnapshotRevision += 1
end

local function setPlayerAttributeIfChanged(name, nextValue)
    if player:GetAttribute(name) == nextValue then
        return
    end
    player:SetAttribute(name, nextValue)
end

local function countChunkSnapshots()
    local count = 0
    for _ in pairs(chunkSnapshotsByFolder) do
        count += 1
    end
    return count
end

local function buildTelemetryPayload(extra)
    local payload = {}
    payload.enabled = Workspace:GetAttribute(ENABLED_ATTR) == true
    payload.guiReady = screenGui ~= nil and screenGui.Parent == playerGui and screenGui.Enabled == true
    payload.worldRootName = Workspace:GetAttribute(WORLD_ROOT_ATTR)
    payload.snapshotCount = countChunkSnapshots()
    payload.fullscreen = isFullscreen
    payload.error = extra and extra.error or nil
    return payload
end

-- Returns (changed, structuralChange) describing whether any tracked player
-- attribute drifted and whether that drift was more than just the snapshot
-- count bumping (which fires on every chunk register/clear).
local function diffTelemetryAttributes(payload)
    local pending = {
        ArnisMinimapEnabled = payload.enabled,
        ArnisMinimapGuiReady = payload.guiReady,
        ArnisMinimapWorldRootName = payload.worldRootName,
        ArnisMinimapSnapshotCount = payload.snapshotCount,
        ArnisMinimapFullscreen = payload.fullscreen,
        ArnisMinimapError = payload.error,
    }
    local changed = false
    local structuralChange = false
    for _, attributeName in ipairs(TELEMETRY_ATTR_NAMES) do
        local nextValue = pending[attributeName]
        if lastTelemetry[attributeName] ~= nextValue then
            setPlayerAttributeIfChanged(attributeName, nextValue)
            lastTelemetry[attributeName] = nextValue
            changed = true
            if attributeName ~= "ArnisMinimapSnapshotCount" then
                structuralChange = true
            end
        end
    end
    return changed, structuralChange
end

local function shouldLogTelemetry(changed, structuralChange, snapshotCount)
    if not changed then
        return false
    end
    return structuralChange or snapshotCount <= 1 or snapshotCount % 10 == 0
end

local function publishMinimapTelemetry(extra)
    local payload = buildTelemetryPayload(extra)
    local changed, structuralChange = diffTelemetryAttributes(payload)
    if shouldLogTelemetry(changed, structuralChange, payload.snapshotCount) then
        print("ARNIS_CLIENT_MINIMAP " .. HttpService:JSONEncode(payload))
    end
end

local function disconnectConnections(connections)
    for _, connection in ipairs(connections) do
        connection:Disconnect()
    end
    table.clear(connections)
end

local function initBuffer()
    pixelBuffer = buffer.create(MAP_SIZE * MAP_SIZE * 4)
end

local function clearBuffer()
    local bg = COLORS.background
    for i = 0, MAP_SIZE * MAP_SIZE - 1 do
        local offset = i * 4
        buffer.writeu8(pixelBuffer, offset, bg[1])
        buffer.writeu8(pixelBuffer, offset + 1, bg[2])
        buffer.writeu8(pixelBuffer, offset + 2, bg[3])
        buffer.writeu8(pixelBuffer, offset + 3, bg[4])
    end
end

local function setPixel(x, y, color)
    if x < 0 or x >= MAP_SIZE or y < 0 or y >= MAP_SIZE then
        return
    end
    local offset = (y * MAP_SIZE + x) * 4
    buffer.writeu8(pixelBuffer, offset, color[1])
    buffer.writeu8(pixelBuffer, offset + 1, color[2])
    buffer.writeu8(pixelBuffer, offset + 2, color[3])
    buffer.writeu8(pixelBuffer, offset + 3, color[4])
end

local function drawLine(x1, y1, x2, y2, color, thickness)
    thickness = thickness or 1
    local dx = x2 - x1
    local dy = y2 - y1
    local steps = math.max(math.abs(dx), math.abs(dy))
    if steps == 0 then
        setPixel(x1, y1, color)
        return
    end
    local xInc = dx / steps
    local yInc = dy / steps
    local half = math.floor(thickness / 2)
    for i = 0, steps do
        local px = math.floor(x1 + xInc * i)
        local py = math.floor(y1 + yInc * i)
        for t = -half, half do
            setPixel(px + t, py, color)
            setPixel(px, py + t, color)
        end
    end
end

local function drawRect(x1, y1, x2, y2, color)
    for y = math.max(0, math.floor(y1)), math.min(MAP_SIZE - 1, math.floor(y2)) do
        for x = math.max(0, math.floor(x1)), math.min(MAP_SIZE - 1, math.floor(x2)) do
            setPixel(x, y, color)
        end
    end
end

local function drawCircle(cx, cy, radius, color)
    local r2 = radius * radius
    for dy = -radius, radius do
        for dx = -radius, radius do
            if dx * dx + dy * dy <= r2 then
                setPixel(math.floor(cx + dx), math.floor(cy + dy), color)
            end
        end
    end
end

local function worldToPixel(worldX, worldZ, camX, camZ)
    local rx = worldX - camX
    local rz = worldZ - camZ
    local activeRadius = isFullscreen and MAP_RADIUS_FULL or MAP_RADIUS
    local scale = MAP_SIZE / (activeRadius * 2)
    local px = MAP_SIZE / 2 + rx * scale
    local py = MAP_SIZE / 2 - rz * scale
    return math.floor(px), math.floor(py)
end

local function footprintToPixelPoints(footprint, ox, oz, camX, camZ)
    local pixelPoints = table.create(#(footprint or {}))
    for index, point in ipairs(footprint or {}) do
        local px, py = worldToPixel(point.x + ox, point.z + oz, camX, camZ)
        pixelPoints[index] = {
            x = px,
            y = py,
        }
    end
    return pixelPoints
end

local function polygonYBounds(pixelPoints)
    local minY = math.huge
    local maxY = -math.huge
    for _, point in ipairs(pixelPoints) do
        if point.y < minY then
            minY = point.y
        end
        if point.y > maxY then
            maxY = point.y
        end
    end
    return math.max(0, math.floor(minY)), math.min(MAP_SIZE - 1, math.ceil(maxY))
end

local function gatherScanlineIntersections(pixelPoints, y, intersections)
    local count = 0
    local pointCount = #pixelPoints
    for index = 1, pointCount do
        local p1 = pixelPoints[index]
        local p2 = pixelPoints[index % pointCount + 1]
        local y1 = p1.y
        local y2 = p2.y
        local crosses = (y1 <= y and y2 > y) or (y2 <= y and y1 > y)
        if crosses then
            local t = (y - y1) / (y2 - y1)
            count += 1
            intersections[count] = p1.x + (p2.x - p1.x) * t
        end
    end
    return count
end

local function drawScanlineSpans(intersections, count, y, color)
    for index = 1, count - 1, 2 do
        drawRect(intersections[index], y, intersections[index + 1], y, color)
    end
end

local function drawFilledPolygon(pixelPoints, color)
    if #pixelPoints < 3 then
        return
    end

    local minY, maxY = polygonYBounds(pixelPoints)
    local intersections = scanlineIntersections

    for y = minY, maxY do
        local count = gatherScanlineIntersections(pixelPoints, y, intersections)
        if count >= 2 then
            -- Partial sort over the filled prefix only; table.sort operates
            -- on [1..#t], so clear the tail we don't use this pass.
            for i = count + 1, #intersections do
                intersections[i] = nil
            end
            table.sort(intersections)
            drawScanlineSpans(intersections, count, y, color)
        end
    end
end

local function drawPlayerHeading(camYaw)
    drawCircle(MAP_SIZE / 2, MAP_SIZE / 2, 4, COLORS.player)

    local dirLen = 10
    local dirX = math.sin(camYaw)
    local dirY = -math.cos(camYaw)
    local centerX = MAP_SIZE / 2
    local centerY = MAP_SIZE / 2
    local tipX = centerX + dirX * dirLen
    local tipY = centerY + dirY * dirLen
    drawLine(centerX, centerY, tipX, tipY, COLORS.player_dir, 2)

    local leftX = tipX - dirY * 2
    local leftY = tipY + dirX * 2
    local rightX = tipX + dirY * 2
    local rightY = tipY - dirX * 2
    drawLine(tipX, tipY, leftX, leftY, COLORS.player_dir, 1)
    drawLine(tipX, tipY, rightX, rightY, COLORS.player_dir, 1)
end

local function refreshChunkSnapshotList()
    local list = snapshotIterBuffer
    local count = 0
    for _, snapshot in pairs(chunkSnapshotsByFolder) do
        count += 1
        list[count] = snapshot
    end
    for i = count + 1, #list do
        list[i] = nil
    end
    return list, count
end

local LANDUSE_COLOR_BY_KIND = table.freeze({
    forest = COLORS.forest,
    wood = COLORS.forest,
    parking = COLORS.parking,
})

local function landuseColor(kind)
    return LANDUSE_COLOR_BY_KIND[kind] or COLORS.park
end

local function drawChunkLanduse(chunk, ox, oz, camX, camZ)
    for _, lu in ipairs(chunk.landuse or {}) do
        local fp = lu.footprint
        if fp and #fp >= 3 then
            drawFilledPolygon(
                footprintToPixelPoints(fp, ox, oz, camX, camZ),
                landuseColor(lu.kind)
            )
        end
    end
end

local function drawWaterPolyline(water, ox, oz, camX, camZ, activeRadius)
    local points = water.points
    if not points then
        return
    end
    local widthPx = math.max(2, math.floor((water.widthStuds or 8) * MAP_SIZE / (activeRadius * 2)))
    for i = 1, #points - 1 do
        local p1 = points[i]
        local p2 = points[i + 1]
        local px1, py1 = worldToPixel(p1.x + ox, p1.z + oz, camX, camZ)
        local px2, py2 = worldToPixel(p2.x + ox, p2.z + oz, camX, camZ)
        drawLine(px1, py1, px2, py2, COLORS.water, widthPx)
    end
end

local function drawChunkWater(chunk, ox, oz, camX, camZ, activeRadius)
    for _, water in ipairs(chunk.water or {}) do
        local fp = water.footprint
        if fp and #fp >= 3 then
            drawFilledPolygon(footprintToPixelPoints(fp, ox, oz, camX, camZ), COLORS.water)
        else
            drawWaterPolyline(water, ox, oz, camX, camZ, activeRadius)
        end
    end
end

local function drawChunkBuildings(chunk, ox, oz, camX, camZ)
    for _, building in ipairs(chunk.buildings or {}) do
        local fp = building.footprint
        if fp and #fp >= 3 then
            drawFilledPolygon(footprintToPixelPoints(fp, ox, oz, camX, camZ), COLORS.building)
        end
    end
end

local function roadColor(kind)
    if MAJOR_ROAD_KINDS[kind] then
        return COLORS.road
    end
    return COLORS.road_minor
end

local function roadWidthPixels(road, activeRadius)
    local widthPx = math.max(1, math.floor((road.widthStuds or 10) * MAP_SIZE / (activeRadius * 2) * 0.5))
    return math.min(widthPx, 4)
end

local function drawRoadSegments(road, ox, oz, camX, camZ, activeRadius)
    local points = road.points
    if not points then
        return
    end
    local color = roadColor(road.kind)
    local widthPx = roadWidthPixels(road, activeRadius)
    for i = 1, #points - 1 do
        local p1 = points[i]
        local p2 = points[i + 1]
        local px1, py1 = worldToPixel(p1.x + ox, p1.z + oz, camX, camZ)
        local px2, py2 = worldToPixel(p2.x + ox, p2.z + oz, camX, camZ)
        drawLine(px1, py1, px2, py2, color, widthPx)
    end
end

local function drawChunkRoads(chunk, ox, oz, camX, camZ, activeRadius)
    for _, road in ipairs(chunk.roads or {}) do
        drawRoadSegments(road, ox, oz, camX, camZ, activeRadius)
    end
end

-- Painter's order preserved: landuse -> water -> buildings -> roads, matching
-- the pre-split rendering loop exactly.
local function drawChunk(chunk, camX, camZ, activeRadius)
    local origin = chunk.originStuds
    local ox = (origin and origin.x) or 0
    local oz = (origin and origin.z) or 0
    drawChunkLanduse(chunk, ox, oz, camX, camZ)
    drawChunkWater(chunk, ox, oz, camX, camZ, activeRadius)
    drawChunkBuildings(chunk, ox, oz, camX, camZ)
    drawChunkRoads(chunk, ox, oz, camX, camZ, activeRadius)
end

local function drawBorder()
    local lastIndex = MAP_SIZE - 1
    for i = 0, lastIndex do
        for t = 0, BORDER_WIDTH - 1 do
            setPixel(i, t, COLORS.border)
            setPixel(i, lastIndex - t, COLORS.border)
            setPixel(t, i, COLORS.border)
            setPixel(lastIndex - t, i, COLORS.border)
        end
    end
end

local function renderMap(camX, camZ)
    clearBuffer()
    local activeRadius = isFullscreen and MAP_RADIUS_FULL or MAP_RADIUS
    local snapshots, count = refreshChunkSnapshotList()
    for i = 1, count do
        drawChunk(snapshots[i], camX, camZ, activeRadius)
    end
    drawBorder()
end

local function ensureGui()
    if screenGui then
        return
    end

    screenGui = Instance.new("ScreenGui")
    screenGui.Name = "MinimapGui"
    screenGui.ResetOnSpawn = false
    screenGui.ZIndexBehavior = Enum.ZIndexBehavior.Sibling
    screenGui.IgnoreGuiInset = true

    local frame = Instance.new("Frame")
    frame.Name = "MinimapFrame"
    frame.Size = UDim2.new(0, MAP_DISPLAY_SIZE + 10, 0, MAP_DISPLAY_SIZE + 10)
    frame.Position = UDim2.new(0, 10, 1, -MAP_DISPLAY_SIZE - 20)
    frame.BackgroundColor3 = Color3.fromRGB(20, 22, 30)
    frame.BorderSizePixel = 0
    frame.Parent = screenGui

    local corner = Instance.new("UICorner")
    corner.CornerRadius = UDim.new(0, 8)
    corner.Parent = frame

    local stroke = Instance.new("UIStroke")
    stroke.Color = Color3.fromRGB(60, 65, 80)
    stroke.Thickness = 2
    stroke.Parent = frame

    imageLabel = Instance.new("ImageLabel")
    imageLabel.Name = "MapImage"
    imageLabel.Size = UDim2.new(0, MAP_DISPLAY_SIZE, 0, MAP_DISPLAY_SIZE)
    imageLabel.Position = UDim2.new(0, 5, 0, 5)
    imageLabel.BackgroundTransparency = 1
    imageLabel.ScaleType = Enum.ScaleType.Stretch
    imageLabel.Parent = frame

    local ok, imageOrError = pcall(function()
        return AssetService:CreateEditableImage({ Size = Vector2.new(MAP_SIZE, MAP_SIZE) })
    end)
    if not ok or not imageOrError then
        publishMinimapTelemetry({
            error = ok and "editable_image_unavailable" or tostring(imageOrError),
        })
        return
    end
    editableImage = imageOrError
    imageLabel.ImageContent = Content.fromObject(editableImage)

    local label = Instance.new("TextLabel")
    label.Name = "Title"
    label.Size = UDim2.new(1, 0, 0, 16)
    label.Position = UDim2.new(0, 0, 1, -16)
    label.BackgroundTransparency = 1
    label.Text = "MAP"
    label.TextColor3 = Color3.fromRGB(140, 145, 160)
    label.TextSize = 11
    label.Font = Enum.Font.GothamBold
    label.Parent = frame
    mapLabel = label

    screenGui.Parent = playerGui
    publishMinimapTelemetry()
end

local function setGuiEnabled(enabled)
    if enabled then
        ensureGui()
    end

    if screenGui then
        screenGui.Enabled = enabled
    end
    publishMinimapTelemetry()
end

local function toggleFullscreen()
    if not screenGui then
        return
    end

    local frame = screenGui:FindFirstChild("MinimapFrame")
    if not frame or not imageLabel or not mapLabel then
        return
    end

    isFullscreen = not isFullscreen
    if isFullscreen then
        local size = MAP_FULLSCREEN_SIZE + 10
        mapLabel.Text = "MAP  [M to close]"
        TweenService:Create(frame, TWEEN_INFO, {
            Size = UDim2.new(0, size, 0, size),
            Position = UDim2.new(0.5, -size / 2, 0.5, -size / 2),
        }):Play()
        TweenService:Create(imageLabel, TWEEN_INFO, {
            Size = UDim2.new(0, MAP_FULLSCREEN_SIZE, 0, MAP_FULLSCREEN_SIZE),
        }):Play()
    else
        local size = MAP_DISPLAY_SIZE + 10
        mapLabel.Text = "MAP"
        TweenService:Create(frame, TWEEN_INFO, {
            Size = UDim2.new(0, size, 0, size),
            Position = UDim2.new(0, 10, 1, -size - 10),
        }):Play()
        TweenService:Create(imageLabel, TWEEN_INFO, {
            Size = UDim2.new(0, MAP_DISPLAY_SIZE, 0, MAP_DISPLAY_SIZE),
        }):Play()
    end
    publishMinimapTelemetry()
end

local function decodeChunkSnapshot(folder)
    local payload = folder:GetAttribute(CHUNK_JSON_ATTR)
    if type(payload) ~= "string" or payload == "" then
        chunkSnapshotsByFolder[folder] = nil
        return
    end

    local ok, decoded = pcall(HttpService.JSONDecode, HttpService, payload)
    if ok and type(decoded) == "table" then
        chunkSnapshotsByFolder[folder] = decoded
    else
        chunkSnapshotsByFolder[folder] = nil
    end
    markSnapshotsDirty()
    publishMinimapTelemetry()
end

local function attachChunkFolder(folder)
    if not folder or not folder:IsA("Folder") or chunkConnections[folder] then
        return
    end

    decodeChunkSnapshot(folder)
    chunkConnections[folder] = {
        folder:GetAttributeChangedSignal(CHUNK_JSON_ATTR):Connect(function()
            decodeChunkSnapshot(folder)
        end),
        folder.Destroying:Connect(function()
            chunkSnapshotsByFolder[folder] = nil
            disconnectConnections(chunkConnections[folder] or {})
            chunkConnections[folder] = nil
            markSnapshotsDirty()
        end),
    }
end

local function detachAllChunks()
    for folder, connections in pairs(chunkConnections) do
        disconnectConnections(connections)
        chunkConnections[folder] = nil
        chunkSnapshotsByFolder[folder] = nil
    end
    markSnapshotsDirty()
    publishMinimapTelemetry()
end

local function bindWorldRoot(worldRoot)
    if currentWorldRoot == worldRoot then
        return
    end

    disconnectConnections(worldRootConnections)
    detachAllChunks()
    currentWorldRoot = worldRoot

    if not currentWorldRoot then
        publishMinimapTelemetry()
        return
    end

    for _, child in ipairs(currentWorldRoot:GetChildren()) do
        attachChunkFolder(child)
    end

    worldRootConnections[#worldRootConnections + 1] = currentWorldRoot.ChildAdded:Connect(attachChunkFolder)
    worldRootConnections[#worldRootConnections + 1] = currentWorldRoot.ChildRemoved:Connect(function(child)
        local connections = chunkConnections[child]
        if connections then
            disconnectConnections(connections)
            chunkConnections[child] = nil
        end
        chunkSnapshotsByFolder[child] = nil
        markSnapshotsDirty()
        publishMinimapTelemetry()
    end)
    publishMinimapTelemetry()
end

local function resolveWorldRoot()
    local worldRootName = Workspace:GetAttribute(WORLD_ROOT_ATTR)
    if type(worldRootName) ~= "string" or worldRootName == "" then
        return nil
    end
    return Workspace:FindFirstChild(worldRootName)
end

UserInputService.InputBegan:Connect(function(input, gameProcessed)
    if gameProcessed then
        return
    end
    if input.KeyCode == Enum.KeyCode.M then
        toggleFullscreen()
    end
end)

Workspace:GetAttributeChangedSignal(WORLD_ROOT_ATTR):Connect(function()
    bindWorldRoot(resolveWorldRoot())
end)

Workspace:GetAttributeChangedSignal(ENABLED_ATTR):Connect(function()
    setGuiEnabled(Workspace:GetAttribute(ENABLED_ATTR) == true)
end)

initBuffer()
setGuiEnabled(Workspace:GetAttribute(ENABLED_ATTR) == true)
bindWorldRoot(resolveWorldRoot())
publishMinimapTelemetry()

RunService.Heartbeat:Connect(function(dt)
    lastUpdate += dt
    if lastUpdate < UPDATE_INTERVAL then
        return
    end
    lastUpdate = 0

    if Workspace:GetAttribute(ENABLED_ATTR) ~= true or not editableImage or not screenGui or not screenGui.Enabled then
        return
    end

    local camera = Workspace.CurrentCamera
    if not camera then
        return
    end

    local camPos = camera.CFrame.Position
    local camLook = camera.CFrame.LookVector
    local camYaw = math.atan2(camLook.X, camLook.Z)
    local heading = math.floor((camYaw * 180 / math.pi) % 360)
    local headingBucket = math.floor(heading / HEADING_BUCKET_DEGREES)
    local movedEnough
    if lastRenderedCamX == nil or lastRenderedCamZ == nil then
        movedEnough = true
    else
        local dxMoved = camPos.X - lastRenderedCamX
        local dzMoved = camPos.Z - lastRenderedCamZ
        movedEnough = dxMoved * dxMoved + dzMoved * dzMoved >= MIN_RENDER_MOVE_STUDS * MIN_RENDER_MOVE_STUDS
    end
    local needsRender = movedEnough
        or lastRenderedSnapshotRevision ~= chunkSnapshotRevision
        or lastRenderedFullscreen ~= isFullscreen
        or lastRenderedHeadingBucket ~= headingBucket

    local function updateHeadingLabel()
        if isFullscreen or not mapLabel then
            return
        end
        local dirIdx = math.floor((heading + 22.5) / 45) % 8 + 1
        mapLabel.Text = string.format("MAP  %s %d°", COMPASS_DIRS[dirIdx], heading)
    end

    if not needsRender then
        updateHeadingLabel()
        return
    end

    renderMap(camPos.X, camPos.Z)
    drawPlayerHeading(camYaw)
    EditableImageCompat.WritePixels(editableImage, Vector2.zero, Vector2.new(MAP_SIZE, MAP_SIZE), pixelBuffer)
    lastRenderedCamX = camPos.X
    lastRenderedCamZ = camPos.Z
    lastRenderedSnapshotRevision = chunkSnapshotRevision
    lastRenderedFullscreen = isFullscreen
    lastRenderedHeadingBucket = headingBucket

    updateHeadingLabel()
end)
