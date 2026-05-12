# az-dioxus-components

基于 Dioxus 的可组合 UI 基础组件库，统一使用 `Az*` 组件名与 `az-*` CSS 类名前缀。

当前首批组件：

- `AzCard`
- `AzTable`
- `AzTableCaption`
- `AzTableHead`
- `AzTableBody`
- `AzTableFooter`
- `AzTableRow`
- `AzTableHeaderCell`
- `AzTableCell`

## 使用

```rust
use az_dioxus_components::prelude::*;
use dioxus::prelude::*;

let _ = rsx! {
    AzCard {
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

## 预览 GUI

生成真实可打开的预览页面：

```bash
cargo run -p az-dioxus-components --example preview
```

默认输出到：

```text
target/az-dioxus-components-preview/index.html
```
