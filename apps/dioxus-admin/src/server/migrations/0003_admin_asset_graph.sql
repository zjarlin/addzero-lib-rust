CREATE TABLE IF NOT EXISTS admin_asset_items (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    detail TEXT NOT NULL,
    source TEXT NOT NULL,
    local_path TEXT,
    relative_path TEXT,
    download_url TEXT,
    content_hash TEXT,
    hash_algorithm TEXT,
    size_bytes BIGINT,
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_asset_tags (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_asset_item_tags (
    item_id TEXT NOT NULL REFERENCES admin_asset_items(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES admin_asset_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE INDEX IF NOT EXISTS admin_asset_items_kind_idx
    ON admin_asset_items (kind, updated_at DESC);

CREATE INDEX IF NOT EXISTS admin_asset_items_hash_idx
    ON admin_asset_items (content_hash)
    WHERE content_hash IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS admin_asset_package_hash_unique
    ON admin_asset_items (hash_algorithm, content_hash)
    WHERE kind = 'package'
      AND hash_algorithm IS NOT NULL
      AND content_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS admin_asset_item_tags_tag_idx
    ON admin_asset_item_tags (tag_id);
