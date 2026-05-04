CREATE TABLE IF NOT EXISTS admin_software_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    vendor TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT '',
    homepage_url TEXT NOT NULL DEFAULT '',
    icon_url TEXT NOT NULL DEFAULT '',
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    trial_platforms JSONB NOT NULL DEFAULT '[]'::jsonb,
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_software_install_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    software_id UUID NOT NULL REFERENCES admin_software_entries(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    installer_kind TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    package_id TEXT NOT NULL DEFAULT '',
    asset_item_id TEXT,
    command_text TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    priority INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS admin_software_entries_title_idx
    ON admin_software_entries (title, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_software_entries_slug_idx
    ON admin_software_entries (slug);

CREATE INDEX IF NOT EXISTS admin_software_install_methods_software_idx
    ON admin_software_install_methods (software_id, priority DESC);

CREATE INDEX IF NOT EXISTS admin_software_install_methods_platform_idx
    ON admin_software_install_methods (platform, installer_kind);

ALTER TABLE IF EXISTS admin_software_entries
    ADD COLUMN IF NOT EXISTS homepage_url TEXT NOT NULL DEFAULT '';

ALTER TABLE IF EXISTS admin_software_install_methods
    ADD COLUMN IF NOT EXISTS asset_item_id TEXT;

DO $$
BEGIN
    IF to_regclass('admin_asset_items') IS NOT NULL
       AND NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'admin_software_install_methods_asset_item_id_fkey'
    ) THEN
        ALTER TABLE admin_software_install_methods
            ADD CONSTRAINT admin_software_install_methods_asset_item_id_fkey
            FOREIGN KEY (asset_item_id)
            REFERENCES admin_asset_items(id)
            ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS admin_software_install_methods_asset_item_idx
    ON admin_software_install_methods (asset_item_id);
