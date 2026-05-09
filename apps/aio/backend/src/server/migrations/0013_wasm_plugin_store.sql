CREATE TABLE IF NOT EXISTS wasm_plugin_packages (
    plugin_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    summary TEXT NOT NULL,
    descriptor JSONB NOT NULL,
    runtime JSONB NOT NULL DEFAULT '{}'::jsonb,
    default_instance_label TEXT,
    binary_bucket TEXT NOT NULL,
    binary_object_key TEXT NOT NULL,
    binary_sha256 TEXT NOT NULL,
    binary_size_bytes BIGINT NOT NULL,
    source_format TEXT NOT NULL DEFAULT 'azplugin',
    firmware_kind TEXT NOT NULL DEFAULT 'business',
    status TEXT NOT NULL DEFAULT 'available',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE IF EXISTS wasm_plugin_packages
    ADD COLUMN IF NOT EXISTS firmware_kind TEXT NOT NULL DEFAULT 'business';

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_status
    ON wasm_plugin_packages(status);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_descriptor
    ON wasm_plugin_packages USING GIN(descriptor);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_firmware_kind
    ON wasm_plugin_packages(firmware_kind);

CREATE TABLE IF NOT EXISTS wasm_plugin_instances (
    slug TEXT PRIMARY KEY,
    plugin_id TEXT NOT NULL REFERENCES wasm_plugin_packages(plugin_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'installed',
    page_ids TEXT[] NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_instances_plugin
    ON wasm_plugin_instances(plugin_id);

CREATE INDEX IF NOT EXISTS idx_wasm_plugin_instances_status
    ON wasm_plugin_instances(status);
