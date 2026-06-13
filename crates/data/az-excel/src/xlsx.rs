use crate::cell_matrix::set_cell;
use crate::cell_reference::parse_cell_reference;
use crate::model::{CellValue, ExcelSheet, ExcelWorkbook, ExportSheetConfig, Range};
use crate::worksheet_xml::{
    build_content_types_xml, build_workbook_relationships_xml, build_workbook_xml,
    build_worksheet_xml,
};
use anyhow::{Context, Result, bail};
use az_derive_aliases::{apply, plain_eq};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const XLSX_WORKBOOK_PATH: &str = "xl/workbook.xml";
const XLSX_WORKBOOK_RELS_PATH: &str = "xl/_rels/workbook.xml.rels";
const XLSX_SHARED_STRINGS_PATH: &str = "xl/sharedStrings.xml";

pub fn write_export_sheet_configs<P>(
    path: P,
    configs: impl IntoIterator<Item = ExportSheetConfig>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let workbook = ExcelWorkbook::from_export_sheet_configs(configs);
    write_xlsx(path, &workbook)
}

pub fn write_xlsx<P>(path: P, workbook: &ExcelWorkbook) -> Result<()>
where
    P: AsRef<Path>,
{
    let file = File::create(path)?;
    write_xlsx_to_writer(file, workbook)?;
    Ok(())
}

pub fn write_xlsx_to_writer<W>(writer: W, workbook: &ExcelWorkbook) -> Result<W>
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

    Ok(zip.finish()?)
}

pub fn read_xlsx<P>(path: P) -> Result<ExcelWorkbook>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    read_xlsx_from_reader(file)
}

pub fn read_xlsx_from_bytes(bytes: &[u8]) -> Result<ExcelWorkbook> {
    read_xlsx_from_reader(Cursor::new(bytes))
}

pub fn read_xlsx_from_reader<R>(reader: R) -> Result<ExcelWorkbook>
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
            .with_context(|| {
                format!(
                    "worksheet relationship `{}` was not found",
                    sheet_ref.relationship_id
                )
            })?;
        let sheet_xml = read_required_entry(&mut archive, target)?;
        let sheet = parse_worksheet(&sheet_ref.name, &sheet_xml, &shared_strings)?;
        workbook.push_sheet(sheet);
    }

    Ok(workbook)
}

fn validate_workbook(workbook: &ExcelWorkbook) -> Result<()> {
    if workbook.sheets.is_empty() {
        bail!("workbook must contain at least one sheet");
    }

    for sheet in &workbook.sheets {
        if sheet.name.trim().is_empty() {
            bail!("sheet name cannot be blank");
        }
    }

    Ok(())
}

fn write_zip_entry<W>(
    zip: &mut ZipWriter<W>,
    path: &str,
    options: SimpleFileOptions,
    contents: &str,
) -> Result<()>
where
    W: Write + Seek,
{
    zip.start_file(path, options)?;
    zip.write_all(contents.as_bytes())?;
    Ok(())
}

fn read_required_entry<R>(archive: &mut ZipArchive<R>, path: &str) -> Result<String>
where
    R: Read + Seek,
{
    let mut file = archive
        .by_name(path)
        .with_context(|| format!("zip entry `{path}` was not found"))?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)?;
    Ok(xml)
}

fn read_optional_entry<R>(archive: &mut ZipArchive<R>, path: &str) -> Result<Option<String>>
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

#[apply(plain_eq)]
struct WorkbookSheetRef {
    name: String,
    relationship_id: String,
}

fn parse_workbook_sheets(xml: &str) -> Result<Vec<WorkbookSheetRef>> {
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

fn parse_relationships(xml: &str) -> Result<std::collections::HashMap<String, String>> {
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

fn parse_shared_strings(xml: &str) -> Result<Vec<String>> {
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

fn parse_worksheet(name: &str, xml: &str, shared_strings: &[String]) -> Result<ExcelSheet> {
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

#[apply(plain_eq)]
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
    ) -> Result<Self> {
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
        Some("inlineStr" | "str") => {
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
            .map(CellValue::from)
            .unwrap_or(CellValue::Empty),
        Some("b") => CellValue::Boolean(matches!(cell.raw_value.trim(), "1" | "true" | "TRUE")),
        _ => parse_default_cell_value(&cell.raw_value, &cell.inline_text),
    };

    (cell.row, cell.col, value)
}

fn parse_default_cell_value(raw_value: &str, inline_text: &str) -> CellValue {
    if !inline_text.is_empty() {
        return CellValue::from(inline_text);
    }

    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return CellValue::Empty;
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.is_finite() {
            return CellValue::Number(number);
        }
        return CellValue::from(trimmed);
    }

    CellValue::from(trimmed)
}

fn row_index_from_attrs(
    element: &quick_xml::events::BytesStart<'_>,
    decoder: quick_xml::encoding::Decoder,
) -> Result<Option<usize>> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() == b"r" {
            let value = attribute.decode_and_unescape_value(decoder)?.into_owned();
            let row = value
                .parse::<usize>()
                .with_context(|| format!("cell reference `{value}` is invalid"))?;
            return Ok(Some(row.saturating_sub(1)));
        }
    }
    Ok(None)
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
