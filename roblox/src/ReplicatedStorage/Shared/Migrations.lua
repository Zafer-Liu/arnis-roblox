local Version = require(script.Parent.Version)

local Migrations = {}

function Migrations.migrate(manifest, targetVersion)
    assert(type(manifest) == "table", "manifest must be a table")
    assert(
        targetVersion == Version.SchemaVersion,
        ("unsupported target schemaVersion %q; expected %q"):format(
            tostring(targetVersion),
            Version.SchemaVersion
        )
    )
    assert(type(manifest.schemaVersion) == "string", "manifest.schemaVersion must be a string")
    assert(
        manifest.schemaVersion == Version.SchemaVersion,
        ("unsupported manifest schemaVersion %q; expected %q"):format(
            manifest.schemaVersion,
            Version.SchemaVersion
        )
    )

    return manifest
end

return Migrations
