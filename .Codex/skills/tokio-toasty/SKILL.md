---
name: tokio-toasty
description: Use when the user wants to design, implement, refactor, or integrate a Rust/Tokio toast or transient-notification system that behaves like a Spring Boot starter: add one or a few crates, apply minimal typed configuration, and get a ready-to-use emit/render pipeline. Trigger on requests about toast notifications, snackbar systems, notification buses, Dioxus or Axum toast integration, provider wiring, auto-configuration, or packaging ad-hoc toast helpers into reusable `tokio-toasty` crates.
---

# Tokio Toasty

Build `tokio-toasty` as a starter-style notification system for Rust apps. Optimize for "import crate + small config + usable defaults", not page-local helpers or duplicated UI snippets.

## Default Shape

1. Keep one public facade crate named `tokio-toasty`.
2. Split only when the boundary is real:
- `tokio-toasty-core`: domain types, traits, policies, and config model.
- `tokio-toasty`: default runtime, builder, re-exports, and starter bootstrap.
- Optional adapters such as `tokio-toasty-dioxus`, `tokio-toasty-axum`, or `tokio-toasty-tauri`.
3. Keep application code thin. Put queueing, dedupe, ttl, fan-out, and adapter wiring in crates, not in `apps/*`.

## Design Rules

1. Start from Tokio primitives. Prefer a small manager built on `tokio::sync::{broadcast, watch, mpsc}` over ad-hoc global mutable state.
2. Expose one obvious emission API such as `ToastyHandle::success("Saved")` or `toasty.success("Saved")`.
3. Model first-class fields from day one:
- level
- title
- body
- ttl
- dedupe key
- action label or callback token
- scope or channel
- metadata
4. Keep configuration typed with a small `ToastyConfig` struct plus builder defaults. Do not hide core behavior behind stringly parsing.
5. Make missing renderers or optional sinks degrade gracefully. Prefer no-op or explicit error values over panics.
6. Re-export the common public surface from the facade crate so most consumers do not import subcrates directly.

## Starter Workflow

1. Identify the topology first:
- local UI only
- backend emits and frontend renders
- desktop app with local event bus
2. Choose the smallest crate set that satisfies that topology. Do not create adapters that are not used.
3. Build the core contracts before framework glue:
- toast event model
- config
- emit handle or sink trait
- queue or store semantics
4. Add one install path per framework, such as provider, layer, plugin, or extension registration.
5. Verify the starter path with one integration test that proves minimal config can emit and receive a toast.
6. Add one failure-path test for closed channels, absent adapter wiring, or dedupe behavior.

## Spring Boot Analogy

Treat `tokio-toasty` like a Rust starter, not a widget collection.

1. Prefer convention over repeated manual wiring.
2. Provide one obvious bootstrap API such as `Toasty::builder()`, `Toasty::install(...)`, or a framework-specific `Provider::new(...)`.
3. Keep app-side setup small enough that the happy path fits in a short README snippet.
4. Start with a compact config surface:
- queue capacity
- default ttl
- max visible count
- dedupe policy
- persistence toggle if needed
5. Do not force callers to manually thread sender and receiver pairs across unrelated layers.

## Repository Fit

1. If the work is repo-internal, prefer `crates/runtime/` for runtime and starter crates.
2. Align with existing starter conventions such as `az-starter-*` when the feature is intended to be Addzero-specific.
3. Preserve `tokio-toasty` naming only when the goal is a reusable standalone crate family instead of an Addzero-only starter.
4. Keep domain logic out of `apps/*`; app code should install adapters and call the facade only.

## Output Expectations

When finishing a `tokio-toasty` task, report:

1. The public crate layout.
2. The default bootstrap API.
3. Where configuration enters the system.
4. What the minimal application-side setup looks like.
5. Which tests prove the starter behavior.
