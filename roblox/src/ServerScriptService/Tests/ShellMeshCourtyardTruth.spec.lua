return function()
    local Workspace = game:GetService("Workspace")
    local ImportService = require(script.Parent.Parent.ImportService)
    local Assert = require(script.Parent.Assert)

    local manifest = {
        schemaVersion = "0.4.0",
        meta = {
            worldName = "ShellMeshCourtyardTruth",
            generator = "test",
            source = "unit",
            metersPerStud = 0.3,
            chunkSizeStuds = 256,
            totalFeatures = 1,
        },
        chunks = {
            {
                id = "0_0",
                originStuds = { x = 0, y = 0, z = 0 },
                terrain = {
                    cellSizeStuds = 16,
                    width = 16,
                    depth = 16,
                    heights = table.create(16 * 16, 0),
                    material = "Grass",
                },
                roads = {},
                rails = {},
                buildings = {
                    {
                        id = "shellmesh_courtyard",
                        footprint = {
                            { x = 0, z = 0 },
                            { x = 48, z = 0 },
                            { x = 48, z = 48 },
                            { x = 0, z = 48 },
                            { x = 0, z = 0 },
                        },
                        holes = {
                            {
                                { x = 16, z = 16 },
                                { x = 32, z = 16 },
                                { x = 32, z = 32 },
                                { x = 16, z = 32 },
                                { x = 16, z = 16 },
                            },
                        },
                        baseY = 0,
                        height = 16,
                        levels = 3,
                        roof = "flat",
                        usage = "residential",
                        material = "Concrete",
                    },
                },
                water = {},
                props = {},
                landuse = {},
                barriers = {},
            },
        },
    }

    local worldRootName = "GeneratedWorld_ShellMeshCourtyardTruth"
    ImportService.ImportManifest(manifest, {
        clearFirst = true,
        worldRootName = worldRootName,
        config = {
            BuildingMode = "shellMesh",
            TerrainMode = "none",
            RoadMode = "none",
            WaterMode = "none",
            LanduseMode = "none",
        },
    })

    local worldRoot = Workspace:FindFirstChild(worldRootName)
    Assert.truthy(worldRoot, "expected shellMesh courtyard test world root")

    local buildingModel =
        worldRoot:FindFirstChild("0_0"):FindFirstChild("Buildings"):FindFirstChild("shellmesh_courtyard")
    Assert.truthy(buildingModel, "expected shellMesh courtyard building model")

    local shellWallEvidenceCount = 0
    local roofParts = 0
    for _, descendant in ipairs(buildingModel:GetDescendants()) do
        if descendant:IsA("BasePart") then
            if descendant:GetAttribute("ArnisShellWallEvidence") == true then
                shellWallEvidenceCount += 1
            end
            if string.find(descendant.Name, "_roof", 1, true) then
                roofParts += 1
            end
        end
    end

    Assert.truthy(
        shellWallEvidenceCount >= 8,
        "expected shellMesh courtyard building to preserve explicit outer and inner wall evidence"
    )
    Assert.truthy(roofParts >= 1, "expected shellMesh courtyard building to keep roof geometry")

    local courtyardProbe = Workspace:GetPartBoundsInBox(CFrame.new(24, 16.2, 24), Vector3.new(6, 2, 6))
    local courtyardRoofParts = 0
    for _, part in ipairs(courtyardProbe) do
        if part:IsDescendantOf(buildingModel) and string.find(part.Name, "_roof", 1, true) then
            courtyardRoofParts += 1
        end
    end

    Assert.equal(courtyardRoofParts, 0, "expected shellMesh courtyard void to remain open")

    worldRoot:Destroy()
end
