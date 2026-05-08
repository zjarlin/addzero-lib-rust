CREATE TABLE IF NOT EXISTS ai_model_providers (
    provider TEXT PRIMARY KEY,
    base_url TEXT,
    default_model TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    key_id TEXT NOT NULL DEFAULT 'default',
    encrypted_api_key TEXT,
    api_key_configured BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE ai_model_providers ADD COLUMN IF NOT EXISTS base_url TEXT;
