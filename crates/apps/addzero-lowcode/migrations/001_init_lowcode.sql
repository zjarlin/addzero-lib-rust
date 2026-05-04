-- Lowcode service — initial schema
-- Tables follow the all-in-pg convention.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ---------------------------------------------------------------------------
-- layouts: top-level container of component nodes
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lowcode_layouts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    nodes       JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- component_defs: registered component type metadata
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lowcode_component_defs (
    type_name    TEXT PRIMARY KEY,
    props_schema JSONB NOT NULL DEFAULT '{}'::jsonb,
    category     TEXT NOT NULL DEFAULT 'general',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- templates: reusable layout templates
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lowcode_templates (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    layout     JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- event_bindings: bindings that connect component events to handlers
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lowcode_event_bindings (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    layout_id    UUID NOT NULL REFERENCES lowcode_layouts(id) ON DELETE CASCADE,
    node_id      UUID NOT NULL,
    event_name   TEXT NOT NULL,
    handler      JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_event_bindings_layout
    ON lowcode_event_bindings(layout_id);
