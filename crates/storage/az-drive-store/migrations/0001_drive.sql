CREATE TABLE IF NOT EXISTS drive_spaces (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_entries (
    id UUID PRIMARY KEY,
    space_id TEXT NOT NULL,
    root_alias TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    kind TEXT NOT NULL,
    latest_version BIGINT NOT NULL DEFAULT 0,
    latest_hash TEXT,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (space_id, root_alias, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_drive_entries_space_root_path
    ON drive_entries(space_id, root_alias, relative_path);

CREATE TABLE IF NOT EXISTS drive_versions (
    id UUID PRIMARY KEY,
    entry_id UUID NOT NULL REFERENCES drive_entries(id) ON DELETE CASCADE,
    version BIGINT NOT NULL,
    content_hash TEXT NOT NULL,
    object_key TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    device_id TEXT NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (entry_id, version)
);

CREATE TABLE IF NOT EXISTS drive_locks (
    entry_id UUID PRIMARY KEY REFERENCES drive_entries(id) ON DELETE CASCADE,
    owner_device_id TEXT NOT NULL,
    owner_name TEXT NOT NULL,
    token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_conflicts (
    id UUID PRIMARY KEY,
    entry_id UUID NOT NULL REFERENCES drive_entries(id) ON DELETE CASCADE,
    base_version BIGINT,
    local_hash TEXT NOT NULL,
    remote_hash TEXT NOT NULL,
    device_id TEXT NOT NULL,
    conflict_path TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
