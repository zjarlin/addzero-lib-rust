use addzero_excel::*;
use std::io::Cursor;

fn string_cell(value: &str) -> CellValue {
    CellValue::String(value.to_owned())
}

#[test]
fn range_excel_reference_roundtrip_works() {
    let range = Range::new(0, 0, 2, 27);

    assert_eq!(range.to_excel_ref(), "A1:AB3");
    assert_eq!(Range::from_excel_ref("A1:AB3").unwrap(), range);
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
fn export_sheet_config_builds_sheet_with_header() {
    let workbook = ExcelWorkbook::from_export_sheet_configs([ExportSheetConfig::new("Orders")
        .with_headers(["name", "qty"])
        .with_rows(vec![vec![string_cell("apple"), CellValue::Number(3.0)]])]);

    assert_eq!(workbook.sheets.len(), 1);
    assert_eq!(workbook.sheets[0].cells.len(), 2);
    assert_eq!(workbook.sheets[0].cells[0][0], string_cell("name"));
    assert_eq!(workbook.sheets[0].cells[1][1], CellValue::Number(3.0));
}
