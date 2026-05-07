CREATE TABLE IF NOT EXISTS admin_scripts (
    name TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_admin_scripts_updated_at
    ON admin_scripts (updated_at DESC);
