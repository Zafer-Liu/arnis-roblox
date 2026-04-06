--!strict
-- RoadSidewalkCurb.spec.lua
-- Verifies that RoadBuilder sidewalk enum and layer offset logic exists.

return function()
    local RoadChunkPlan = require(game:GetService("ServerScriptService").ImportService.RoadChunkPlan)

    describe("RoadChunkPlan sidewalk enum", function()
        it("should resolve sidewalk mode from road.sidewalk enum", function()
            -- Build a plan with a road that has sidewalk = "left"
            local roads = {
                {
                    id = "test_left",
                    kind = "residential",
                    sidewalk = "left",
                    points = {
                        { x = 0, y = 0, z = 0 },
                        { x = 10, y = 0, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            expect(plan.roads[1].sidewalkMode).to.equal("left")
        end)

        it("should resolve sidewalk mode from road.sidewalk enum for both", function()
            local roads = {
                {
                    id = "test_both",
                    kind = "primary",
                    sidewalk = "both",
                    points = {
                        { x = 0, y = 0, z = 0 },
                        { x = 10, y = 0, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            expect(plan.roads[1].sidewalkMode).to.equal("both")
        end)

        it("should fall back to hasSidewalk boolean when sidewalk enum is absent", function()
            local roads = {
                {
                    id = "test_bool",
                    kind = "residential",
                    hasSidewalk = true,
                    points = {
                        { x = 0, y = 0, z = 0 },
                        { x = 10, y = 0, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            expect(plan.roads[1].sidewalkMode).to.equal("both")
        end)

        it("should resolve separate sidewalk mode", function()
            local roads = {
                {
                    id = "test_separate",
                    kind = "tertiary",
                    sidewalk = "separate",
                    points = {
                        { x = 0, y = 0, z = 0 },
                        { x = 10, y = 0, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            expect(plan.roads[1].sidewalkMode).to.equal("separate")
        end)
    end)

    describe("RoadChunkPlan layer offset", function()
        it("should apply positive layer elevation", function()
            local roads = {
                {
                    id = "test_layer_pos",
                    kind = "motorway",
                    layer = 2,
                    points = {
                        { x = 0, y = 10, z = 0 },
                        { x = 20, y = 10, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            -- Layer 2 at 5 studs/layer = 10 studs above the base Y of 10 = 20
            local seg = plan.roads[1].segments[1]
            expect(seg.p1.Y).to.be.near(20, 1)
        end)

        it("should apply negative layer elevation for underpasses", function()
            local roads = {
                {
                    id = "test_layer_neg",
                    kind = "residential",
                    layer = -1,
                    points = {
                        { x = 0, y = 10, z = 0 },
                        { x = 20, y = 10, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            -- Layer -1 at 5 studs/layer = -5 studs from base Y of 10 = 5
            local seg = plan.roads[1].segments[1]
            expect(seg.p1.Y).to.be.near(5, 1)
        end)

        it("should not offset roads with layer 0", function()
            local roads = {
                {
                    id = "test_layer_zero",
                    kind = "residential",
                    layer = 0,
                    points = {
                        { x = 0, y = 10, z = 0 },
                        { x = 20, y = 10, z = 0 },
                    },
                },
            }
            local origin = { x = 0, y = 0, z = 0 }
            local plan = RoadChunkPlan.build(roads, origin, nil, {})
            local seg = plan.roads[1].segments[1]
            expect(seg.p1.Y).to.be.near(10, 1)
        end)
    end)
end
