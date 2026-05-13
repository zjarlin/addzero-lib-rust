use std::fs;
use std::io;
use std::path::PathBuf;

use az_lowcode::{ComponentNode, ComponentRegistry, GridArea};

fn main() -> io::Result<()> {
    let html = build_preview_document()?;
    let output_path = preview_output_path()?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;
    println!("Preview written to {}", output_path.display());
    Ok(())
}

fn build_preview_document() -> io::Result<String> {
    let registry = ComponentRegistry::with_builtins();
    let card = registry.render(&preview_node()).map_err(io::Error::other)?;

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>az-edge preview</title>
  <style>{}</style>
</head>
<body>
  <main class="preview-shell">
    <section class="preview-editor">
      <section class="preview-card-stage">
        {}
      </section>
      <aside class="preview-contract-panel">
        <div class="preview-contract-panel__header">
          <span>REST Contract</span>
          <code id="preview-operation-id">api_edge_weather</code>
        </div>
        <pre id="contract-preview"></pre>
      </aside>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"#,
        preview_styles(),
        card,
        preview_script()
    ))
}

fn preview_output_path() -> io::Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join("target")
        .join("az-lowcode-preview")
        .join("az-edge.html"))
}

fn preview_node() -> ComponentNode {
    ComponentNode {
        id: "az-edge-weather".into(),
        type_key: "az-edge".into(),
        props: serde_json::json!({
            "title": "Weather bridge",
            "variant": "curl",
            "method": "POST",
            "path": "/api/edge/weather",
            "template": "curl https://api.example.com/weather?q={{city}}&units={{units}}",
            "inputs": [
                { "name": "city", "type": "string", "description": "City keyword" },
                { "name": "units", "type": "string", "description": "Metric or imperial", "required": false, "default_value": "metric" }
            ],
            "outputs": [
                { "name": "temperature", "type": "number", "description": "Temperature result" },
                { "name": "condition", "type": "string", "description": "Weather summary" }
            ],
            "timeout_secs": 10
        }),
        grid_area: GridArea {
            col_start: 1,
            col_end: 2,
            row_start: 1,
            row_end: 2,
        },
        children: vec![],
    }
}

fn preview_styles() -> &'static str {
    r#"
:root {
  --bg: #0f172a;
  --panel: #111827;
  --panel-2: #1f2937;
  --panel-3: #0b1220;
  --line: #334155;
  --line-2: #475569;
  --ink: #e5eefc;
  --muted: #94a3b8;
  --accent: #22c55e;
  --accent-2: #38bdf8;
}

* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  min-height: 100%;
  background:
    radial-gradient(circle at top, rgba(56, 189, 248, 0.18), transparent 28%),
    linear-gradient(180deg, #0b1120 0%, #111827 100%);
  color: var(--ink);
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
}

body {
  padding: 28px;
}

.preview-shell {
  width: min(100%, 1280px);
  margin: 0 auto;
}

.preview-editor {
  display: grid;
  grid-template-columns: minmax(360px, 1.1fr) minmax(320px, 0.9fr);
  gap: 20px;
  align-items: start;
}

.preview-card-stage,
.preview-contract-panel {
  background: rgba(15, 23, 42, 0.92);
  border: 1px solid var(--line);
  border-radius: 12px;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.35);
}

.preview-card-stage {
  padding: 18px;
}

.preview-contract-panel {
  overflow: hidden;
}

.preview-contract-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--line);
  color: var(--muted);
}

.preview-contract-panel__header code {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  color: var(--accent-2);
}

#contract-preview {
  margin: 0;
  padding: 16px;
  min-height: 520px;
  overflow: auto;
  font-size: 13px;
  line-height: 1.5;
  color: var(--ink);
  background: linear-gradient(180deg, rgba(15, 23, 42, 0.98), rgba(11, 18, 32, 1));
}

.lc-az-edge-card {
  display: grid;
  gap: 16px;
  background: linear-gradient(180deg, rgba(17, 24, 39, 0.96), rgba(15, 23, 42, 0.98));
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 18px;
}

.lc-az-edge-card__summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  color: var(--muted);
  font-size: 13px;
}

.lc-az-edge-card__summary-title {
  color: var(--accent);
  font-weight: 600;
}

.lc-az-edge-card__summary-route {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
}

.lc-az-edge-card__form {
  display: grid;
  gap: 16px;
}

.lc-az-edge-card__header,
.lc-az-edge-card__meta,
.lc-az-edge-card__io {
  display: grid;
  gap: 12px;
}

.lc-az-edge-card__header {
  grid-template-columns: minmax(0, 1fr) 160px;
}

.lc-az-edge-card__meta {
  grid-template-columns: 140px minmax(0, 1fr) 120px;
}

.lc-az-edge-card__io {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.lc-az-edge-card__field,
.lc-az-edge-card__param-block {
  display: grid;
  gap: 8px;
}

.lc-az-edge-card__field span,
.lc-az-edge-card__param-header span {
  color: var(--muted);
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.lc-az-edge-card input,
.lc-az-edge-card select,
.lc-az-edge-card textarea,
.lc-az-edge-card button {
  font: inherit;
}

.lc-az-edge-card input,
.lc-az-edge-card select,
.lc-az-edge-card textarea {
  width: 100%;
  border: 1px solid var(--line-2);
  border-radius: 8px;
  background: var(--panel-2);
  color: var(--ink);
  padding: 10px 12px;
}

.lc-az-edge-card textarea {
  min-height: 140px;
  resize: vertical;
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  line-height: 1.5;
}

.lc-az-edge-card__title-input {
  font-size: 18px;
  font-weight: 600;
}

.lc-az-edge-card__param-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.lc-az-edge-card__param-add,
.lc-az-edge-card__param-remove {
  border: 1px solid var(--line-2);
  border-radius: 8px;
  background: var(--panel-3);
  color: var(--muted);
  padding: 8px 10px;
  cursor: pointer;
}

.lc-az-edge-card__param-list {
  display: grid;
  gap: 10px;
}

.lc-az-edge-card__param-row {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: rgba(11, 18, 32, 0.8);
}

.lc-az-edge-card__param-main {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) 110px minmax(0, 0.9fr) auto auto;
  gap: 8px;
  align-items: center;
}

.lc-az-edge-card__required {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--muted);
  font-size: 13px;
}

.lc-az-edge-card__required input {
  width: auto;
}

@media (max-width: 1024px) {
  .preview-editor {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  body {
    padding: 16px;
  }

  .lc-az-edge-card__header,
  .lc-az-edge-card__meta,
  .lc-az-edge-card__io,
  .lc-az-edge-card__param-main {
    grid-template-columns: 1fr;
  }
}
"#
}

fn preview_script() -> &'static str {
    r#"
const card = document.querySelector('.lc-az-edge-card');
const contractPreview = document.getElementById('contract-preview');
const operationId = document.getElementById('preview-operation-id');

function normalizeType(raw) {
  return ['string', 'number', 'boolean', 'json'].includes(raw) ? raw : 'string';
}

function collectParams(scope) {
  return Array.from(card.querySelectorAll(`.lc-az-edge-card__param-row[data-param-scope="${scope}"]`))
    .map((row) => {
      const name = row.querySelector('[data-field="name"]').value.trim();
      const ty = normalizeType(row.querySelector('[data-field="type"]').value);
      const description = row.querySelector('[data-field="description"]').value.trim();
      const defaultValueRaw = row.querySelector('[data-field="default"]').value.trim();
      const required = row.querySelector('[data-field="required"]').checked;
      const param = { name, type: ty, required };
      if (description) param.description = description;
      if (defaultValueRaw) param.default_value = defaultValueRaw;
      return param;
    })
    .filter((param) => param.name.length > 0);
}

function operationIdFromPath(path) {
  const value = path
    .replace(/^\/+|\/+$/g, '')
    .replace(/[^a-zA-Z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .toLowerCase();
  return value || 'az_edge';
}

function collectCardState() {
  const title = card.querySelector('[name="title"]').value.trim();
  const variant = card.querySelector('[name="variant"]').value;
  const method = card.querySelector('[name="method"]').value;
  const path = card.querySelector('[name="path"]').value.trim();
  const template = card.querySelector('[name="template"]').value;
  const timeoutRaw = card.querySelector('[name="timeout_secs"]').value.trim();

  const result = {
    title,
    variant,
    method,
    path,
    template,
    inputs: collectParams('inputs'),
    outputs: collectParams('outputs'),
  };

  if (timeoutRaw) result.timeout_secs = Number(timeoutRaw);
  return result;
}

function renderContract() {
  const state = collectCardState();
  operationId.textContent = operationIdFromPath(state.path);
  contractPreview.textContent = JSON.stringify({
    operation_id: operationId.textContent,
    method: state.method,
    path: state.path,
    variant: state.variant,
    request_schema: {
      type: 'object',
      properties: Object.fromEntries(state.inputs.map((param) => [param.name, { type: param.type }])),
      required: state.inputs.filter((param) => param.required).map((param) => param.name),
    },
    response_schema: {
      type: 'object',
      properties: Object.fromEntries(state.outputs.map((param) => [param.name, { type: param.type }])),
      required: state.outputs.filter((param) => param.required).map((param) => param.name),
    },
    template: state.template,
    timeout_secs: state.timeout_secs,
  }, null, 2);

  const route = card.querySelector('.lc-az-edge-card__summary-route');
  route.textContent = `${state.method} ${state.path}`;
  card.dataset.runtime = state.variant;
}

function reindexRows(scope) {
  Array.from(card.querySelectorAll(`.lc-az-edge-card__param-row[data-param-scope="${scope}"]`))
    .forEach((row, index) => {
      row.querySelector('[data-field="name"]').name = `${scope}[${index}][name]`;
      row.querySelector('[data-field="type"]').name = `${scope}[${index}][type]`;
      row.querySelector('[data-field="default"]').name = `${scope}[${index}][default_value]`;
      row.querySelector('[data-field="required"]').name = `${scope}[${index}][required]`;
      row.querySelector('[data-field="description"]').name = `${scope}[${index}][description]`;
    });
}

function cloneBlankRow(scope) {
  const list = card.querySelector(`.lc-az-edge-card__param-block[data-param-scope="${scope}"] .lc-az-edge-card__param-list`);
  const source = list.querySelector('.lc-az-edge-card__param-row');
  const row = source.cloneNode(true);
  row.querySelectorAll('input').forEach((input) => {
    if (input.type === 'checkbox') {
      input.checked = true;
    } else {
      input.value = '';
    }
  });
  row.querySelectorAll('select').forEach((select) => {
    select.value = 'string';
  });
  list.appendChild(row);
  reindexRows(scope);
  renderContract();
}

card.addEventListener('click', (event) => {
  const addButton = event.target.closest('[data-add-row]');
  if (addButton) {
    cloneBlankRow(addButton.dataset.addRow);
    return;
  }

  const removeButton = event.target.closest('[data-remove-row]');
  if (removeButton) {
    const row = removeButton.closest('.lc-az-edge-card__param-row');
    const scope = row.dataset.paramScope;
    const siblings = card.querySelectorAll(`.lc-az-edge-card__param-row[data-param-scope="${scope}"]`);
    if (siblings.length > 1) {
      row.remove();
      reindexRows(scope);
    } else {
      row.querySelectorAll('input').forEach((input) => {
        if (input.type === 'checkbox') {
          input.checked = true;
        } else {
          input.value = '';
        }
      });
      row.querySelectorAll('select').forEach((select) => {
        select.value = 'string';
      });
    }
    renderContract();
  }
});

card.addEventListener('input', renderContract);
card.addEventListener('change', renderContract);
renderContract();
"#
}
