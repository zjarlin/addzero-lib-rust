use dioxus::prelude::*;

use crate::style::inline_style;

const COMPONENT_STYLE_ID: &str = "az-dioxus-components";

const COMPONENT_CSS: &str = r#"
.surface-card {
  min-width: 0;
  border: 1px solid var(--line-soft);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.035);
}

.surface-card__body {
  min-width: 0;
  padding: 14px;
}

.toolbar-button {
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.07);
  color: var(--text);
  font-size: 13px;
  text-decoration: none;
  white-space: nowrap;
}

.toolbar-button--primary {
  background: #f2f2f0;
  color: #191a1b;
}

.toolbar-button--danger {
  border-color: rgba(220, 95, 69, 0.3);
  background: rgba(220, 95, 69, 0.1);
  color: var(--warning);
}

.toolbar-button--danger:hover {
  background: rgba(220, 95, 69, 0.2);
}

.toolbar-button--compact {
  min-height: 24px;
  height: 24px;
  padding: 0 8px;
  font-size: 11px;
}

.toolbar-button--disabled,
.toolbar-button:disabled {
  opacity: 0.45;
  pointer-events: none;
}

.toolbar-button--page {
  min-width: 24px;
  text-align: center;
}

.toolbar-button--table-gap {
  margin-left: 4px;
}

.status-badge {
  min-height: 20px;
  display: inline-flex;
  align-items: center;
  padding: 1px 7px;
  border: 1px solid var(--line-soft);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.2;
  white-space: nowrap;
}

.status-badge--accent {
  border-color: rgba(47, 155, 255, 0.3);
  background: rgba(47, 155, 255, 0.12);
  color: var(--accent);
}

.status-badge--warn {
  border-color: rgba(255, 122, 50, 0.3);
  background: rgba(255, 122, 50, 0.1);
  color: var(--warning);
}

.table-view__scroller {
  min-width: 0;
  overflow: auto;
}

.table-view {
  width: 100%;
  border-collapse: collapse;
  color: var(--text-muted);
  font-size: 12px;
  table-layout: auto;
}

.table-view--bordered {
  border-collapse: separate;
  border-spacing: 0;
}

.table-view__header-cell,
.table-view__cell {
  padding: 6px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  text-align: left;
  vertical-align: middle;
  white-space: nowrap;
}

.table-view--frozen-header .table-view__header-cell,
.table-view__header-cell {
  position: sticky;
  top: 0;
  z-index: 1;
  background: rgba(24, 26, 27, 0.96);
  color: var(--text);
  font-size: 11px;
  font-weight: 640;
  text-transform: none;
  letter-spacing: 0;
}

.table-view__cell--frozen {
  position: sticky;
  z-index: 2;
  background: rgba(20, 22, 23, 0.98);
  box-shadow: 1px 0 0 var(--line-soft);
}

.table-view--frozen-header .table-view__header-cell.table-view__cell--frozen {
  z-index: 3;
}

:root[data-theme="light"] .table-view__cell--frozen {
  background: rgba(255, 255, 255, 0.98);
}

.table-view__cell {
  overflow: hidden;
  line-height: 1.35;
  text-overflow: ellipsis;
}

.table-view__body tr:nth-child(even) .table-view__cell {
  background: rgba(255, 255, 255, 0.018);
}

.table-view__body tr:hover .table-view__cell {
  background: rgba(255, 255, 255, 0.055);
}

.table-view--dense .table-view__header-cell,
.table-view--dense .table-view__cell {
  padding: 4px 8px;
}

.table-view__body tr:last-child .table-view__cell {
  border-bottom: 0;
}

.table-view__cell--center {
  text-align: center;
}

.table-view__cell--end,
.table-view__cell--numeric {
  text-align: right;
}

.table-view__cell--empty {
  color: var(--text-subtle);
  text-align: center;
  white-space: normal;
}

.table-view__cell code {
  padding: 1px 5px;
  border-radius: 4px;
  background: rgba(47, 155, 255, 0.08);
  color: var(--accent);
  font-size: 11px;
}

.table-view__cell .toolbar-button {
  height: 24px;
  padding: 0 8px;
  border-radius: 5px;
  font-size: 11px;
}

.grammar-search__box {
  min-height: 38px;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 0 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.07);
  color: var(--text-subtle);
}

.grammar-search__icon {
  flex: 0 0 auto;
  color: var(--text-muted);
  font-size: 15px;
}

.grammar-search__input {
  width: 100%;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 14px;
}

.grammar-search__input::placeholder {
  color: var(--text-subtle);
}

.grammar-search__tokens,
.grammar-search__fields {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.grammar-search__token,
.grammar-search__field {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 3px 8px;
  border: 1px solid var(--line-soft);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.055);
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.2;
}

.grammar-search__token {
  border-color: rgba(47, 155, 255, 0.26);
  background: rgba(47, 155, 255, 0.12);
}

.grammar-search__token--term {
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.07);
}

.grammar-search__token-key,
.grammar-search__field-key {
  color: #d7ecff;
  font-weight: 700;
}

.grammar-search__token-separator,
.grammar-search__field-label {
  color: var(--text-subtle);
}

.form-grid,
.settings-form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 12px;
}

.form-grid--wide {
  grid-template-columns: 1fr;
}

.form-row__required {
  margin-left: 4px;
  color: var(--warning);
}

.settings-input {
  width: 100%;
  min-width: 0;
  height: 34px;
  padding: 0 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  outline: 0;
  background: rgba(0, 0, 0, 0.16);
  color: var(--text);
  font: inherit;
  font-size: 14px;
}

.settings-input:focus {
  border-color: rgba(47, 155, 255, 0.5);
}

.checkbox-row {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--text-muted);
  font-size: 12px;
}

.checkbox-row input {
  width: auto;
  height: auto;
}

.accordion {
  margin-bottom: 6px;
  border: 1px solid var(--line-soft);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.035);
}

.accordion__summary {
  padding: 12px 14px;
  border-bottom: 1px solid var(--line-soft);
  color: var(--text);
  font-size: 14px;
  font-weight: 620;
  cursor: pointer;
}

.accordion[open] .accordion__summary {
  border-bottom-color: var(--line);
}

.accordion__body {
  display: grid;
  gap: 10px;
  padding: 12px 14px;
}

.accordion--inline {
  display: inline-block;
  margin: 0;
}

.accordion--tree-form {
  margin: 6px 8px;
  border: 0;
  background: rgba(255, 255, 255, 0.04);
}

.accordion__summary--compact,
.compact-summary {
  padding: 2px 6px;
  font-size: 11px;
}

.accordion__body--compact {
  gap: 6px;
  padding: 8px 10px;
}

.form-input--compact {
  height: 28px;
  font-size: 12px;
}

.workbench-tree__list--tight {
  gap: 0;
}

.workbench-tree__list--tight .nav-button {
  min-height: 24px;
  padding: 3px 8px;
  gap: 6px;
  border-radius: 0;
  font-size: 12px;
}

.empty-panel {
  width: min(520px, 100%);
  display: grid;
  justify-items: center;
  gap: 12px;
  padding: 32px;
  color: var(--text);
  text-align: center;
}

.empty-panel__mark {
  width: 42px;
  height: 42px;
  display: grid;
  place-items: center;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--content-raised);
  color: var(--text-muted);
  font-size: 22px;
}

.empty-panel h1 {
  margin: 0;
  font-size: 28px;
  font-weight: 650;
  letter-spacing: 0;
}

.empty-panel p {
  max-width: 420px;
  margin: 0;
  color: var(--text-subtle);
  font-size: 14px;
  line-height: 1.6;
}
"#;

pub(crate) fn component_style() -> Element {
    inline_style(COMPONENT_STYLE_ID, COMPONENT_CSS)
}
