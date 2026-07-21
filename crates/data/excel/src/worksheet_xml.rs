use std::borrow::Cow;

use crate::cell_reference::encode_cell_reference;
use crate::model::{CellValue, ExcelSheet, ExcelWorkbook};

pub(crate) fn build_content_types_xml(workbook: &ExcelWorkbook) -> String {
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

pub(crate) fn build_workbook_xml(workbook: &ExcelWorkbook) -> String {
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

pub(crate) fn build_workbook_relationships_xml(workbook: &ExcelWorkbook) -> String {
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

pub(crate) fn build_worksheet_xml(sheet: &ExcelSheet) -> String {
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
        CellValue::Number(number) => {
            if number.is_finite() {
                format!(r#"<c r="{reference}"><v>{number}</v></c>"#)
            } else {
                String::new()
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::model::{CellValue, ExcelSheet, ExcelWorkbook};

    use super::{build_content_types_xml, build_worksheet_xml};

    #[test]
    fn build_content_types_xml_contains_required_entries() {
        let workbook = ExcelWorkbook::new()
            .with_sheets(vec![ExcelSheet::new("Sheet1"), ExcelSheet::new("Sheet2")]);
        let xml = build_content_types_xml(&workbook);

        assert!(xml.contains("sheet1.xml"));
        assert!(xml.contains("sheet2.xml"));
        assert!(xml.contains("workbook.xml"));
        assert!(xml.contains("styles.xml"));
    }

    #[test]
    fn build_worksheet_xml_escapes_xml_special_chars() {
        let sheet = ExcelSheet::new("test").with_rows(vec![vec![CellValue::String(
            "<script>alert('xss')</script>".into(),
        )]]);
        let xml = build_worksheet_xml(&sheet);

        assert!(xml.contains("&lt;script&gt;"));
        assert!(!xml.contains("<script>"));
    }
}
