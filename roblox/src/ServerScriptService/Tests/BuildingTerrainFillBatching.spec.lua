return function()
    local BuildingBuilder = require(script.Parent.Parent.ImportService.Builders.BuildingBuilder)
    local Assert = require(script.Parent.Assert)

    local parent = Instance.new("Folder")
    parent.Name = "BuildingTerrainFillBatching"
    parent.Parent = workspace

    local fillCalls = 0
    local originalFillTerrainBlock = BuildingBuilder._fillTerrainBlock
    BuildingBuilder._fillTerrainBlock = function(_cf, _size, _material)
        fillCalls += 1
    end

    local ok, err = pcall(function()
        local model = BuildingBuilder.FallbackBuild(parent, {
            id = "batched_fill_building",
            footprint = {
                { x = 0, z = 0 },
                { x = 28, z = 0 },
                { x = 28, z = 18 },
                { x = 0, z = 18 },
            },
            baseY = 0,
            height = 16,
            levels = 3,
            roof = "flat",
            usage = "apartments",
            material = "Brick",
        }, { x = 0, y = 0, z = 0 }, {}, nil)

        Assert.truthy(model, "expected fallback build to return a model")
        Assert.equal(fillCalls, 1, "expected rectangular building terrain fill to batch identical interior rows into one FillBlock")
    end)

    BuildingBuilder._fillTerrainBlock = originalFillTerrainBlock
    parent:Destroy()

    if not ok then
        error(err)
    end
end
