CREATE TABLE IF NOT EXISTS admin_knowledge_sources (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_ref TEXT,
    title TEXT NOT NULL DEFAULT '',
    author TEXT,
    locale TEXT NOT NULL DEFAULT 'zh-CN',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    captured_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_raw_items (
    id TEXT PRIMARY KEY,
    source_id TEXT REFERENCES admin_knowledge_sources(id) ON DELETE SET NULL,
    raw_type TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    content_hash TEXT,
    hash_algorithm TEXT,
    mime_type TEXT,
    locale TEXT NOT NULL DEFAULT 'zh-CN',
    author TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    importance_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    quality_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    token_estimate INT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    captured_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT '',
    canonical_key TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    visibility TEXT NOT NULL DEFAULT 'default',
    locale TEXT NOT NULL DEFAULT 'zh-CN',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0,
    importance_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_ai_refresh_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_edges (
    id TEXT PRIMARY KEY,
    from_node_id TEXT NOT NULL REFERENCES admin_knowledge_nodes(id) ON DELETE CASCADE,
    to_node_id TEXT NOT NULL REFERENCES admin_knowledge_nodes(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0,
    weight DOUBLE PRECISION NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL DEFAULT 'system',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (from_node_id, to_node_id, edge_type)
);

CREATE TABLE IF NOT EXISTS admin_knowledge_evidence (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES admin_knowledge_nodes(id) ON DELETE CASCADE,
    raw_item_id TEXT NOT NULL REFERENCES admin_knowledge_raw_items(id) ON DELETE CASCADE,
    evidence_type TEXT NOT NULL,
    excerpt TEXT NOT NULL DEFAULT '',
    offset_start INT,
    offset_end INT,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_suggestions (
    id TEXT PRIMARY KEY,
    suggestion_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    related_id TEXT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    reason TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_exceptions (
    id TEXT PRIMARY KEY,
    exception_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    subject_node_id TEXT REFERENCES admin_knowledge_nodes(id) ON DELETE CASCADE,
    related_node_id TEXT REFERENCES admin_knowledge_nodes(id) ON DELETE CASCADE,
    related_raw_item_id TEXT REFERENCES admin_knowledge_raw_items(id) ON DELETE CASCADE,
    ai_recommendation TEXT NOT NULL DEFAULT '',
    reason TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    resolution TEXT,
    resolved_by TEXT,
    resolved_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_knowledge_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    target_scope TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    stats JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS admin_knowledge_sources_type_idx
    ON admin_knowledge_sources (source_type, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_raw_items_source_idx
    ON admin_knowledge_raw_items (source_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_raw_items_status_idx
    ON admin_knowledge_raw_items (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_raw_items_hash_idx
    ON admin_knowledge_raw_items (hash_algorithm, content_hash)
    WHERE content_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS admin_knowledge_nodes_type_idx
    ON admin_knowledge_nodes (node_type, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_nodes_status_idx
    ON admin_knowledge_nodes (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_nodes_canonical_idx
    ON admin_knowledge_nodes (canonical_key)
    WHERE canonical_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS admin_knowledge_edges_from_idx
    ON admin_knowledge_edges (from_node_id, edge_type);

CREATE INDEX IF NOT EXISTS admin_knowledge_edges_to_idx
    ON admin_knowledge_edges (to_node_id, edge_type);

CREATE INDEX IF NOT EXISTS admin_knowledge_edges_status_idx
    ON admin_knowledge_edges (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_evidence_node_idx
    ON admin_knowledge_evidence (node_id, created_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_evidence_raw_idx
    ON admin_knowledge_evidence (raw_item_id, created_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_suggestions_status_idx
    ON admin_knowledge_suggestions (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_exceptions_status_idx
    ON admin_knowledge_exceptions (status, severity, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_knowledge_jobs_status_idx
    ON admin_knowledge_jobs (status, created_at DESC);
