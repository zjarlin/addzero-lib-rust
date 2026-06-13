use az_excel::{
    merge::find_vertical_merge_ranges,
    model::{CellValue, ExcelSheet, ExcelWorkbook, ExportSheetConfig, Range},
    xlsx::{read_xlsx_from_reader, write_xlsx_to_writer},
};
use std::io::Cursor;

fn string_cell(value: &str) -> CellValue {
    CellValue::from(value)
}

#[test]
fn range_excel_reference_roundtrip_works() {
    let range = Range::new(0, 0, 2, 27);

    assert_eq!(range.to_excel_ref(), "A1:AB3");
    assert_eq!(Range::from_excel_ref("A1:AB3").unwrap(), range);
}

#[test]
fn cell_value_is_empty_only_matches_empty_variant() {
    assert!(CellValue::Empty.is_empty());
    assert!(!CellValue::String(String::new()).is_empty());
    assert!(!CellValue::Number(0.0).is_empty());
    assert!(!CellValue::Boolean(false).is_empty());
}

#[test]
fn cell_value_display_formats_numbers_booleans_and_empty_values() {
    assert_eq!(CellValue::Number(3.0).to_string(), "3");
    assert_eq!(CellValue::Number(0.0).to_string(), "0");
    assert_eq!(CellValue::Number(-0.0).to_string(), "-0");
    assert_eq!(CellValue::Number(3.5).to_string(), "3.5");
    assert_eq!(CellValue::Number(-2.5).to_string(), "-2.5");
    assert_eq!(CellValue::Number(f64::NAN).to_string(), "NaN");
    assert_eq!(CellValue::Number(f64::INFINITY).to_string(), "inf");
    assert_eq!(CellValue::Empty.to_string(), "");
    assert_eq!(CellValue::Boolean(true).to_string(), "true");
    assert_eq!(CellValue::Boolean(false).to_string(), "false");
}

#[test]
fn range_from_excel_ref_rejects_invalid_shapes() {
    assert!(Range::from_excel_ref("").is_err());
    assert!(Range::from_excel_ref("A1").is_err());
    assert!(Range::from_excel_ref("A1:B2:C3").is_err());
}

#[test]
fn range_from_excel_ref_accepts_valid_range() {
    let range = Range::from_excel_ref("A1:C3").unwrap();

    assert_eq!(range.start_row, 0);
    assert_eq!(range.start_col, 0);
    assert_eq!(range.end_row, 2);
    assert_eq!(range.end_col, 2);
}

#[test]
fn merge_ranges_are_found_for_repeated_values() {
    let rows = vec![
        vec![string_cell("A"), string_cell("North")],
        vec![string_cell("A"), string_cell("South")],
        vec![string_cell("B"), string_cell("South")],
        vec![string_cell("B"), string_cell("South")],
        vec![CellValue::Empty, string_cell("West")],
    ];

    let ranges = find_vertical_merge_ranges(&rows, &[0, 1]);
    assert_eq!(
        ranges,
        vec![
            Range::new(0, 0, 1, 0),
            Range::new(2, 0, 3, 0),
            Range::new(1, 1, 3, 1),
        ]
    );
}

#[test]
fn workbook_roundtrip_preserves_sheet_data() {
    let workbook = ExcelWorkbook::new().with_sheets(vec![
        ExcelSheet::new("Summary")
            .with_rows(vec![
                vec![
                    string_cell("name"),
                    string_cell("qty"),
                    string_cell("flag"),
                    string_cell(" note "),
                ],
                vec![
                    string_cell("widget"),
                    CellValue::Number(2.0),
                    CellValue::Boolean(true),
                    string_cell(" keep "),
                ],
                vec![
                    string_cell("widget"),
                    CellValue::Number(3.5),
                    CellValue::Boolean(false),
                    string_cell("keep"),
                ],
            ])
            .with_merge_ranges(vec![Range::new(1, 0, 2, 0)]),
    ]);

    let cursor = Cursor::new(Vec::new());
    let cursor = write_xlsx_to_writer(cursor, &workbook).unwrap();
    let roundtrip = read_xlsx_from_reader(Cursor::new(cursor.into_inner())).unwrap();

    assert_eq!(roundtrip, workbook);
}

#[test]
fn writing_empty_workbook_returns_validation_error() {
    let cursor = Cursor::new(Vec::new());
    let error = write_xlsx_to_writer(cursor, &ExcelWorkbook::new()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("workbook must contain at least one sheet")
    );
}

#[test]
fn writing_blank_sheet_name_returns_validation_error() {
    let workbook = ExcelWorkbook::new().with_sheets(vec![ExcelSheet::new("   ")]);
    let cursor = Cursor::new(Vec::new());
    let error = write_xlsx_to_writer(cursor, &workbook).unwrap_err();

    assert!(error.to_string().contains("sheet name cannot be blank"));
}

#[test]
fn workbook_sheet_returns_error_for_out_of_bounds() {
    let workbook = ExcelWorkbook::new().with_sheets(vec![ExcelSheet::new("Sheet1")]);

    assert!(workbook.sheet(0).is_ok());
    assert!(workbook.sheet(1).is_err());
}

#[test]
fn export_sheet_config_builds_sheet_with_header() {
    let workbook = ExcelWorkbook::from_export_sheet_configs([ExportSheetConfig::new("Orders")
        .with_headers(["name", "qty"])
        .with_rows(vec![vec![string_cell("apple"), CellValue::Number(3.0)]])]);

    assert_eq!(workbook.sheets.len(), 1);
    assert_eq!(workbook.sheets[0].cells.len(), 2);
    assert_eq!(workbook.sheets[0].cells[0][0], string_cell("name"));
    assert_eq!(workbook.sheets[0].cells[1][1], CellValue::Number(3.0));
}
