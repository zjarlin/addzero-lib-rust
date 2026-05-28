CREATE TABLE IF NOT EXISTS clianything_market (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    source_type TEXT NOT NULL,
    entry_kind TEXT NOT NULL,
    vendor_name TEXT NOT NULL DEFAULT '',
    latest_version TEXT NOT NULL DEFAULT '',
    homepage_url TEXT NOT NULL DEFAULT '',
    repo_url TEXT NOT NULL DEFAULT '',
    docs_url TEXT NOT NULL DEFAULT '',
    entry_point TEXT NOT NULL DEFAULT '',
    category_code TEXT NOT NULL DEFAULT '',
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clianything_market_i18n (
    market_id UUID NOT NULL REFERENCES clianything_market(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    display_name TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    description_md TEXT NOT NULL DEFAULT '',
    install_guide_md TEXT NOT NULL DEFAULT '',
    docs_summary TEXT NOT NULL DEFAULT '',
    requires_text TEXT NOT NULL DEFAULT '',
    install_command TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (market_id, locale)
);

CREATE TABLE IF NOT EXISTS clianything_market_category (
    code TEXT PRIMARY KEY,
    label_zh TEXT NOT NULL DEFAULT '',
    label_en TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clianything_market_tag (
    code TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clianything_market_tag_rel (
    market_id UUID NOT NULL REFERENCES clianything_market(id) ON DELETE CASCADE,
    tag_code TEXT NOT NULL REFERENCES clianything_market_tag(code) ON DELETE CASCADE,
    PRIMARY KEY (market_id, tag_code)
);

CREATE TABLE IF NOT EXISTS clianything_market_install_method (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id UUID NOT NULL REFERENCES clianything_market(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    installer_kind TEXT NOT NULL,
    package_id TEXT NOT NULL DEFAULT '',
    command_template TEXT NOT NULL DEFAULT '',
    validation_note TEXT NOT NULL DEFAULT '',
    priority INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS clianything_market_doc_ref (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id UUID NOT NULL REFERENCES clianything_market(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '',
    source_label TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS clianything_market_import_job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name TEXT NOT NULL,
    format TEXT NOT NULL,
    mode TEXT NOT NULL,
    submitted_by TEXT NOT NULL DEFAULT '',
    total_rows INTEGER NOT NULL DEFAULT 0,
    success_rows INTEGER NOT NULL DEFAULT 0,
    failed_rows INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'completed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clianything_market_import_row (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES clianything_market_import_job(id) ON DELETE CASCADE,
    row_index INTEGER NOT NULL,
    slug TEXT NOT NULL DEFAULT '',
    success BOOLEAN NOT NULL DEFAULT FALSE,
    error_message TEXT,
    market_id UUID REFERENCES clianything_market(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS clianything_market_release (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schema_version TEXT NOT NULL,
    entry_count INTEGER NOT NULL DEFAULT 0,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clianything_market_install_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id UUID NOT NULL REFERENCES clianything_market(id) ON DELETE CASCADE,
    method_id UUID,
    platform TEXT NOT NULL,
    installer_kind TEXT NOT NULL,
    command TEXT NOT NULL,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    exit_code INTEGER,
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS clianything_market_status_idx
    ON clianything_market (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS clianything_market_category_idx
    ON clianything_market (category_code);

CREATE INDEX IF NOT EXISTS clianything_market_tag_rel_tag_idx
    ON clianything_market_tag_rel (tag_code);

CREATE INDEX IF NOT EXISTS clianything_market_install_market_idx
    ON clianything_market_install_method (market_id, priority DESC);

CREATE INDEX IF NOT EXISTS clianything_market_doc_market_idx
    ON clianything_market_doc_ref (market_id, locale);

CREATE INDEX IF NOT EXISTS clianything_market_import_job_created_idx
    ON clianything_market_import_job (created_at DESC);

CREATE INDEX IF NOT EXISTS clianything_market_install_history_market_idx
    ON clianything_market_install_history (market_id, created_at DESC);
