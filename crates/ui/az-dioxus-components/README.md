# az-dioxus-components

基于 Dioxus 的可组合 UI 基础组件库。组件名和组件产出的 CSS class 使用职责名，不使用 `Az*`、`Nb*`、`az-*` 或 `nb-*` 前缀。

当前首批组件：

- `SurfaceCard`
- `GrammarSearchInput`
- `Table`
- `TableCaption`
- `TableHead`
- `TableBody`
- `TableFooter`
- `TableRow`
- `TableHeaderCell`
- `TableCell`
- `DataTable`
- `DataTableColumn`
- `DataTableRow`
- `DataTableCell`

## 使用

```rust,no_run
use az_dioxus_components::prelude::*;
use dioxus::prelude::*;

let _ = rsx! {
    SurfaceCard {
        GrammarSearchInput {
            value: "keyword:addhost; tag:rust,java; def:fun,export,alias".to_string(),
            placeholder: "keyword:addhost; tag:rust,java; def:fun,export,alias",
            fields: vec![
                GrammarSearchField::new("keyword", "关键词"),
                GrammarSearchField::new("tag", "标签"),
                GrammarSearchField::new("def", "定义"),
            ],
            oninput: move |_| {},
        }
        Table {
            striped: true,
            TableCaption { "Runtime nodes" }
            TableHead {
                TableRow {
                    TableHeaderCell { "Name" }
                    TableHeaderCell { "Status" }
                }
            }
            TableBody {
                TableRow {
                    TableCell { "edge-01" }
                    TableCell { "healthy" }
                }
            }
        }
    }
};
```

## 测试

单独跑这个 crate：

```bash
cargo test -p az-dioxus-components
```

只跑表格组件测试：

```bash
cargo test -p az-dioxus-components --test table
cargo test -p az-dioxus-components --test data_table
```

只跑卡片组件测试：

```bash
cargo test -p az-dioxus-components --test surface_card
```

只跑语法式搜索组件测试：

```bash
cargo test -p az-dioxus-components --test grammar_search
```

## 预览 GUI

生成真实可打开的纯表格预览页面，不混入其他壳子。当前示例直接接 `adui-dioxus::Table`：

```bash
cargo run -p az-dioxus-components --example preview
```

默认输出到：

```text
target/az-dioxus-components-preview/index.html
```

按钮二次封装和属性透传示例：

```bash
cargo run -p az-dioxus-components --example button_preview
```

这个示例里 `DemoButton` 内部直接用 `Button { ..props }` 转发全部属性。
