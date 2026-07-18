//! 系统字典左树右表管理页面。

use dioxus::prelude::*;
use registry::ui::{
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    dialog::{
        Dialog, DialogBody, DialogClose, DialogContent, DialogDescription, DialogFooter,
        DialogHeader, DialogTitle, DialogTrigger,
    },
    form::{FormContent, FormFieldWrapper, FormGroup, FormLabel},
    input::{Input, InputType},
    table::{Table, TableHead, TableHeader, TableRow},
};

use crate::plugin::api::NativeRenderContext;

/// 字典管理专用 renderer 标识。
pub const DICTIONARY_RENDERER_ID: &str = "system.dictionary.page";

/// 渲染字典类型树和字典项表格。
pub fn dictionary_workbench_page(context: NativeRenderContext) -> Element {
    rsx! {
        div {
            id: "dictionary-workbench",
            class: "dictionary-workbench space-y-4",
            "data-api-base": context.api_base_url,
            "data-active-route": context.active_route,
            Card {
                class: "dictionary-workbench-header",
                CardHeader {
                    class: "dictionary-workbench-header-grid",
                    div {
                        Badge { variant: BadgeVariant::Outline, "PostgreSQL 字典中心" }
                        CardTitle { class: "mt-3 text-lg", "字典管理" }
                        CardDescription {
                            "按作用域组织字典类型，在右侧维护编码、展示名称、原始值和状态。"
                        }
                    }
                    div { class: "dictionary-workbench-actions",
                        Button {
                            id: "dictionary-type-create-button",
                            size: ButtonSize::Sm,
                            "新建字典类型"
                        }
                        Button {
                            id: "dictionary-reload-button",
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            "刷新"
                        }
                    }
                }
                CardContent {
                    p {
                        id: "dictionary-action-status",
                        class: "dictionary-action-status text-sm text-muted-foreground",
                        "正在连接字典数据源..."
                    }
                }
            }

            div { class: "dictionary-workbench-grid",
                Card { class: "dictionary-type-panel",
                    CardHeader {
                        CardTitle { "字典类型树" }
                        CardDescription { "按 scope 分组，点击叶子节点切换右侧表格。" }
                    }
                    CardContent { class: "dictionary-panel-content",
                        input {
                            id: "dictionary-type-search",
                            class: "az-input",
                            r#type: "search",
                            placeholder: "搜索名称或编码",
                            autocomplete: "off",
                        }
                        div {
                            id: "dictionary-type-tree",
                            class: "dictionary-type-tree",
                            p { class: "dictionary-empty-state", "正在加载字典类型..." }
                        }
                    }
                }

                Card { class: "dictionary-item-panel",
                    CardHeader { class: "dictionary-item-header",
                        div { class: "min-w-0",
                            div { class: "dictionary-selected-title-row",
                                h2 { id: "dictionary-selected-name", class: "leading-none font-semibold", "选择一个字典类型" }
                                span { id: "dictionary-selected-status", class: "dictionary-status-badge", "未选择" }
                            }
                            p { id: "dictionary-selected-description", class: "text-muted-foreground text-sm", "左侧字典类型决定当前表格的数据边界。" }
                            div { class: "dictionary-selected-meta",
                                code { id: "dictionary-selected-code", "—" }
                                span { id: "dictionary-selected-scope", "scope: —" }
                                span { id: "dictionary-selected-kind", "raw: —" }
                            }
                        }
                        div { class: "dictionary-workbench-actions",
                            Button {
                                id: "dictionary-type-edit-button",
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: true,
                                "编辑类型"
                            }
                            Button {
                                id: "dictionary-type-delete-button",
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                disabled: true,
                                "删除类型"
                            }
                        }
                    }
                    CardContent { class: "dictionary-panel-content",
                        div { class: "dictionary-table-toolbar",
                            input {
                                id: "dictionary-item-search",
                                class: "az-input",
                                r#type: "search",
                                placeholder: "搜索标签、编码或原始值",
                                autocomplete: "off",
                                disabled: true,
                            }
                            Button {
                                id: "dictionary-item-create-button",
                                size: ButtonSize::Sm,
                                disabled: true,
                                "新建字典项"
                            }
                        }
                        Table { class: "dictionary-item-table",
                            TableHeader {
                                TableRow {
                                    TableHead { "排序" }
                                    TableHead { "展示名称" }
                                    TableHead { "编码" }
                                    TableHead { "原始值" }
                                    TableHead { "状态" }
                                    TableHead { "更新时间" }
                                    TableHead { "操作" }
                                }
                            }
                            tbody { id: "dictionary-item-table-body",
                                tr {
                                    td { colspan: "7", class: "dictionary-empty-state", "请先从左侧选择字典类型。" }
                                }
                            }
                        }
                        div { class: "dictionary-pagination",
                            span { id: "dictionary-pagination-summary", "0 条" }
                            div { class: "dictionary-form-actions",
                                Button {
                                    id: "dictionary-page-previous",
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    disabled: true,
                                    "上一页"
                                }
                                Button {
                                    id: "dictionary-page-next",
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    disabled: true,
                                    "下一页"
                                }
                            }
                        }
                    }
                }
            }
            DictionaryTypeDialog {}
            DictionaryItemDialog {}
        }
        script {
            "data-aio-script": "system-dictionary-workbench",
            dangerous_inner_html: DICTIONARY_WORKBENCH_SCRIPT,
        }
    }
}

#[component]
fn DictionaryTypeDialog() -> Element {
    rsx! {
        Dialog { class: "dictionary-dialog-root",
            DialogTrigger { class: "dictionary-dialog-trigger dictionary-type-dialog-trigger",
                "打开字典类型编辑器"
            }
            DialogContent { class: "dictionary-dialog-content dictionary-type-dialog-content",
                DialogHeader {
                    DialogTitle { "字典类型" }
                    DialogDescription { "维护字典编码、作用域、原始值类型和枚举开放策略。" }
                }
                form { id: "dictionary-type-form", class: "dictionary-dialog-form",
                    DialogBody { class: "dictionary-dialog-body",
                        h4 { id: "dictionary-type-form-title", class: "dictionary-dialog-mode-title", "新建字典类型" }
                        FormGroup { class: "dictionary-form-grid",
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-name", "名称" }
                                FormContent {
                                    Input {
                                        id: "dictionary-type-name",
                                        class: "az-input",
                                        name: "name",
                                        required: true,
                                        placeholder: "例如：笔记类型"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-code", "编码" }
                                FormContent {
                                    Input {
                                        id: "dictionary-type-code",
                                        class: "az-input font-mono",
                                        name: "code",
                                        required: true,
                                        placeholder: "note_type"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-scope", "作用域" }
                                FormContent {
                                    Input {
                                        id: "dictionary-type-scope",
                                        class: "az-input font-mono",
                                        name: "scope",
                                        value: "system",
                                        required: true
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-raw-kind", "原始值类型" }
                                FormContent {
                                    select { id: "dictionary-type-raw-kind", class: "az-input", name: "rawValueKind",
                                        option { value: "string", "字符串" }
                                        option { value: "int", "整数" }
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-sort", "排序" }
                                FormContent {
                                    Input {
                                        id: "dictionary-type-sort",
                                        class: "az-input",
                                        name: "sortIndex",
                                        r#type: InputType::Number,
                                        value: "0"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-type-status", "状态" }
                                FormContent {
                                    select { id: "dictionary-type-status", class: "az-input", name: "status",
                                        option { value: "enabled", "启用" }
                                        option { value: "disabled", "停用" }
                                    }
                                }
                            }
                        }
                        FormFieldWrapper {
                            FormLabel { html_for: "dictionary-type-description", "描述" }
                            FormContent {
                                textarea {
                                    id: "dictionary-type-description",
                                    class: "az-input",
                                    name: "description",
                                    rows: "3",
                                    placeholder: "说明字典的业务用途"
                                }
                            }
                        }
                        label { class: "dictionary-checkbox-field",
                            input { name: "openEnum", r#type: "checkbox" }
                            span { "开放枚举，允许承载未知原始值" }
                        }
                    }
                    DialogFooter {
                        DialogClose { class: "dictionary-dialog-close dictionary-type-dialog-close", "取消" }
                        Button { button_type: "submit", "保存类型" }
                    }
                }
            }
        }
    }
}

#[component]
fn DictionaryItemDialog() -> Element {
    let empty_json = "{}";

    rsx! {
        Dialog { class: "dictionary-dialog-root",
            DialogTrigger { class: "dictionary-dialog-trigger dictionary-item-dialog-trigger",
                "打开字典项编辑器"
            }
            DialogContent { class: "dictionary-dialog-content dictionary-item-dialog-content",
                DialogHeader {
                    DialogTitle { "字典项" }
                    DialogDescription { "维护展示名称、稳定编码、原始值和扩展元数据。" }
                }
                form { id: "dictionary-item-form", class: "dictionary-dialog-form",
                    DialogBody { class: "dictionary-dialog-body",
                        h4 { id: "dictionary-item-form-title", class: "dictionary-dialog-mode-title", "新建字典项" }
                        FormGroup { class: "dictionary-form-grid dictionary-form-grid-wide",
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-item-label", "展示名称" }
                                FormContent {
                                    Input {
                                        id: "dictionary-item-label",
                                        class: "az-input",
                                        name: "label",
                                        required: true,
                                        placeholder: "在线"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-item-code", "编码" }
                                FormContent {
                                    Input {
                                        id: "dictionary-item-code",
                                        class: "az-input font-mono",
                                        name: "code",
                                        required: true,
                                        placeholder: "online"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-item-raw-value", "原始值" }
                                FormContent {
                                    Input {
                                        id: "dictionary-item-raw-value",
                                        class: "az-input font-mono",
                                        name: "rawValue",
                                        required: true,
                                        placeholder: "online"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-item-sort", "排序" }
                                FormContent {
                                    Input {
                                        id: "dictionary-item-sort",
                                        class: "az-input",
                                        name: "sortIndex",
                                        r#type: InputType::Number,
                                        value: "0"
                                    }
                                }
                            }
                            FormFieldWrapper {
                                FormLabel { html_for: "dictionary-item-status", "状态" }
                                FormContent {
                                    select { id: "dictionary-item-status", class: "az-input", name: "status",
                                        option { value: "enabled", "启用" }
                                        option { value: "disabled", "停用" }
                                    }
                                }
                            }
                        }
                        FormFieldWrapper {
                            FormLabel { html_for: "dictionary-item-description", "描述" }
                            FormContent {
                                Input {
                                    id: "dictionary-item-description",
                                    class: "az-input",
                                    name: "description",
                                    placeholder: "说明该值的业务语义"
                                }
                            }
                        }
                        FormFieldWrapper {
                            FormLabel { html_for: "dictionary-item-meta", "元数据 JSON" }
                            FormContent {
                                textarea {
                                    id: "dictionary-item-meta",
                                    class: "az-input font-mono",
                                    name: "metaJson",
                                    rows: "3",
                                    value: empty_json
                                }
                            }
                        }
                    }
                    DialogFooter {
                        DialogClose { class: "dictionary-dialog-close dictionary-item-dialog-close", "取消" }
                        Button { button_type: "submit", "保存字典项" }
                    }
                }
            }
        }
    }
}

const DICTIONARY_WORKBENCH_SCRIPT: &str = r#"
(() => {
  const root = document.getElementById('dictionary-workbench');
  if (!root || root.dataset.bound === 'true') return;
  root.dataset.bound = 'true';

  const apiBase = root.dataset.apiBase || '';
  const elements = {
    status: document.getElementById('dictionary-action-status'),
    reload: document.getElementById('dictionary-reload-button'),
    typeSearch: document.getElementById('dictionary-type-search'),
    typeTree: document.getElementById('dictionary-type-tree'),
    typeCreate: document.getElementById('dictionary-type-create-button'),
    typeEdit: document.getElementById('dictionary-type-edit-button'),
    typeDelete: document.getElementById('dictionary-type-delete-button'),
    typeForm: document.getElementById('dictionary-type-form'),
    typeFormTitle: document.getElementById('dictionary-type-form-title'),
    typeDialogTrigger: document.querySelector('.dictionary-type-dialog-trigger'),
    typeDialogContent: document.querySelector('.dictionary-type-dialog-content'),
    typeFormCancel: document.querySelector('.dictionary-type-dialog-close'),
    selectedName: document.getElementById('dictionary-selected-name'),
    selectedStatus: document.getElementById('dictionary-selected-status'),
    selectedDescription: document.getElementById('dictionary-selected-description'),
    selectedCode: document.getElementById('dictionary-selected-code'),
    selectedScope: document.getElementById('dictionary-selected-scope'),
    selectedKind: document.getElementById('dictionary-selected-kind'),
    itemSearch: document.getElementById('dictionary-item-search'),
    itemCreate: document.getElementById('dictionary-item-create-button'),
    itemForm: document.getElementById('dictionary-item-form'),
    itemFormTitle: document.getElementById('dictionary-item-form-title'),
    itemDialogTrigger: document.querySelector('.dictionary-item-dialog-trigger'),
    itemDialogContent: document.querySelector('.dictionary-item-dialog-content'),
    itemFormCancel: document.querySelector('.dictionary-item-dialog-close'),
    itemBody: document.getElementById('dictionary-item-table-body'),
    pageSummary: document.getElementById('dictionary-pagination-summary'),
    pagePrevious: document.getElementById('dictionary-page-previous'),
    pageNext: document.getElementById('dictionary-page-next')
  };
  const state = {
    types: [],
    selectedType: null,
    items: [],
    itemQuery: '',
    offset: 0,
    size: 50,
    total: 0
  };

  function setStatus(message, tone) {
    elements.status.textContent = message;
    elements.status.dataset.tone = tone || 'muted';
  }

  async function request(path, options) {
    const requestOptions = Object.assign({ headers: { Accept: 'application/json' } }, options || {});
    if (requestOptions.body) {
      requestOptions.headers = Object.assign({}, requestOptions.headers, { 'Content-Type': 'application/json' });
    }
    const response = await fetch(apiBase + path, requestOptions);
    const payload = await response.json().catch(() => null);
    if (!response.ok || !payload || payload.code >= 400) {
      throw new Error((payload && payload.msg) || ('请求失败：HTTP ' + response.status));
    }
    return payload.data;
  }

  function escapeHtml(value) {
    return String(value == null ? '' : value).replace(/[&<>"']/g, character => ({
      '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
    })[character]);
  }

  function formatTime(value) {
    if (!value) return '—';
    return new Date(value).toLocaleString('zh-CN', { hour12: false });
  }

  function initialTypeId() {
    const route = new URLSearchParams(window.location.search).get('route') || '';
    const query = route.split('?')[1] || '';
    return new URLSearchParams(query).get('typeId');
  }

  function syncRoute(typeId) {
    const url = new URL(window.location.href);
    const route = '/system/dictionary/note-types?typeId=' + encodeURIComponent(typeId);
    url.searchParams.set('route', route);
    window.history.replaceState(null, '', url);
  }

  function selectedTypeById(id) {
    return state.types.find(item => item.id === id) || null;
  }

  function renderTypeTree() {
    const query = (elements.typeSearch.value || '').trim().toLowerCase();
    const filtered = state.types.filter(item => {
      return !query || item.name.toLowerCase().includes(query) || item.code.toLowerCase().includes(query);
    });
    if (!filtered.length) {
      elements.typeTree.innerHTML = '<p class="dictionary-empty-state">暂无匹配的字典类型。</p>';
      return;
    }
    const groups = new Map();
    filtered.forEach(item => {
      const scope = item.scope || 'system';
      if (!groups.has(scope)) groups.set(scope, []);
      groups.get(scope).push(item);
    });
    elements.typeTree.innerHTML = Array.from(groups.entries()).map(([scope, items]) => {
      const nodes = items.map(item => {
        const selected = state.selectedType && state.selectedType.id === item.id;
        return '<button type="button" class="dictionary-tree-node' + (selected ? ' dictionary-tree-node--selected' : '') + '" data-type-id="' + escapeHtml(item.id) + '">' +
          '<span class="dictionary-tree-node-icon">▤</span>' +
          '<span class="dictionary-tree-node-copy"><strong>' + escapeHtml(item.name) + '</strong><code>' + escapeHtml(item.code) + '</code></span>' +
          '<span class="dictionary-tree-node-count">' + item.itemCount + '</span>' +
          '</button>';
      }).join('');
      return '<section class="dictionary-tree-group"><div class="dictionary-tree-group-title"><span>⌄</span><strong>' + escapeHtml(scope) + '</strong><span>' + items.length + '</span></div>' + nodes + '</section>';
    }).join('');
  }

  function renderSelectedType() {
    const type = state.selectedType;
    const disabled = !type;
    elements.typeEdit.disabled = disabled;
    elements.typeDelete.disabled = disabled;
    elements.itemSearch.disabled = disabled;
    elements.itemCreate.disabled = disabled;
    if (!type) {
      elements.selectedName.textContent = '选择一个字典类型';
      elements.selectedStatus.textContent = '未选择';
      elements.selectedStatus.dataset.status = 'disabled';
      elements.selectedDescription.textContent = '左侧字典类型决定当前表格的数据边界。';
      elements.selectedCode.textContent = '—';
      elements.selectedScope.textContent = 'scope: —';
      elements.selectedKind.textContent = 'raw: —';
      return;
    }
    elements.selectedName.textContent = type.name;
    elements.selectedStatus.textContent = type.status === 'enabled' ? '启用' : '停用';
    elements.selectedStatus.dataset.status = type.status;
    elements.selectedDescription.textContent = type.description || '暂无描述';
    elements.selectedCode.textContent = type.code;
    elements.selectedScope.textContent = 'scope: ' + type.scope;
    elements.selectedKind.textContent = 'raw: ' + type.rawValueKind + (type.openEnum ? ' · open' : ' · closed');
  }

  function renderItems() {
    if (!state.selectedType) {
      elements.itemBody.innerHTML = '<tr><td colspan="7" class="dictionary-empty-state">请先从左侧选择字典类型。</td></tr>';
      renderPagination();
      return;
    }
    if (!state.items.length) {
      elements.itemBody.innerHTML = '<tr><td colspan="7" class="dictionary-empty-state">当前字典还没有条目，点击“新建字典项”开始维护。</td></tr>';
      renderPagination();
      return;
    }
    elements.itemBody.innerHTML = state.items.map(item => {
      const enabled = item.status === 'enabled';
      return '<tr>' +
        '<td class="dictionary-sort-cell">' + item.sortIndex + '</td>' +
        '<td><strong>' + escapeHtml(item.label) + '</strong><small>' + escapeHtml(item.description || '—') + '</small></td>' +
        '<td><code>' + escapeHtml(item.code) + '</code></td>' +
        '<td><code>' + escapeHtml(item.rawValue) + '</code></td>' +
        '<td><span class="dictionary-status-badge" data-status="' + escapeHtml(item.status) + '">' + (enabled ? '启用' : '停用') + '</span></td>' +
        '<td>' + formatTime(item.updatedAtMs) + '</td>' +
        '<td><div class="dictionary-row-actions">' +
          '<button type="button" data-action="item-edit" data-id="' + escapeHtml(item.id) + '">编辑</button>' +
          '<button type="button" data-action="item-toggle" data-id="' + escapeHtml(item.id) + '">' + (enabled ? '停用' : '启用') + '</button>' +
          '<button type="button" data-action="item-delete" data-id="' + escapeHtml(item.id) + '" data-danger="true">删除</button>' +
        '</div></td>' +
      '</tr>';
    }).join('');
    renderPagination();
  }

  function renderPagination() {
    const start = state.total ? state.offset + 1 : 0;
    const end = Math.min(state.offset + state.size, state.total);
    elements.pageSummary.textContent = start + '-' + end + ' / 共 ' + state.total + ' 条';
    elements.pagePrevious.disabled = state.offset === 0;
    elements.pageNext.disabled = state.offset + state.size >= state.total;
  }

  async function loadTypes(preferredId) {
    setStatus('正在读取字典类型...', 'muted');
    state.types = await request('/api/system/dictionary-types');
    const nextId = preferredId || (state.selectedType && state.selectedType.id) || initialTypeId();
    state.selectedType = selectedTypeById(nextId) || state.types[0] || null;
    renderTypeTree();
    renderSelectedType();
    if (state.selectedType) {
      syncRoute(state.selectedType.id);
      await loadItems();
    } else {
      state.items = [];
      state.total = 0;
      renderItems();
      setStatus('暂无字典类型，可以从左上角新建。', 'muted');
    }
  }

  async function loadItems() {
    if (!state.selectedType) return;
    const query = new URLSearchParams({
      dictionaryTypeId: state.selectedType.id,
      o: String(state.offset),
      s: String(state.size)
    });
    if (state.itemQuery) query.set('q', state.itemQuery);
    setStatus('正在读取“' + state.selectedType.name + '”字典项...', 'muted');
    const page = await request('/api/system/dictionary-items?' + query.toString());
    state.items = page.d || [];
    state.total = page.t || 0;
    renderItems();
    setStatus('已加载 ' + state.types.length + ' 个字典类型，当前显示 ' + state.total + ' 个字典项。', 'success');
  }

  async function selectType(id) {
    const selected = selectedTypeById(id);
    if (!selected) return;
    state.selectedType = selected;
    state.offset = 0;
    state.itemQuery = '';
    elements.itemSearch.value = '';
    closeTypeDialog();
    closeItemDialog();
    syncRoute(id);
    renderTypeTree();
    renderSelectedType();
    await loadItems();
  }

  function openTypeDialog(type) {
    const form = elements.typeForm;
    form.dataset.editId = type ? type.id : '';
    elements.typeFormTitle.textContent = type ? '编辑字典类型' : '新建字典类型';
    form.elements.name.value = type ? type.name : '';
    form.elements.code.value = type ? type.code : '';
    form.elements.scope.value = type ? type.scope : 'system';
    form.elements.rawValueKind.value = type ? type.rawValueKind : 'string';
    form.elements.sortIndex.value = type ? type.sortIndex : 0;
    form.elements.status.value = type ? type.status : 'enabled';
    form.elements.description.value = type ? type.description : '';
    form.elements.openEnum.checked = type ? type.openEnum : false;
    elements.typeDialogTrigger.click();
    window.requestAnimationFrame(() => form.elements.name.focus());
  }

  function closeTypeDialog() {
    if (elements.typeDialogContent.dataset.state === 'open') {
      elements.typeFormCancel.click();
    }
    elements.typeForm.dataset.editId = '';
  }

  function openItemDialog(item) {
    if (!state.selectedType) return;
    const form = elements.itemForm;
    form.dataset.editId = item ? item.id : '';
    elements.itemFormTitle.textContent = item ? '编辑字典项' : '新建字典项';
    form.elements.label.value = item ? item.label : '';
    form.elements.code.value = item ? item.code : '';
    form.elements.rawValue.value = item ? item.rawValue : '';
    form.elements.sortIndex.value = item ? item.sortIndex : 0;
    form.elements.status.value = item ? item.status : 'enabled';
    form.elements.description.value = item ? item.description : '';
    form.elements.metaJson.value = item ? item.metaJson : '{}';
    elements.itemDialogTrigger.click();
    window.requestAnimationFrame(() => form.elements.label.focus());
  }

  function closeItemDialog() {
    if (elements.itemDialogContent.dataset.state === 'open') {
      elements.itemFormCancel.click();
    }
    elements.itemForm.dataset.editId = '';
  }

  function typePayload(form) {
    return {
      name: form.elements.name.value,
      code: form.elements.code.value,
      scope: form.elements.scope.value,
      rawValueKind: form.elements.rawValueKind.value,
      sortIndex: Number(form.elements.sortIndex.value || 0),
      status: form.elements.status.value,
      description: form.elements.description.value,
      openEnum: form.elements.openEnum.checked
    };
  }

  function itemPayload(form, item) {
    return {
      dictionaryTypeId: state.selectedType.id,
      label: form ? form.elements.label.value : item.label,
      code: form ? form.elements.code.value : item.code,
      rawValue: form ? form.elements.rawValue.value : item.rawValue,
      sortIndex: form ? Number(form.elements.sortIndex.value || 0) : item.sortIndex,
      status: form ? form.elements.status.value : item.status,
      description: form ? form.elements.description.value : item.description,
      metaJson: form ? form.elements.metaJson.value : item.metaJson
    };
  }

  elements.typeForm.addEventListener('submit', async event => {
    event.preventDefault();
    const editId = elements.typeForm.dataset.editId;
    try {
      const saved = await request(editId ? '/api/system/dictionary-types/' + encodeURIComponent(editId) : '/api/system/dictionary-types', {
        method: editId ? 'PUT' : 'POST',
        body: JSON.stringify(typePayload(elements.typeForm))
      });
      closeTypeDialog();
      setStatus(editId ? '字典类型已更新。' : '字典类型已创建。', 'success');
      await loadTypes(saved.id);
    } catch (error) {
      setStatus('保存字典类型失败：' + error.message, 'error');
    }
  });

  elements.itemForm.addEventListener('submit', async event => {
    event.preventDefault();
    const editId = elements.itemForm.dataset.editId;
    try {
      await request(editId ? '/api/system/dictionary-items/' + encodeURIComponent(editId) : '/api/system/dictionary-items', {
        method: editId ? 'PUT' : 'POST',
        body: JSON.stringify(itemPayload(elements.itemForm, null))
      });
      closeItemDialog();
      setStatus(editId ? '字典项已更新。' : '字典项已创建。', 'success');
      await loadTypes(state.selectedType.id);
    } catch (error) {
      setStatus('保存字典项失败：' + error.message, 'error');
    }
  });

  root.addEventListener('click', async event => {
    const typeNode = event.target.closest('[data-type-id]');
    if (typeNode) {
      await selectType(typeNode.dataset.typeId);
      return;
    }
    const action = event.target.closest('[data-action]');
    if (!action) return;
    const item = state.items.find(value => value.id === action.dataset.id);
    if (!item) return;
    try {
      if (action.dataset.action === 'item-edit') {
        openItemDialog(item);
        return;
      }
      if (action.dataset.action === 'item-toggle') {
        const payload = itemPayload(null, item);
        payload.status = item.status === 'enabled' ? 'disabled' : 'enabled';
        await request('/api/system/dictionary-items/' + encodeURIComponent(item.id), {
          method: 'PUT',
          body: JSON.stringify(payload)
        });
        await loadTypes(state.selectedType.id);
        return;
      }
      if (action.dataset.action === 'item-delete' && window.confirm('确定删除字典项“' + item.label + '”吗？')) {
        await request('/api/system/dictionary-items/' + encodeURIComponent(item.id), { method: 'DELETE' });
        await loadTypes(state.selectedType.id);
      }
    } catch (error) {
      setStatus('字典项操作失败：' + error.message, 'error');
    }
  });

  elements.typeCreate.addEventListener('click', () => openTypeDialog(null));
  elements.typeEdit.addEventListener('click', () => openTypeDialog(state.selectedType));
  elements.typeFormCancel.addEventListener('click', event => event.preventDefault());
  elements.itemCreate.addEventListener('click', () => openItemDialog(null));
  elements.itemFormCancel.addEventListener('click', event => event.preventDefault());
  elements.reload.addEventListener('click', () => loadTypes(state.selectedType && state.selectedType.id).catch(error => setStatus('刷新失败：' + error.message, 'error')));
  elements.typeDelete.addEventListener('click', async () => {
    if (!state.selectedType || !window.confirm('删除字典类型会同时删除全部字典项，确定删除“' + state.selectedType.name + '”吗？')) return;
    try {
      await request('/api/system/dictionary-types/' + encodeURIComponent(state.selectedType.id), { method: 'DELETE' });
      state.selectedType = null;
      await loadTypes(null);
      setStatus('字典类型已删除。', 'success');
    } catch (error) {
      setStatus('删除字典类型失败：' + error.message, 'error');
    }
  });
  elements.typeSearch.addEventListener('input', renderTypeTree);

  let itemSearchTimer = null;
  elements.itemSearch.addEventListener('input', () => {
    window.clearTimeout(itemSearchTimer);
    itemSearchTimer = window.setTimeout(async () => {
      state.itemQuery = elements.itemSearch.value.trim();
      state.offset = 0;
      try {
        await loadItems();
      } catch (error) {
        setStatus('搜索字典项失败：' + error.message, 'error');
      }
    }, 220);
  });
  elements.pagePrevious.addEventListener('click', async () => {
    state.offset = Math.max(0, state.offset - state.size);
    await loadItems();
  });
  elements.pageNext.addEventListener('click', async () => {
    state.offset += state.size;
    await loadItems();
  });

  loadTypes(null).catch(error => {
    setStatus('加载字典管理失败：' + error.message, 'error');
    elements.typeTree.innerHTML = '<p class="dictionary-empty-state">无法连接 PostgreSQL 字典数据源。</p>';
  });
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_page_renders_tree_table_contract() {
        let markup = dioxus_ssr::render_element(dictionary_workbench_page(NativeRenderContext {
            active_route: "/system/dictionary/note-types".to_string(),
            api_base_url: String::new(),
        }));

        // 页面骨架必须明确呈现左树、右表以及真实 CRUD API。
        assert!(markup.contains("字典类型树"));
        assert!(markup.contains("dictionary-item-table-body"));
        assert!(markup.contains("/api/system/dictionary-types"));
        assert!(markup.contains("/api/system/dictionary-items"));
        assert!(markup.contains("system-dictionary-workbench"));
        assert!(markup.contains("data-name=\"DialogContent\""));
        assert!(markup.contains("data-name=\"FormField\""));
    }
}
