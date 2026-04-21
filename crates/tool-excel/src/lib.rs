#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use quick_xml::Reader;
use quick_xml::encoding::EncodingError;
use quick_xml::escape::EscapeError;
use quick_xml::events::Event;
use quick_xml::events::attributes::AttrError;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub type ExcelResult<T> = Result<T, ExcelError>;

const XLSX_WORKBOOK_PATH: &str = "xl/workbook.xml";
const XLSX_WORKBOOK_RELS_PATH: &str = "xl/_rels/workbook.xml.rels";
const XLSX_SHARED_STRINGS_PATH: &str = "xl/sharedStrings.xml";

#[derive(Debug, Error)]
pub enum ExcelError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("xml encoding error: {0}")]
    Encoding(#[from] EncodingError),
    #[error("xml escape error: {0}")]
    Escape(#[from] EscapeError),
    #[error("xml attribute error: {0}")]
    Attr(#[from] AttrError),
    #[error("workbook must contain at least one sheet")]
    EmptyWorkbook,
    #[error("sheet name cannot be blank")]
    BlankSheetName,
    #[error("zip entry `{0}` was not found")]
    MissingEntry(String),
    #[error("worksheet relationship `{0}` was not found")]
    MissingWorksheetRelationship(String),
    #[error("cell reference `{0}` is invalid")]
    InvalidCellReference(String),
    #[error("range reference `{0}` is invalid")]
    InvalidRangeReference(String),
    #[error("worksheet index {0} is out of bounds")]
    WorksheetIndexOutOfBounds(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Empty,
    String(String),
    Number(f64),
    Boolean(bool),
}

impl CellValue {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn as_display_string(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::String(value) => value.clone(),
            Self::Number(value) => {
                if value.fract() == 0.0 {
                    format!("{value:.0}")
                } else {
                    value.to_string()
                }
            }
            Self::Boolean(value) => value.to_string(),
        }
    }
}

impl Display for CellValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => Ok(()),
            Self::String(value) => f.write_str(value),
            Self::Number(value) => Display::fmt(value, f),
            Self::Boolean(value) => Display::fmt(value, f),
        }
    }
}

impl From<&str> for CellValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for CellValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<f64> for CellValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for CellValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn from_excel_ref(reference: &str) -> ExcelResult<Self> {
        let mut parts = reference.split(':');
        let start = parts
            .next()
            .ok_or_else(|| ExcelError::InvalidRangeReference(reference.to_owned()))?;
        let end = parts
            .next()
            .ok_or_else(|| ExcelError::InvalidRangeReference(reference.to_owned()))?;
        if parts.next().is_some() {
            return Err(ExcelError::InvalidRangeReference(reference.to_owned()));
        }

        let (start_row, start_col) = parse_cell_reference(start)?;
        let (end_row, end_col) = parse_cell_reference(end)?;
        Ok(Self::new(start_row, start_col, end_row, end_col))
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Default)]
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

    pub fn sheet(&self, index: usize) -> ExcelResult<&ExcelSheet> {
        self.sheets
            .get(index)
            .ok_or(ExcelError::WorksheetIndexOutOfBounds(index))
    }
}

#[derive(Debug, Clone, PartialEq)]
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
            cells.push(config.headers.into_iter().map(CellValue::String).collect());
        }
        cells.extend(config.rows);

        Self {
            name: config.name,
            cells,
            merge_ranges: config.merge_ranges,
        }
    }
}

pub fn find_vertical_merge_ranges(rows: &[Vec<CellValue>], columns: &[usize]) -> Vec<Range> {
    columns
        .iter()
        .flat_map(|&column| find_vertical_merge_ranges_for_column(rows, column))
        .collect()
}

pub fn find_vertical_merge_ranges_for_column(rows: &[Vec<CellValue>], column: usize) -> Vec<Range> {
    let mut ranges = Vec::new();
    let mut start: Option<usize> = None;
    let mut last_value: Option<&CellValue> = None;

    for (row_index, row) in rows.iter().enumerate() {
        let current = row.get(column);
        let can_merge = matches!(current, Some(value) if !value.is_empty());

        match (start, last_value, current) {
            (Some(range_start), Some(previous), Some(current_value))
                if can_merge && current_value == previous =>
            {
                if row_index == rows.len() - 1 {
                    ranges.push(Range::new(range_start, column, row_index, column));
                }
            }
            (Some(range_start), Some(previous), Some(current_value))
                if can_merge && current_value != previous =>
            {
                if row_index - range_start > 1 {
                    ranges.push(Range::new(range_start, column, row_index - 1, column));
                }
                start = Some(row_index);
                last_value = current;
            }
            (Some(range_start), Some(_), _) => {
                if row_index - range_start > 1 {
                    ranges.push(Range::new(range_start, column, row_index - 1, column));
                }
                start = if can_merge { Some(row_index) } else { None };
                last_value = current;
            }
            (None, _, Some(current_value)) if !current_value.is_empty() => {
                start = Some(row_index);
                last_value = current;
                if row_index == rows.len() - 1 {
                    start = None;
                    last_value = None;
                }
            }
            _ => {
                start = None;
                last_value = None;
            }
        }
    }

    if let (Some(range_start), Some(_)) = (start, last_value) {
        if rows.len().saturating_sub(range_start) > 1 {
            ranges.push(Range::new(range_start, column, rows.len() - 1, column));
        }
    }

    ranges
}

pub fn write_export_sheet_configs<P>(
    path: P,
    configs: impl IntoIterator<Item = ExportSheetConfig>,
) -> ExcelResult<()>
where
    P: AsRef<Path>,
{
    let workbook = ExcelWorkbook::from_export_sheet_configs(configs);
    write_xlsx(path, &workbook)
}

pub fn write_xlsx<P>(path: P, workbook: &ExcelWorkbook) -> ExcelResult<()>
where
    P: AsRef<Path>,
{
    let file = File::create(path)?;
    write_xlsx_to_writer(file, workbook)?;
    Ok(())
}

pub fn write_xlsx_to_writer<W>(writer: W, workbook: &ExcelWorkbook) -> ExcelResult<W>
where
    W: Write + Seek,
{
    validate_workbook(workbook)?;

    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    write_zip_entry(
        &mut zip,
        "[Content_Types].xml",
        options,
        &build_content_types_xml(workbook),
    )?;
    write_zip_entry(&mut zip, "_rels/.rels", options, ROOT_RELS_XML)?;
    write_zip_entry(
        &mut zip,
        XLSX_WORKBOOK_PATH,
        options,
        &build_workbook_xml(workbook),
    )?;
    write_zip_entry(
        &mut zip,
        XLSX_WORKBOOK_RELS_PATH,
        options,
        &build_workbook_relationships_xml(workbook),
    )?;
    write_zip_entry(&mut zip, "xl/styles.xml", options, STYLES_XML)?;

    for (sheet_index, sheet) in workbook.sheets.iter().enumerate() {
        let path = format!("xl/worksheets/sheet{}.xml", sheet_index + 1);
        write_zip_entry(&mut zip, &path, options, &build_worksheet_xml(sheet))?;
    }

    zip.finish().map_err(ExcelError::from)
}

pub fn read_xlsx<P>(path: P) -> ExcelResult<ExcelWorkbook>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    read_xlsx_from_reader(file)
}

pub fn read_xlsx_from_bytes(bytes: &[u8]) -> ExcelResult<ExcelWorkbook> {
    read_xlsx_from_reader(Cursor::new(bytes))
}

pub fn read_xlsx_from_reader<R>(reader: R) -> ExcelResult<ExcelWorkbook>
where
    R: Read + Seek,
{
    let mut archive = ZipArchive::new(reader)?;
    let workbook_xml = read_required_entry(&mut archive, XLSX_WORKBOOK_PATH)?;
    let relationships_xml = read_required_entry(&mut archive, XLSX_WORKBOOK_RELS_PATH)?;
    let shared_strings = read_optional_entry(&mut archive, XLSX_SHARED_STRINGS_PATH)?
        .map(|xml| parse_shared_strings(&xml))
        .transpose()?
        .unwrap_or_default();

    let sheets = parse_workbook_sheets(&workbook_xml)?;
    let relationships = parse_relationships(&relationships_xml)?;

    let mut workbook = ExcelWorkbook::new();
    for sheet_ref in sheets {
        let target = relationships
            .get(&sheet_ref.relationship_id)
            .ok_or_else(|| {
                ExcelError::MissingWorksheetRelationship(sheet_ref.relationship_id.clone())
            })?;
        let sheet_xml = read_required_entry(&mut archive, target)?;
        let sheet = parse_worksheet(&sheet_ref.name, &sheet_xml, &shared_strings)?;
        workbook.push_sheet(sheet);
    }

    Ok(workbook)
}

fn validate_workbook(workbook: &ExcelWorkbook) -> ExcelResult<()> {
    if workbook.sheets.is_empty() {
        return Err(ExcelError::EmptyWorkbook);
    }

    for sheet in &workbook.sheets {
        if sheet.name.trim().is_empty() {
            return Err(ExcelError::BlankSheetName);
        }
    }

    Ok(())
}

fn write_zip_entry<W>(
    zip: &mut ZipWriter<W>,
    path: &str,
    options: SimpleFileOptions,
    contents: &str,
) -> ExcelResult<()>
where
    W: Write + Seek,
{
    zip.start_file(path, options)?;
    zip.write_all(contents.as_bytes())?;
    Ok(())
}

fn read_required_entry<R>(archive: &mut ZipArchive<R>, path: &str) -> ExcelResult<String>
where
    R: Read + Seek,
{
    let mut file = archive
        .by_name(path)
        .map_err(|_| ExcelError::MissingEntry(path.to_owned()))?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)?;
    Ok(xml)
}

fn read_optional_entry<R>(archive: &mut ZipArchive<R>, path: &str) -> ExcelResult<Option<String>>
where
    R: Read + Seek,
{
    match archive.by_name(path) {
        Ok(mut file) => {
            let mut xml = String::new();
            file.read_to_string(&mut xml)?;
            Ok(Some(xml))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[derive(Debug, Clone)]
struct WorkbookSheetRef {
    name: String,
    relationship_id: String,
}

fn parse_workbook_sheets(xml: &str) -> ExcelResult<Vec<WorkbookSheetRef>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut sheets = Vec::new();
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Empty(element) | Event::Start(element)
                if element.name().as_ref() == b"sheet" =>
            {
                let mut name = None;
                let mut relationship_id = None;
                for attribute in element.attributes() {
                    let attribute = attribute?;
                    match attribute.key.as_ref() {
                        b"name" => {
                            name = Some(
                                attribute
                                    .decode_and_unescape_value(reader.decoder())?
                                    .into_owned(),
                            );
                        }
                        b"r:id" => {
                            relationship_id = Some(
                                attribute
                                    .decode_and_unescape_value(reader.decoder())?
                                    .into_owned(),
                            );
                        }
                        _ => {}
                    }
                }

                if let (Some(name), Some(relationship_id)) = (name, relationship_id) {
                    sheets.push(WorkbookSheetRef {
                        name,
                        relationship_id,
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(sheets)
}

fn parse_relationships(xml: &str) -> ExcelResult<std::collections::HashMap<String, String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();
    let mut relationships = std::collections::HashMap::new();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Empty(element) | Event::Start(element)
                if element.name().as_ref() == b"Relationship" =>
            {
                let mut id = None;
                let mut target = None;
                for attribute in element.attributes() {
                    let attribute = attribute?;
                    match attribute.key.as_ref() {
                        b"Id" => {
                            id = Some(
                                attribute
                                    .decode_and_unescape_value(reader.decoder())?
                                    .into_owned(),
                            );
                        }
                        b"Target" => {
                            let raw = attribute
                                .decode_and_unescape_value(reader.decoder())?
                                .into_owned();
                            target = Some(resolve_zip_path("xl", &raw));
                        }
                        _ => {}
                    }
                }

                if let (Some(id), Some(target)) = (id, target) {
                    relationships.insert(id, target);
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(relationships)
}

fn parse_shared_strings(xml: &str) -> ExcelResult<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buffer = Vec::new();
    let mut strings = Vec::new();
    let mut current = String::new();
    let mut in_si = false;
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.name().as_ref() == b"si" => {
                in_si = true;
                current.clear();
            }
            Event::End(element) if element.name().as_ref() == b"si" => {
                strings.push(current.clone());
                current.clear();
                in_si = false;
            }
            Event::Start(element) if in_si && element.name().as_ref() == b"t" => {
                in_text = true;
            }
            Event::End(element) if element.name().as_ref() == b"t" => {
                in_text = false;
            }
            Event::Text(text) if in_text => {
                current.push_str(text.xml_content()?.as_ref());
            }
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(strings)
}

fn parse_worksheet(name: &str, xml: &str, shared_strings: &[String]) -> ExcelResult<ExcelSheet> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buffer = Vec::new();
    let mut cells = Vec::<Vec<CellValue>>::new();
    let mut merge_ranges = Vec::new();

    let mut current_row = 0usize;
    let mut next_row = 0usize;
    let mut next_col = 0usize;
    let mut current_cell: Option<PendingCell> = None;
    let mut in_value = false;
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.name().as_ref() == b"row" => {
                current_row = row_index_from_attrs(&element, reader.decoder())?.unwrap_or(next_row);
                next_row = current_row + 1;
                next_col = 0;
            }
            Event::Start(element) if element.name().as_ref() == b"c" => {
                current_cell = Some(PendingCell::from_attrs(
                    &element,
                    reader.decoder(),
                    current_row,
                    next_col,
                )?);
                next_col = current_cell.as_ref().map_or(next_col, |cell| cell.col + 1);
            }
            Event::Empty(element) if element.name().as_ref() == b"c" => {
                let cell =
                    PendingCell::from_attrs(&element, reader.decoder(), current_row, next_col)?;
                next_col = cell.col + 1;
                set_cell(&mut cells, cell.row, cell.col, CellValue::Empty);
            }
            Event::Start(element) if element.name().as_ref() == b"v" => {
                in_value = true;
            }
            Event::End(element) if element.name().as_ref() == b"v" => {
                in_value = false;
            }
            Event::Start(element) if element.name().as_ref() == b"t" => {
                in_text = true;
            }
            Event::End(element) if element.name().as_ref() == b"t" => {
                in_text = false;
            }
            Event::Text(text) => {
                if let Some(cell) = current_cell.as_mut() {
                    if in_value {
                        cell.raw_value.push_str(text.xml_content()?.as_ref());
                    } else if in_text {
                        cell.inline_text.push_str(text.xml_content()?.as_ref());
                    }
                }
            }
            Event::End(element) if element.name().as_ref() == b"c" => {
                if let Some(cell) = current_cell.take() {
                    let value = finalize_cell(cell, shared_strings);
                    set_cell(&mut cells, value.0, value.1, value.2);
                }
            }
            Event::Empty(element) if element.name().as_ref() == b"mergeCell" => {
                for attribute in element.attributes() {
                    let attribute = attribute?;
                    if attribute.key.as_ref() == b"ref" {
                        let reference = attribute
                            .decode_and_unescape_value(reader.decoder())?
                            .into_owned();
                        merge_ranges.push(Range::from_excel_ref(&reference)?);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(ExcelSheet {
        name: name.to_owned(),
        cells,
        merge_ranges,
    })
}

#[derive(Debug, Clone)]
struct PendingCell {
    row: usize,
    col: usize,
    cell_type: Option<String>,
    raw_value: String,
    inline_text: String,
}

impl PendingCell {
    fn from_attrs(
        element: &quick_xml::events::BytesStart<'_>,
        decoder: quick_xml::encoding::Decoder,
        default_row: usize,
        default_col: usize,
    ) -> ExcelResult<Self> {
        let mut row = default_row;
        let mut col = default_col;
        let mut cell_type = None;

        for attribute in element.attributes() {
            let attribute = attribute?;
            match attribute.key.as_ref() {
                b"r" => {
                    let reference = attribute.decode_and_unescape_value(decoder)?.into_owned();
                    let (parsed_row, parsed_col) = parse_cell_reference(&reference)?;
                    row = parsed_row;
                    col = parsed_col;
                }
                b"t" => {
                    cell_type = Some(attribute.decode_and_unescape_value(decoder)?.into_owned());
                }
                _ => {}
            }
        }

        Ok(Self {
            row,
            col,
            cell_type,
            raw_value: String::new(),
            inline_text: String::new(),
        })
    }
}

fn finalize_cell(cell: PendingCell, shared_strings: &[String]) -> (usize, usize, CellValue) {
    let value = match cell.cell_type.as_deref() {
        Some("inlineStr") | Some("str") => {
            if cell.inline_text.is_empty() {
                CellValue::String(cell.raw_value)
            } else {
                CellValue::String(cell.inline_text)
            }
        }
        Some("s") => cell
            .raw_value
            .parse::<usize>()
            .ok()
            .and_then(|index| shared_strings.get(index).cloned())
            .map(CellValue::String)
            .unwrap_or(CellValue::Empty),
        Some("b") => CellValue::Boolean(matches!(cell.raw_value.trim(), "1" | "true" | "TRUE")),
        _ => parse_default_cell_value(&cell.raw_value, &cell.inline_text),
    };

    (cell.row, cell.col, value)
}

fn parse_default_cell_value(raw_value: &str, inline_text: &str) -> CellValue {
    if !inline_text.is_empty() {
        return CellValue::String(inline_text.to_owned());
    }

    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return CellValue::Empty;
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        return CellValue::Number(number);
    }

    CellValue::String(trimmed.to_owned())
}

fn row_index_from_attrs(
    element: &quick_xml::events::BytesStart<'_>,
    decoder: quick_xml::encoding::Decoder,
) -> ExcelResult<Option<usize>> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() == b"r" {
            let value = attribute.decode_and_unescape_value(decoder)?.into_owned();
            let row = value
                .parse::<usize>()
                .map_err(|_| ExcelError::InvalidCellReference(value.clone()))?;
            return Ok(Some(row.saturating_sub(1)));
        }
    }
    Ok(None)
}

fn set_cell(cells: &mut Vec<Vec<CellValue>>, row: usize, col: usize, value: CellValue) {
    while cells.len() <= row {
        cells.push(Vec::new());
    }
    while cells[row].len() <= col {
        cells[row].push(CellValue::Empty);
    }
    cells[row][col] = value;
}

fn build_content_types_xml(workbook: &ExcelWorkbook) -> String {
    let sheet_overrides = workbook
        .sheets
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
                index + 1
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
            r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
            r#"<Default Extension="xml" ContentType="application/xml"/>"#,
            r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
            r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#,
            "{}",
            r#"</Types>"#
        ),
        sheet_overrides
    )
}

fn build_workbook_xml(workbook: &ExcelWorkbook) -> String {
    let sheets = workbook
        .sheets
        .iter()
        .enumerate()
        .map(|(index, sheet)| {
            format!(
                r#"<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#,
                escape_xml_attribute(&sheet.name),
                index + 1,
                index + 1
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<bookViews><workbookView xWindow="0" yWindow="0" windowWidth="28800" windowHeight="17280"/></bookViews>"#,
            r#"<sheets>{}</sheets>"#,
            r#"</workbook>"#
        ),
        sheets
    )
}

fn build_workbook_relationships_xml(workbook: &ExcelWorkbook) -> String {
    let mut relationships = workbook
        .sheets
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
                index + 1,
                index + 1
            )
        })
        .collect::<Vec<_>>();

    relationships.push(format!(
        r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        workbook.sheets.len() + 1
    ));

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            "{}",
            r#"</Relationships>"#
        ),
        relationships.join("")
    )
}

fn build_worksheet_xml(sheet: &ExcelSheet) -> String {
    let rows = sheet
        .cells
        .iter()
        .enumerate()
        .map(|(row_index, row)| build_row_xml(row_index, row))
        .collect::<Vec<_>>()
        .join("");

    let merge_xml = if sheet.merge_ranges.is_empty() {
        String::new()
    } else {
        let refs = sheet
            .merge_ranges
            .iter()
            .map(|range| format!(r#"<mergeCell ref="{}"/>"#, range.to_excel_ref()))
            .collect::<Vec<_>>()
            .join("");
        format!(
            r#"<mergeCells count="{}">{}</mergeCells>"#,
            sheet.merge_ranges.len(),
            refs
        )
    };

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
            r#"<sheetData>{}</sheetData>{}"#,
            r#"</worksheet>"#
        ),
        rows, merge_xml
    )
}

fn build_row_xml(row_index: usize, row: &[CellValue]) -> String {
    let cells = row
        .iter()
        .enumerate()
        .filter(|(_, cell)| !cell.is_empty())
        .map(|(col_index, value)| build_cell_xml(row_index, col_index, value))
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<row r="{}">{}</row>"#, row_index + 1, cells)
}

fn build_cell_xml(row_index: usize, col_index: usize, value: &CellValue) -> String {
    let reference = encode_cell_reference(row_index, col_index);
    match value {
        CellValue::Empty => String::new(),
        CellValue::String(text) => {
            let preserve = needs_preserve_space(text);
            let preserve_attr = if preserve {
                r#" xml:space="preserve""#
            } else {
                ""
            };
            format!(
                r#"<c r="{reference}" t="inlineStr"><is><t{preserve_attr}>{}</t></is></c>"#,
                escape_xml_text(text)
            )
        }
        CellValue::Number(number) => format!(r#"<c r="{reference}"><v>{number}</v></c>"#),
        CellValue::Boolean(value) => {
            let flag = usize::from(*value);
            format!(r#"<c r="{reference}" t="b"><v>{flag}</v></c>"#)
        }
    }
}

fn needs_preserve_space(value: &str) -> bool {
    value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace)
}

fn escape_xml_text(value: &str) -> Cow<'_, str> {
    if value.contains(['&', '<', '>', '"', '\'']) {
        Cow::Owned(
            value
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;"),
        )
    } else {
        Cow::Borrowed(value)
    }
}

fn escape_xml_attribute(value: &str) -> Cow<'_, str> {
    escape_xml_text(value)
}

fn resolve_zip_path(base_dir: &str, target: &str) -> String {
    let mut path = if target.starts_with('/') {
        PathBuf::new()
    } else {
        PathBuf::from(base_dir)
    };
    path.push(target);
    normalize_path(&path)
}

fn normalize_path(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

fn encode_cell_reference(row: usize, col: usize) -> String {
    format!("{}{}", encode_column_name(col), row + 1)
}

fn encode_column_name(mut col: usize) -> String {
    let mut bytes = Vec::new();
    loop {
        bytes.push((b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = (col / 26) - 1;
    }
    bytes.iter().rev().collect()
}

fn parse_cell_reference(reference: &str) -> ExcelResult<(usize, usize)> {
    let mut letters = String::new();
    let mut numbers = String::new();

    for character in reference.chars() {
        if character.is_ascii_alphabetic() {
            if !numbers.is_empty() {
                return Err(ExcelError::InvalidCellReference(reference.to_owned()));
            }
            letters.push(character.to_ascii_uppercase());
        } else if character.is_ascii_digit() {
            numbers.push(character);
        } else {
            return Err(ExcelError::InvalidCellReference(reference.to_owned()));
        }
    }

    if letters.is_empty() || numbers.is_empty() {
        return Err(ExcelError::InvalidCellReference(reference.to_owned()));
    }

    let col = decode_column_name(&letters)?;
    let row = numbers
        .parse::<usize>()
        .map_err(|_| ExcelError::InvalidCellReference(reference.to_owned()))?;
    if row == 0 {
        return Err(ExcelError::InvalidCellReference(reference.to_owned()));
    }

    Ok((row - 1, col))
}

fn decode_column_name(column: &str) -> ExcelResult<usize> {
    let mut value = 0usize;
    for character in column.chars() {
        if !character.is_ascii_uppercase() {
            return Err(ExcelError::InvalidCellReference(column.to_owned()));
        }
        value = value * 26 + (character as usize - 'A' as usize + 1);
    }
    Ok(value - 1)
}

const ROOT_RELS_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>"#,
    r#"</Relationships>"#
);

const STYLES_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
    r#"<fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>"#,
    r#"<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>"#,
    r#"<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>"#,
    r#"<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>"#,
    r#"<cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>"#,
    r#"<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>"#,
    r#"</styleSheet>"#
);
