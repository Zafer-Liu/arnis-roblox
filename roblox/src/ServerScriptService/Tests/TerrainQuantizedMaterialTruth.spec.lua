return function()
    local Workspace = game:GetService("Workspace")
    local TerrainBuilder = require(script.Parent.Parent.ImportService.Builders.TerrainBuilder)
    local Assert = require(script.Parent.Assert)

    Workspace.Terrain:Clear()

    local function readTopVoxelMaterial(centerX, centerZ)
        local region = Region3.new(
            Vector3.new(centerX - 1.9, 4.1, centerZ - 1.9),
            Vector3.new(centerX + 1.9, 7.9, centerZ + 1.9)
        ):ExpandToGrid(4)
        local materials, occupancies = Workspace.Terrain:ReadVoxels(region, 4)
        return materials[1][1][1], occupancies[1][1][1]
    end

    local function assertQuantizedMaterials(caseName, chunk, voxelCenters, expectedMaterials)
        TerrainBuilder.Build(nil, chunk)

        for index, voxelCenter in ipairs(voxelCenters) do
            local material, occupancy = readTopVoxelMaterial(voxelCenter.x, voxelCenter.z)
            Assert.truthy(
                occupancy > 0.01,
                ("expected filled terrain occupancy in %s voxel %d"):format(caseName, index)
            )
            Assert.equal(
                material,
                expectedMaterials[index],
                ("expected %s voxel %d to use the explicit material from its center-owning source cell"):format(
                    caseName,
                    index
                )
            )
        end

        Workspace.Terrain:Clear()
    end

    local ok, err = xpcall(function()
        assertQuantizedMaterials(
            "positive_origin_cell_size_2",
            {
                id = "terrain_quantized_material_truth",
                originStuds = { x = 1, y = 0, z = 1 },
                terrain = {
                    cellSizeStuds = 2,
                    width = 4,
                    depth = 4,
                    heights = table.create(16, 8),
                    materials = {
                        "Grass", "Mud", "Sand", "Mud",
                        "Rock", "Slate", "Ground", "Slate",
                        "Ground", "Mud", "Rock", "Mud",
                        "Rock", "Slate", "Sand", "Slate",
                    },
                    material = "Grass",
                },
            },
            {
                { x = 2, z = 2 },
                { x = 6, z = 2 },
                { x = 2, z = 6 },
                { x = 6, z = 6 },
            },
            {
                Enum.Material.Grass,
                Enum.Material.Sand,
                Enum.Material.Ground,
                Enum.Material.Rock,
            }
        )

        assertQuantizedMaterials(
            "misaligned_negative_origin_cell_size_3",
            {
                id = "terrain_quantized_material_truth_negative_origin",
                originStuds = { x = -5, y = 0, z = -5 },
                terrain = {
                    cellSizeStuds = 3,
                    width = 4,
                    depth = 4,
                    heights = table.create(16, 8),
                    materials = {
                        "Grass", "Mud", "Sand", "Rock",
                        "Rock", "Slate", "Ground", "Grass",
                        "Ground", "Mud", "Rock", "Sand",
                        "Rock", "Slate", "Sand", "Ground",
                    },
                    material = "Grass",
                },
            },
            {
                { x = -6, z = -6 },
                { x = -2, z = -6 },
                { x = 2, z = 2 },
                { x = 6, z = 6 },
            },
            {
                Enum.Material.Grass,
                Enum.Material.Mud,
                Enum.Material.Rock,
                Enum.Material.Ground,
            }
        )
    end, debug.traceback)

    Workspace.Terrain:Clear()

    if not ok then
        error(err, 0)
    end
end
