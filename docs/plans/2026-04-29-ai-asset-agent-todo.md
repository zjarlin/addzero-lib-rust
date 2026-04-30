# AI Asset Agent Probe Sync TODO

Date: 2026-04-29

This document records the follow-up sync strategy for the independent agent probe project. The current admin implementation defines the database boundary, CRUD APIs, prompt button management, provider settings, and capture-to-asset flow. The probe itself remains a separate project.

## Shared Contract

- Treat every managed object as an `assets` row plus optional `asset_edges`.
- Use PostgreSQL as the formal source of truth; filesystem and scanner outputs are import/export adapters.
- Keep sync idempotent by matching stable external identifiers in `assets.metadata`.
- Write conflicts as explicit sync records or metadata states, not silent overwrites.
- Never expose provider API keys to the probe UI; model credentials are resolved by the server-side provider config.

## Notes

- Direction: capture input and generated notes write to PG first; optional export to markdown-like surfaces can be added later.
- Identity: use `assets.id` for generated notes and `metadata.capture_asset_id` for the originating capture.
- Conflict strategy: when a note is edited outside admin, create a new revision edge instead of overwriting the latest PG body.
- Filesystem mapping: deferred; do not assume notes are markdown files until an explicit export adapter exists.
- PG writeback: write `kind = capture` for raw input and `kind = note` for normalized notes; create `summarizes` and inferred relation edges.

## Skills

- Direction: existing `SKILL.md` sync remains the adapter; admin writes are mirrored into `assets.kind = skill`.
- Identity: use `metadata.skill_name` as the stable external key for matching existing skills.
- Conflict strategy: if PG asset body and `SKILL.md` body diverge, prefer a conflict record with both hashes over automatic replacement.
- Filesystem mapping: `skill_name/SKILL.md` under the configured skills root.
- PG writeback: store `description`, `source`, and sync hashes in `assets.metadata`.

## Software

- Direction: scanner imports installed software into PG; manual edits in admin should update only asset metadata unless a package manager adapter exists.
- Identity: use bundle id, app path, or normalized executable path in `assets.metadata`.
- Conflict strategy: scanner refresh can update technical fields, but user-authored tags and notes must be preserved.
- Filesystem mapping: `/Applications`, `/System/Applications`, and user application directories are read-only scan sources.
- PG writeback: store scan timestamp, source path, platform, and detected version in metadata.

## Packages

- Direction: installer/package scan imports local files, computes BLAKE3, and uploads to MinIO or compatible object storage.
- Identity: content hash is the primary identity; local path is only an observation.
- Conflict strategy: if the same hash appears at multiple paths, merge observations under one asset rather than duplicating packages.
- Filesystem mapping: configurable installer directories, plus object storage relative path after upload.
- PG writeback: store hash, object key, download URL, size, and upload status.

## Prompt Buttons

- Direction: admin UI manages prompt buttons in PG; probe may consume enabled buttons but should not duplicate prompt execution logic.
- Identity: use `ai_prompt_buttons.id` as the stable key.
- Conflict strategy: prompt edits are authoritative in PG; external prompt files, if added, must be imported as drafts first.
- Filesystem mapping: none in v1.
- PG writeback: store target kind, provider, model, enabled state, and prompt template.

## Agent Probe Project Boundary

- The probe should implement per-asset sync workers rather than one generic filesystem crawler.
- Each worker must define source identity, conflict policy, dry-run output, and PG writeback fields before writing data.
- The first viable worker should be notes or skills because both already have admin-side CRUD and graph semantics.
