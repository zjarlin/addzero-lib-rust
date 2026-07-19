//! 轻量级 XLSX（Excel）文件读写库。
//!
//! 基于 ZIP 归档和 XML 解析（quick-xml），纯 Rust 实现 `.xlsx` 文件的读取与写入，
//! 不依赖 Microsoft Office 或外部 DLL。
//!
//! # 主要功能
//!
//! - **读取**：从文件路径、字节数片或任意 `Read + Seek` 来源解析 `.xlsx`
//! - **写入**：将工作簿写入文件或任意 `Write + Seek` 目标
//! - **多工作表**：支持单工作簿包含多个工作表
//! - **单元格类型**：字符串、数值、布尔值、空值
//! - **合并单元格**：读写均支持单元格合并区域
//! - **自动垂直合并检测**：`find_vertical_merge_ranges` 可自动识别连续相同值区域
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use az_excel::model::{CellValue, ExcelSheet, ExcelWorkbook};
//! use az_excel::xlsx::{read_xlsx, write_xlsx};
//!
//! # fn main() -> anyhow::Result<()> {
//! let sheet = ExcelSheet::new("Sheet1").with_rows(vec![
//!     vec![CellValue::String("姓名".into()), CellValue::String("年龄".into())],
//!     vec![CellValue::String("张三".into()), CellValue::Number(30.0)],
//! ]);
//! let workbook = ExcelWorkbook::new().with_sheets(vec![sheet]);
//! write_xlsx("/tmp/output.xlsx", &workbook)?;
//!
//! let loaded = read_xlsx("/tmp/output.xlsx")?;
//! assert_eq!(loaded.sheets.len(), 1);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

automod::dir!(pub "src");
