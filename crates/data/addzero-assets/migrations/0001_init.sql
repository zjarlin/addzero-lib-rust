CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    content_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS assets_kind_updated_idx
    ON assets (kind, updated_at DESC);

CREATE INDEX IF NOT EXISTS assets_tags_gin
    ON assets USING GIN (tags);

CREATE INDEX IF NOT EXISTS assets_content_hash_idx
    ON assets (content_hash);

CREATE TABLE IF NOT EXISTS asset_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    target_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    relation TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (source_asset_id, target_asset_id, relation)
);

CREATE INDEX IF NOT EXISTS asset_edges_source_idx
    ON asset_edges (source_asset_id);

CREATE INDEX IF NOT EXISTS asset_edges_target_idx
    ON asset_edges (target_asset_id);

CREATE TABLE IF NOT EXISTS ai_model_providers (
    provider TEXT PRIMARY KEY,
    default_model TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    key_id TEXT NOT NULL DEFAULT 'default',
    encrypted_api_key TEXT,
    api_key_configured BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ai_prompt_buttons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    prompt_template TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ai_prompt_buttons_target_idx
    ON ai_prompt_buttons (target_kind, enabled);
