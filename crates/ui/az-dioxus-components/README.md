# az-dioxus-components

基于 Dioxus 的可组合 UI 基础组件库，统一使用 `Az*` 组件名与 `az-*` CSS 类名前缀。

当前首批组件：

- `AzCard`
- `AzGrammarSearchInput`
- `AzTable`
- `AzTableCaption`
- `AzTableHead`
- `AzTableBody`
- `AzTableFooter`
- `AzTableRow`
- `AzTableHeaderCell`
- `AzTableCell`

## 使用

```rust,no_run
use az_dioxus_components::prelude::*;
use dioxus::prelude::*;

let _ = rsx! {
    AzCard {
        AzGrammarSearchInput {
            value: "keyword:addhost; tag:rust,java; def:fun,export,alias".to_string(),
            placeholder: "keyword:addhost; tag:rust,java; def:fun,export,alias",
            fields: vec![
                AzGrammarSearchField::new("keyword", "关键词"),
                AzGrammarSearchField::new("tag", "标签"),
                AzGrammarSearchField::new("def", "定义"),
            ],
            oninput: move |_| {},
        }
        AzTable {
            striped: true,
            AzTableCaption { "Runtime nodes" }
            AzTableHead {
                AzTableRow {
                    AzTableHeaderCell { "Name" }
                    AzTableHeaderCell { "Status" }
                }
            }
            AzTableBody {
                AzTableRow {
                    AzTableCell { "edge-01" }
                    AzTableCell { "healthy" }
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
cargo test -p az-dioxus-components --test az_table
```

只跑卡片组件测试：

```bash
cargo test -p az-dioxus-components --test az_card
```

只跑语法式搜索组件测试：

```bash
cargo test -p az-dioxus-components --test az_grammar_search
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
