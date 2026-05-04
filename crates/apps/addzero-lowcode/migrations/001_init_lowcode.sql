-- Lowcode service — initial schema (issue #75)
-- Tables follow the all-in-pg convention with lc_ prefix.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ---------------------------------------------------------------------------
-- lc_layout: top-level container with JSONB schema + optimistic versioning
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lc_layout (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    schema      JSONB NOT NULL,
    version     INT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- lc_component: registered component type metadata
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lc_component (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type_key      TEXT NOT NULL UNIQUE,
    props_schema  JSONB NOT NULL DEFAULT '{}'::jsonb,
    category      TEXT NOT NULL DEFAULT 'basic',
    icon          TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- lc_event_binding: bindings that connect component events to handlers
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS lc_event_binding (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    layout_id        UUID NOT NULL REFERENCES lc_layout(id) ON DELETE CASCADE,
    component_path   TEXT NOT NULL,
    event_type       TEXT NOT NULL,
    handler_type     TEXT NOT NULL,
    handler_config   JSONB NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_event_binding_layout
    ON lc_event_binding(layout_id);
