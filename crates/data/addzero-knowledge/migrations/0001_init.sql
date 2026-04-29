CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS knowledge_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL UNIQUE,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS knowledge_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES knowledge_sources(id) ON DELETE CASCADE,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    filename TEXT NOT NULL,
    source_path TEXT NOT NULL UNIQUE,
    relative_path TEXT NOT NULL,
    bytes BIGINT NOT NULL,
    section_count INTEGER NOT NULL,
    preview TEXT NOT NULL,
    excerpt TEXT NOT NULL,
    headings TEXT[] NOT NULL DEFAULT '{}',
    body TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS knowledge_documents_source_idx
    ON knowledge_documents (source_id, is_active);

CREATE INDEX IF NOT EXISTS knowledge_documents_path_idx
    ON knowledge_documents (source_path);

CREATE INDEX IF NOT EXISTS knowledge_documents_hash_idx
    ON knowledge_documents (content_hash);
