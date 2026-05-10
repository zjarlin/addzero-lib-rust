CREATE TABLE IF NOT EXISTS sys_api_key (
    id UUID PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES sys_user(id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL UNIQUE,
    key_prefix TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    owner_space_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sys_api_key_user_id ON sys_api_key(user_id);
CREATE INDEX IF NOT EXISTS idx_sys_api_key_prefix ON sys_api_key(key_prefix);
