local ReplicatedStorage = game:GetService("ReplicatedStorage")

local Migrations = require(ReplicatedStorage.Shared.Migrations)
local Assert = require(script.Parent.Assert)

return function()
    local function expectMigrationRejected(schemaVersion)
        local manifest = {
            schemaVersion = schemaVersion,
            meta = {
                worldName = "Test",
                generator = "test",
                source = "test",
                metersPerStud = 1,
                chunkSizeStuds = 256,
                bbox = { minLat = 0, minLon = 0, maxLat = 1, maxLon = 1 },
            },
            chunks = {
                {
                    id = "0_0",
                    originStuds = { x = 0, y = 0, z = 0 },
                    roads = { { id = "r1" } },
                    rails = {},
                    buildings = { { id = "b1" } },
                    water = {},
                    props = {},
                },
            },
        }

        local ok, err = pcall(function()
            Migrations.migrate(manifest, "0.4.0")
        end)

        Assert.falsy(ok, "expected schemaVersion " .. schemaVersion .. " to be rejected")
        Assert.truthy(
            tostring(err):find("0.4.0", 1, true) ~= nil or tostring(err):find(schemaVersion, 1, true) ~= nil,
            "expected rejection error to mention schema version"
        )
    end

    expectMigrationRejected("0.1.0")
    expectMigrationRejected("0.2.0")
    expectMigrationRejected("0.3.0")

    local manifest = {
        schemaVersion = "0.4.0",
        meta = { totalFeatures = 5 },
        chunks = {},
    }
    local result = Migrations.migrate(manifest, "0.4.0")
    Assert.equal(result.schemaVersion, "0.4.0", "expected unchanged schema version")
end
