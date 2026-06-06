CREATE TABLE IF NOT EXISTS sys_sync_device (
    device_name TEXT PRIMARY KEY,
    home_dir TEXT NOT NULL,
    os TEXT NOT NULL,
    arch TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS biz_sync_root (
    device_name TEXT NOT NULL REFERENCES sys_sync_device(device_name) ON DELETE CASCADE,
    alias TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    space_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (device_name, alias),
    CONSTRAINT biz_sync_root_relative_path_home_bound CHECK (
        relative_path <> ''
        AND relative_path <> '.'
        AND relative_path <> '..'
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
    )
);

CREATE INDEX IF NOT EXISTS idx_biz_sync_root_space_id
    ON biz_sync_root(space_id);

CREATE TABLE IF NOT EXISTS biz_sync_file_record (
    space_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    file_kind TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    crdt_version BYTEA NOT NULL DEFAULT ''::bytea,
    status TEXT NOT NULL,
    size_bytes BIGINT,
    updated_by_device TEXT NOT NULL REFERENCES sys_sync_device(device_name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (space_id, relative_path),
    CONSTRAINT biz_sync_file_record_kind_check CHECK (
        file_kind IN ('text', 'binary', 'directory', 'missing')
    ),
    CONSTRAINT biz_sync_file_record_status_check CHECK (
        status IN ('synced', 'syncing', 'error', 'shared', 'deleted')
    ),
    CONSTRAINT biz_sync_file_record_size_check CHECK (
        size_bytes IS NULL OR size_bytes >= 0
    ),
    CONSTRAINT biz_sync_file_record_relative_path_home_bound CHECK (
        relative_path <> ''
        AND relative_path <> '.'
        AND relative_path <> '..'
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
    )
);

CREATE INDEX IF NOT EXISTS idx_biz_sync_file_record_space_updated
    ON biz_sync_file_record(space_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_biz_sync_file_record_device
    ON biz_sync_file_record(updated_by_device);

CREATE TABLE IF NOT EXISTS biz_sync_crdt_update_log (
    id BIGSERIAL PRIMARY KEY,
    space_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    source_device TEXT NOT NULL REFERENCES sys_sync_device(device_name),
    base_version BYTEA,
    version BYTEA NOT NULL,
    blob BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT biz_sync_crdt_update_log_relative_path_home_bound CHECK (
        relative_path <> ''
        AND relative_path <> '.'
        AND relative_path <> '..'
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
    )
);

CREATE INDEX IF NOT EXISTS idx_biz_sync_crdt_update_log_file_created
    ON biz_sync_crdt_update_log(space_id, relative_path, created_at);

CREATE INDEX IF NOT EXISTS idx_biz_sync_crdt_update_log_source_device
    ON biz_sync_crdt_update_log(source_device);

CREATE TABLE IF NOT EXISTS biz_sync_object_metadata (
    space_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    chunk_size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, relative_path, content_hash),
    CONSTRAINT biz_sync_object_metadata_size_check CHECK (size_bytes >= 0),
    CONSTRAINT biz_sync_object_metadata_chunk_size_check CHECK (chunk_size_bytes > 0),
    CONSTRAINT biz_sync_object_metadata_relative_path_home_bound CHECK (
        relative_path <> ''
        AND relative_path <> '.'
        AND relative_path <> '..'
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
    )
);

CREATE TABLE IF NOT EXISTS biz_sync_object_chunk (
    space_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    chunk_index BIGINT NOT NULL,
    offset_bytes BIGINT NOT NULL,
    size_bytes BIGINT NOT NULL,
    object_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, relative_path, content_hash, chunk_index),
    FOREIGN KEY (space_id, relative_path, content_hash)
        REFERENCES biz_sync_object_metadata(space_id, relative_path, content_hash)
        ON DELETE CASCADE,
    CONSTRAINT biz_sync_object_chunk_index_check CHECK (chunk_index >= 0),
    CONSTRAINT biz_sync_object_chunk_offset_check CHECK (offset_bytes >= 0),
    CONSTRAINT biz_sync_object_chunk_size_check CHECK (size_bytes >= 0),
    CONSTRAINT biz_sync_object_chunk_object_key_check CHECK (object_key <> '')
);

CREATE INDEX IF NOT EXISTS idx_biz_sync_object_chunk_key
    ON biz_sync_object_chunk(object_key);

CREATE TABLE IF NOT EXISTS sys_sync_session (
    session_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL REFERENCES sys_sync_device(device_name) ON DELETE CASCADE,
    connected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    remote_addr TEXT,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_sys_sync_session_device
    ON sys_sync_session(device_name);
