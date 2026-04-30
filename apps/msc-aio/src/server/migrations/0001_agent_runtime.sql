CREATE TABLE IF NOT EXISTS agent_artifacts (
    id UUID PRIMARY KEY,
    channel TEXT NOT NULL,
    title TEXT NOT NULL,
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    package_format TEXT NOT NULL,
    download_url TEXT NOT NULL,
    checksum TEXT NOT NULL,
    install_command TEXT NOT NULL,
    launch_command TEXT NOT NULL,
    uninstall_command TEXT NOT NULL,
    service_name TEXT NOT NULL,
    note TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS agent_pairing_sessions (
    id UUID PRIMARY KEY,
    channel TEXT NOT NULL,
    device_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    status TEXT NOT NULL,
    poll_token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    approved_at TIMESTAMPTZ,
    exchanged_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS agent_nodes (
    id UUID PRIMARY KEY,
    display_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    channel TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    status TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    paired_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ,
    last_uploaded_count BIGINT NOT NULL DEFAULT 0,
    last_downloaded_count BIGINT NOT NULL DEFAULT 0,
    last_conflict_count BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS agent_skill_states (
    node_id UUID NOT NULL,
    skill_name TEXT NOT NULL,
    last_synced_hash TEXT NOT NULL,
    last_synced_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (node_id, skill_name)
);

CREATE TABLE IF NOT EXISTS agent_skill_conflicts (
    id UUID PRIMARY KEY,
    node_id UUID NOT NULL,
    skill_name TEXT NOT NULL,
    server_hash TEXT NOT NULL,
    agent_hash TEXT NOT NULL,
    server_updated_at TIMESTAMPTZ,
    agent_updated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolution TEXT,
    agent_keywords TEXT[] NOT NULL DEFAULT '{}',
    agent_description TEXT NOT NULL DEFAULT '',
    agent_body TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS agent_pairing_sessions_status_idx
    ON agent_pairing_sessions (status);

CREATE INDEX IF NOT EXISTS agent_nodes_status_idx
    ON agent_nodes (status);

CREATE INDEX IF NOT EXISTS agent_skill_conflicts_node_idx
    ON agent_skill_conflicts (node_id, skill_name, resolved_at);
