use crate::model::{CellValue, Range};

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

    if let (Some(range_start), Some(_)) = (start, last_value)
        && rows.len().saturating_sub(range_start) > 1
    {
        ranges.push(Range::new(range_start, column, rows.len() - 1, column));
    }

    ranges
}
