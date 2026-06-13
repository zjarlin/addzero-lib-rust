# az-excel

轻量级 XLSX（Excel）文件读写库，纯 Rust 实现，不依赖 Microsoft Office。

## 功能

- 从文件、字节数片或任意 `Read + Seek` 来源读取 `.xlsx`
- 将工作簿写入文件或任意 `Write + Seek` 目标
- 支持多工作表（工作簿包含多个 `ExcelSheet`）
- 单元格类型：字符串、浮点数、布尔值、空值
- 读写均支持合并单元格（merge ranges）
- 自动检测垂直方向连续相同值区域（`find_vertical_merge_ranges`）
- 基于 `ExportSheetConfig` 的便捷导出入口

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-excel = { path = "../az-excel" }       # workspace 内部引用
# 或发布后：
# az-excel = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_excel::model::{CellValue, ExcelSheet, ExcelWorkbook, ExportSheetConfig};
use az_excel::xlsx::{read_xlsx, write_xlsx};

// --- 写入 ---
let sheet = ExcelSheet::new("数据").with_rows(vec![
    vec![CellValue::String("姓名".into()), CellValue::String("年龄".into())],
    vec![CellValue::String("张三".into()), CellValue::Number(30.0)],
    vec![CellValue::String("李四".into()), CellValue::Number(25.0)],
]);

let workbook = ExcelWorkbook::new().with_sheets(vec![sheet]);
write_xlsx("/tmp/report.xlsx", &workbook).unwrap();

// --- 读取 ---
let wb = read_xlsx("/tmp/report.xlsx").unwrap();
let first_sheet = wb.sheet(0).unwrap();
assert_eq!(first_sheet.name, "数据");
```

### 从 ExportSheetConfig 快速导出

```rust
use az_excel::model::{CellValue, ExportSheetConfig};
use az_excel::xlsx::write_export_sheet_configs;

let config = ExportSheetConfig::new("Sheet1")
    .with_headers(["列A", "列B"])
    .with_rows(vec![
        vec![CellValue::Number(1.0), CellValue::String("hello".into())],
    ]);

write_export_sheet_configs("/tmp/out.xlsx", [config]).unwrap();
```

## 依赖的 crates

- `quick-xml` - XML 流式读写
- `zip` - ZIP 归档读写（XLSX 文件格式基础）
- `anyhow` - 错误上下文与统一 `Result`
