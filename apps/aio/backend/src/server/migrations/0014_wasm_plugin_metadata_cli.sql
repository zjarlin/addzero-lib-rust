ALTER TABLE IF EXISTS wasm_plugin_packages
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS github_url TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS maintainer_type TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS maintainer_name TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS primary_language TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS category TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS install_command TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS agent_install_command TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_metadata
    ON wasm_plugin_packages USING GIN(metadata);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_category
    ON wasm_plugin_packages(category)
    WHERE category != '';

CREATE TABLE IF NOT EXISTS wasm_plugin_cli_commands (
    plugin_id TEXT NOT NULL REFERENCES wasm_plugin_packages(plugin_id) ON DELETE CASCADE,
    command_name TEXT NOT NULL,
    file_name TEXT NOT NULL,
    object_bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    object_sha256 TEXT NOT NULL,
    object_size_bytes BIGINT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text/x-shellscript',
    install_path TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'available',
    installed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (plugin_id, command_name)
);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_cli_commands_status
    ON wasm_plugin_cli_commands(status);
