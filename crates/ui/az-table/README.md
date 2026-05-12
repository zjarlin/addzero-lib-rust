# az-table

基于 `az-dioxus-components` 语义表格原语封装的数据驱动表格组件。

它适合两类场景：

- 上层只想喂 `columns + rows`，不想每次手写 `thead/tbody/tr/td`
- 组件库需要统一 `az-*` class contract，同时保留更低层原语给复杂页面自由组合

## 使用

```rust
use az_table::{AzTable, AzTableAlign, AzTableCell, AzTableColumn, AzTableRow};
use dioxus::prelude::*;

let columns = vec![
    AzTableColumn {
        key: "name".into(),
        header: "Name".into(),
        class: None,
        align: AzTableAlign::Start,
    },
    AzTableColumn {
        key: "runtime".into(),
        header: "Runtime".into(),
        class: None,
        align: AzTableAlign::Center,
    },
];

let rows = vec![
    AzTableRow {
        key: "curl".into(),
        cells: vec![AzTableCell::from("az-edge"), AzTableCell::from("curl")],
        class: None,
    },
];

let _ = rsx! {
    AzTable {
        columns: columns,
        rows: rows,
        caption: Some("Edge runtimes".to_string()),
        striped: true,
    }
};
```

## 测试

单独跑这个 crate：

```bash
cargo test -p az-table
```

只跑对外渲染契约测试：

```bash
cargo test -p az-table --test public_api
```
