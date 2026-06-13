use crate::cell_reference::{encode_cell_reference, parse_cell_reference};
use anyhow::{Context, Result, bail};
use az_derive_aliases::{
    apply, from_display, plain_copy_eq, plain_default_partial_eq, plain_partial_eq,
};

#[apply(from_display)]
pub enum CellValue {
    #[display("")]
    Empty,
    #[from(&str)]
    #[from(String)]
    #[display("{_0}")]
    String(String),
    #[from(ignore)]
    #[display("{_0}")]
    Number(f64),
    #[from(bool)]
    #[display("{_0}")]
    Boolean(bool),
}

impl CellValue {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl From<f64> for CellValue {
    fn from(value: f64) -> Self {
        if value.is_finite() {
            Self::Number(value)
        } else {
            Self::Empty
        }
    }
}

#[apply(plain_copy_eq)]
pub struct Range {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl Range {
    pub fn new(start_row: usize, start_col: usize, end_row: usize, end_col: usize) -> Self {
        Self {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }

    pub fn to_excel_ref(&self) -> String {
        format!(
            "{}:{}",
            encode_cell_reference(self.start_row, self.start_col),
            encode_cell_reference(self.end_row, self.end_col)
        )
    }

    pub fn from_excel_ref(reference: &str) -> Result<Self> {
        let mut parts = reference.split(':');
        let start = parts
            .next()
            .with_context(|| format!("range reference `{reference}` is invalid"))?;
        let end = parts
            .next()
            .with_context(|| format!("range reference `{reference}` is invalid"))?;
        if parts.next().is_some() {
            bail!("range reference `{reference}` is invalid");
        }

        let (start_row, start_col) = parse_cell_reference(start)?;
        let (end_row, end_col) = parse_cell_reference(end)?;
        Ok(Self::new(start_row, start_col, end_row, end_col))
    }
}

#[apply(plain_partial_eq)]
pub struct ExcelSheet {
    pub name: String,
    pub cells: Vec<Vec<CellValue>>,
    pub merge_ranges: Vec<Range>,
}

impl ExcelSheet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cells: Vec::new(),
            merge_ranges: Vec::new(),
        }
    }

    pub fn with_rows(mut self, rows: impl Into<Vec<Vec<CellValue>>>) -> Self {
        self.cells = rows.into();
        self
    }

    pub fn with_merge_ranges(mut self, merge_ranges: impl Into<Vec<Range>>) -> Self {
        self.merge_ranges = merge_ranges.into();
        self
    }

    pub fn push_row(&mut self, row: impl Into<Vec<CellValue>>) -> &mut Self {
        self.cells.push(row.into());
        self
    }

    pub fn push_merge_range(&mut self, range: Range) -> &mut Self {
        self.merge_ranges.push(range);
        self
    }
}

#[apply(plain_default_partial_eq)]
pub struct ExcelWorkbook {
    pub sheets: Vec<ExcelSheet>,
}

impl ExcelWorkbook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sheets(mut self, sheets: impl Into<Vec<ExcelSheet>>) -> Self {
        self.sheets = sheets.into();
        self
    }

    pub fn push_sheet(&mut self, sheet: ExcelSheet) -> &mut Self {
        self.sheets.push(sheet);
        self
    }

    pub fn from_export_sheet_configs(configs: impl IntoIterator<Item = ExportSheetConfig>) -> Self {
        let sheets = configs.into_iter().map(ExcelSheet::from).collect();
        Self { sheets }
    }

    pub fn sheet(&self, index: usize) -> Result<&ExcelSheet> {
        self.sheets
            .get(index)
            .with_context(|| format!("worksheet index {index} is out of bounds"))
    }
}

#[apply(plain_partial_eq)]
pub struct ExportSheetConfig {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<CellValue>>,
    pub merge_ranges: Vec<Range>,
}

impl ExportSheetConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            headers: Vec::new(),
            rows: Vec::new(),
            merge_ranges: Vec::new(),
        }
    }

    pub fn with_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_rows(mut self, rows: impl Into<Vec<Vec<CellValue>>>) -> Self {
        self.rows = rows.into();
        self
    }

    pub fn with_merge_ranges(mut self, merge_ranges: impl Into<Vec<Range>>) -> Self {
        self.merge_ranges = merge_ranges.into();
        self
    }
}

impl From<ExportSheetConfig> for ExcelSheet {
    fn from(config: ExportSheetConfig) -> Self {
        let mut cells =
            Vec::with_capacity(config.rows.len() + usize::from(!config.headers.is_empty()));
        if !config.headers.is_empty() {
            cells.push(config.headers.into_iter().map(CellValue::from).collect());
        }
        cells.extend(config.rows);

        Self {
            name: config.name,
            cells,
            merge_ranges: config.merge_ranges,
        }
    }
}
