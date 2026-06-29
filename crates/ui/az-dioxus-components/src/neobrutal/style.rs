pub(crate) const NEOBRUTAL_STYLE_ID: &str = "az-dioxus-neobrutal";

pub(crate) const NEOBRUTAL_CSS: &str = r#"
.shell {
  --shell-bg: #fff8e8;
  --shell-ink: #111111;
  --shell-muted: #3a3a3a;
  --shell-line: #111111;
  --shell-main: #ffcf24;
  --shell-accent: #ff5f5f;
  --shell-secondary: #7dd3fc;
  --shell-panel: #ffffff;
  --shell-shadow: 6px 6px 0 var(--shell-line);
  position: relative;
  width: 100vw;
  height: 100vh;
  display: grid;
  grid-template-columns: 318px minmax(0, 1fr);
  gap: 0;
  padding: 0;
  overflow: hidden;
  background:
    linear-gradient(to right, rgba(17, 17, 17, 0.1) 1px, transparent 1px),
    linear-gradient(to bottom, rgba(17, 17, 17, 0.1) 1px, transparent 1px),
    var(--shell-bg);
  background-size: 34px 34px;
  transition:
    grid-template-columns 160ms ease,
    gap 160ms ease;
}

.shell--collapsed {
  grid-template-columns: 0 minmax(0, 1fr);
  gap: 0;
}

.shell--collapsed .sidebar {
  width: 0;
  min-width: 0;
  padding-right: 0;
  padding-left: 0;
  opacity: 0;
  pointer-events: none;
}

.shell--collapsed .titlebar-controls {
  left: 8px;
}

:root[data-theme="dark"] .shell {
  --shell-bg: #1f1f1f;
  --shell-ink: #f8fafc;
  --shell-muted: #d4d4d8;
  --shell-line: #f8fafc;
  --shell-main: #ffcf24;
  --shell-accent: #ff5f5f;
  --shell-secondary: #7dd3fc;
  --shell-panel: #101112;
  background:
    linear-gradient(to right, rgba(248, 250, 252, 0.1) 1px, transparent 1px),
    linear-gradient(to bottom, rgba(248, 250, 252, 0.1) 1px, transparent 1px),
    var(--shell-bg);
  background-size: 34px 34px;
}

.workbench-slot {
  min-width: 0;
  min-height: 0;
}

.sidebar {
  position: relative;
  z-index: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 44px 10px 10px;
  overflow: hidden;
  isolation: isolate;
  color: var(--shell-ink);
  transition:
    opacity 130ms ease,
    padding 160ms ease;
}

.sidebar::before {
  content: "";
  position: absolute;
  inset: 0;
  z-index: 0;
  border: 3px solid var(--shell-line);
  border-radius: 0;
  background: linear-gradient(165deg, #ffffff 0 70%, #fef08a 70% 100%);
  box-shadow: none;
}

:root[data-theme="dark"] .sidebar::before {
  background:
    linear-gradient(165deg, #18181b 0 70%, #854d0e 70% 100%);
}

.sidebar__section {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sidebar__section--actions {
  gap: 6px;
}

.sidebar__section--contents,
.sidebar__section--recent {
  min-height: 0;
  overflow: hidden;
}

.sidebar__heading {
  margin: 4px 0 4px;
  padding: 0 2px;
  color: var(--shell-muted);
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 0;
  text-transform: uppercase;
}

.sidebar-tree {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-tree--primary {
  gap: 4px;
}

.nav-button,
.project-row,
.thread-row,
.settings-button,
.icon-button,
.model-button {
  border: 0;
  color: inherit;
  background: transparent;
  cursor: default;
}

.nav-button,
.project-row,
.settings-button {
  width: 100%;
  min-height: 34px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 8px;
  border: 2px solid transparent;
  border-radius: 8px;
  color: var(--shell-ink);
  font-size: 14px;
  font-weight: 850;
  text-align: left;
  text-decoration: none;
  cursor: pointer;
}

.thread-row {
  width: 100%;
  min-height: 30px;
  padding: 5px 8px 5px 34px;
  border-radius: 6px;
  color: var(--shell-muted);
  font-size: 13px;
  font-weight: 750;
  line-height: 1.25;
  text-align: left;
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nav-button:hover,
.project-row:hover,
.thread-row:hover,
.settings-button:hover,
.nav-button--active {
  border-color: var(--shell-line);
  background: #ffffff;
  box-shadow: 3px 3px 0 var(--shell-line);
  color: #111111;
}

.nav-button--active,
.settings-button--active {
  background: var(--shell-main);
  color: #111111;
}

.nav-button--tree {
  padding-left: calc(6px + var(--tree-indent, 0px));
}

.nav-button__icon,
.project-row__icon,
.settings-button__icon {
  width: 16px;
  display: inline-flex;
  justify-content: center;
  color: var(--shell-ink);
  font-size: 15px;
  font-weight: 900;
}

.nav-button__label,
.project-row__label,
.settings-button__label {
  min-width: 0;
  overflow: hidden;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nav-button__detail {
  min-width: 28px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  padding: 2px 6px;
  overflow: hidden;
  border: 2px solid var(--shell-line);
  border-radius: 5px;
  background: #ffffff;
  color: var(--shell-muted);
  font-size: 12px;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar__footer {
  position: relative;
  z-index: 1;
  margin-top: auto;
}

.settings-button {
  min-height: 32px;
  padding-left: 0;
}

.plugin-group {
  display: grid;
  gap: 2px;
}

.plugin-group[open] {
  gap: 4px;
}

.plugin-group__summary {
  list-style: none;
  cursor: default;
}

.plugin-group__summary::-webkit-details-marker {
  display: none;
}

.plugin-group__chevron {
  margin-left: auto;
  color: var(--shell-ink);
  font-size: 13px;
  font-weight: 900;
  transition: transform 120ms ease;
}

.plugin-group[open] .plugin-group__chevron {
  transform: rotate(180deg);
}

.plugin-group__panel {
  min-width: 0;
  margin-left: 7px;
  padding: 6px 0 6px 12px;
  border-left: 3px solid var(--shell-line);
}

.sidebar-menu-search {
  min-width: 0;
  min-height: 34px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 7px;
  border: 2px solid var(--shell-line);
  border-radius: 5px;
  background: var(--shell-panel);
  box-shadow: 3px 3px 0 var(--shell-line);
  color: var(--shell-ink);
}

.sidebar-menu-search__icon {
  width: 18px;
  flex: 0 0 auto;
  font-size: 15px;
  font-weight: 900;
  text-align: center;
}

.sidebar-menu-search input {
  min-width: 0;
  width: 100%;
  border: 0;
  outline: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 12px;
  font-weight: 850;
}

.sidebar-menu-search input::placeholder {
  color: color-mix(in srgb, var(--shell-ink) 58%, transparent);
}

.sidebar-menu-shell {
  min-width: 0;
  display: grid;
  gap: 6px;
}

.sidebar-menu-domain,
.sidebar-menu-node {
  min-width: 0;
}

.sidebar-tree--domain {
  margin-left: 2px;
  padding-left: 7px;
  border-left: 3px solid var(--shell-line);
}

.sidebar-tree-branch {
  min-width: 0;
  position: relative;
  display: grid;
  gap: 2px;
}

.sidebar-tree-branch > summary {
  list-style: none;
}

.sidebar-tree-branch > summary::-webkit-details-marker {
  display: none;
}

.sidebar-tree-branch__summary {
  cursor: default;
}

.sidebar-tree-branch__chevron {
  margin-left: auto;
  color: var(--shell-ink);
  font-size: 11px;
  font-weight: 900;
  transition: transform 120ms ease;
}

.sidebar-tree-branch[open] > summary .sidebar-tree-branch__chevron {
  transform: rotate(180deg);
}

.sidebar-tree--nested {
  position: relative;
  gap: 2px;
  padding-block: 1px 2px;
}

.sidebar-tree--nested::before {
  content: "";
  position: absolute;
  top: -2px;
  bottom: 4px;
  left: var(--tree-line, 8px);
  border-left: 2px solid var(--shell-line);
}

.sidebar-tree--nested > .sidebar-menu-node,
.sidebar-tree--nested > .sidebar-tree-branch {
  position: relative;
}

.sidebar-tree--nested > .sidebar-menu-node::before,
.sidebar-tree--nested > .sidebar-tree-branch::before {
  content: "";
  position: absolute;
  top: 15px;
  left: var(--tree-parent-line, 8px);
  width: 10px;
  border-top: 2px solid var(--shell-line);
}

.nav-button--plugin {
  min-height: 28px;
  padding: 3px 6px;
  font-size: 13px;
}

.nav-button--plugin .nav-button__icon {
  font-size: 14px;
}

.titlebar-controls {
  position: absolute;
  z-index: 6;
  top: 8px;
  left: 10px;
  height: 30px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-button,
.sidebar-toggle {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--shell-line);
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 3px 3px 0 var(--shell-line);
  color: var(--shell-ink);
  font-size: 18px;
  text-decoration: none;
  cursor: pointer;
}

.sidebar-toggle {
  position: relative;
}

.sidebar-toggle__glyph {
  width: 15px;
  height: 15px;
  position: relative;
  display: block;
  border: 1.5px solid currentColor;
  border-radius: 4px;
  color: var(--shell-ink);
}

.sidebar-toggle__glyph::before {
  content: "";
  position: absolute;
  inset: 2px auto 2px 4px;
  width: 1.5px;
  border-radius: 1px;
  background: currentColor;
  opacity: 0.9;
}

.shell--collapsed .sidebar-toggle__glyph::before {
  inset: 2px 4px 2px auto;
}

.titlebar-nav {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--shell-line);
  border-radius: 7px;
  background: #ffffff;
  color: var(--shell-ink);
  font-size: 24px;
  line-height: 1;
}

.titlebar-nav--disabled {
  opacity: 0.55;
}

.workspace {
  position: relative;
  z-index: 2;
  min-width: 0;
  min-height: 0;
  margin: 0;
  display: grid;
  grid-template-rows: 56px 1fr;
  overflow: hidden;
  border: 3px solid var(--shell-line);
  border-left: 0;
  border-radius: 0;
  background: var(--shell-panel);
  box-shadow: none;
  transition:
    margin 160ms ease,
    border-radius 160ms ease;
}

.shell--collapsed .workspace {
  margin-left: 0;
  border-left: 3px solid var(--shell-line);
  border-radius: 0;
}

.header-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 18px;
  border-bottom: 3px solid var(--shell-line);
  background: #ffffff;
}

:root[data-theme="dark"] .header-bar,
:root[data-theme="dark"] .model-button,
:root[data-theme="dark"] .icon-button,
:root[data-theme="dark"] .titlebar-nav,
:root[data-theme="dark"] .titlebar-controls .sidebar-toggle {
  background: #18181b;
  color: #f8fafc;
}

.header-bar__actions {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.model-button {
  min-width: 52px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 9px;
  border: 2px solid var(--shell-line);
  border-radius: 8px;
  background: #fafafa;
  box-shadow: 3px 3px 0 var(--shell-line);
  color: var(--shell-ink);
  font-size: 13px;
  font-weight: 850;
  cursor: pointer;
}

.model-button__mark {
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--shell-line);
  border-radius: 6px;
  background: var(--shell-secondary);
  color: #ffffff;
  font-size: 10px;
}

.model-button__chevron {
  color: currentColor;
  font-size: 14px;
}

.workspace__body {
  position: relative;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-rows: minmax(0, 1fr);
  padding: 0;
  overflow: hidden;
}

.workspace__body--lowcode {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
}

.workspace__body--catalog {
  display: block;
  padding: 0;
  overflow: auto;
}

.content-center-slot {
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.content-center-slot--plugin {
  display: block;
  background: var(--content);
}

.content-center-slot--welcome {
  display: grid;
  place-items: center;
  padding: 44px 24px;
}

.workspace__body--lowcode .content-center-slot--plugin {
  overflow: hidden;
}

.page {
  --page-bg: #fff8e8;
  --page-ink: #111111;
  --page-muted: #3a3a3a;
  --page-line: #111111;
  --page-main: #ffcf24;
  --page-accent: #ff5f5f;
  --page-secondary: #7dd3fc;
  --page-panel: #ffffff;
  --page-shadow: 6px 6px 0 var(--page-line);
  --page-radius: 8px;
  min-height: 100%;
  padding: 30px;
  display: grid;
  align-content: start;
  gap: 24px;
  overflow: auto;
  background:
    linear-gradient(to right, rgba(17, 17, 17, 0.11) 1px, transparent 1px),
    linear-gradient(to bottom, rgba(17, 17, 17, 0.11) 1px, transparent 1px),
    var(--page-bg);
  background-size: 34px 34px;
  color: var(--page-ink);
}

.hero,
.card {
  border: 3px solid var(--page-line);
  border-radius: var(--page-radius);
  background: var(--page-panel);
  box-shadow: var(--page-shadow);
  color: var(--page-ink);
}

.hero {
  min-height: 178px;
  padding: 24px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(240px, 360px);
  gap: 20px;
  align-items: end;
  background:
    radial-gradient(circle at 92% 18%, var(--page-secondary) 0 78px, transparent 80px),
    linear-gradient(135deg, #fff 0 58%, var(--page-main) 58% 100%);
}

.hero--compact {
  min-height: 126px;
}

.hero h1 {
  margin: 4px 0 0;
  color: var(--page-ink);
  font-size: clamp(28px, 4vw, 52px);
  font-weight: 900;
  line-height: 1;
}

.hero p {
  max-width: 720px;
  margin: 12px 0 0;
  color: var(--page-muted);
  font-size: 15px;
  line-height: 1.55;
}

.card {
  min-width: 0;
  padding: 18px;
}

.card--accent {
  background: #fef08a;
}

.card--selected {
  box-shadow: 3px 3px 0 var(--page-line);
  transform: translate(3px, 3px);
}

.button {
  min-height: 34px;
  padding: 7px 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 3px solid var(--page-line);
  border-radius: 7px;
  background: #ffffff;
  box-shadow: 4px 4px 0 var(--page-line);
  color: var(--page-ink);
  font-size: 13px;
  font-weight: 850;
  line-height: 1;
  text-decoration: none;
  cursor: pointer;
  transition:
    transform 120ms ease,
    box-shadow 120ms ease;
}

.button:hover {
  transform: translate(3px, 3px);
  box-shadow: 1px 1px 0 var(--page-line);
}

.button--primary {
  background: var(--page-accent);
  color: #ffffff;
}

.eyebrow {
  margin: 0;
  color: var(--page-ink);
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 0;
  text-transform: uppercase;
}

.block-title {
  display: grid;
  gap: 6px;
}

.block-title h2 {
  margin: 0;
  color: var(--page-ink);
  font-size: 21px;
  font-weight: 900;
  line-height: 1.12;
}

.block-title p {
  margin: 0;
  color: var(--page-muted);
  font-size: 13px;
  line-height: 1.48;
}

.badge {
  min-height: 28px;
  padding: 4px 9px;
  display: inline-flex;
  align-items: center;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  color: var(--page-ink);
  font-size: 12px;
  font-weight: 820;
  line-height: 1;
}

.badge--accent {
  background: var(--page-secondary);
}

.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.split {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(300px, 420px);
  gap: 20px;
  align-items: start;
}

.field {
  min-width: 0;
  display: grid;
  gap: 7px;
}

.field__label {
  color: var(--page-ink);
  font-size: 13px;
  font-weight: 900;
}

.field__hint {
  color: var(--page-muted);
  font-size: 12px;
  line-height: 1.35;
}

.input {
  width: 100%;
  min-width: 0;
  min-height: 40px;
  padding: 8px 10px;
  border: 3px solid var(--page-line);
  border-radius: 7px;
  background: #ffffff;
  color: var(--page-ink);
  font: inherit;
  font-size: 13px;
  outline: 0;
}

.input:focus {
  box-shadow: 0 0 0 3px var(--page-secondary);
}

.code-block {
  min-width: 0;
  max-width: 100%;
  margin: 0;
  padding: 12px;
  overflow: auto;
  border: 3px solid var(--page-line);
  border-radius: 7px;
  background: #111111;
  color: #f8fafc;
  font-size: 12px;
  line-height: 1.55;
  white-space: pre-wrap;
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

.codex-start {
  width: min(724px, calc(100vw - 496px));
  min-width: 320px;
  display: grid;
  gap: 18px;
  justify-items: stretch;
  margin-top: clamp(10px, 17vh, 180px);
  align-self: start;
}

.codex-start h1 {
  margin: 0 0 22px;
  color: #26262b;
  font-size: 30px;
  font-weight: 560;
  line-height: 1.2;
  text-align: center;
}

.empty-panel--compact {
  display: none;
}

.project-layout {
  position: absolute;
  inset: 0;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  pointer-events: none;
}

.right-slot {
  width: 340px;
  min-height: 0;
  overflow: auto;
  border-left: 1px solid var(--line-soft);
  background: rgba(247, 247, 248, 0.92);
}

.floating-panel-slot {
  position: absolute;
  inset: 0;
  z-index: 10;
  pointer-events: none;
}

.button,
.model-button,
.icon-button,
.titlebar-nav,
.titlebar-controls .sidebar-toggle {
  transition:
    transform 150ms ease,
    box-shadow 150ms ease;
}

.button:hover,
.model-button:hover,
.icon-button:hover,
.titlebar-controls .sidebar-toggle:hover {
  transform: translate(4px, 4px);
  box-shadow: none;
}

@media (max-width: 920px) {
  .shell {
    grid-template-columns: 292px minmax(0, 1fr);
  }

  .hero,
  .grid,
  .split {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .codex-start {
    width: min(620px, calc(100vw - 340px));
  }
}

@media (max-width: 720px) {
  .shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    display: none;
  }

  .hero,
  .grid,
  .split {
    grid-template-columns: 1fr;
  }

  .workspace {
    margin: 0;
    border: 0;
    border-radius: 0;
  }

  .titlebar-controls {
    display: none;
  }

  .codex-start {
    width: min(100%, 620px);
    min-width: 0;
    margin-top: 10vh;
  }

  .content-center-slot--welcome {
    padding: 32px 16px;
  }

  .codex-start h1 {
    font-size: 26px;
  }
}
"#;
