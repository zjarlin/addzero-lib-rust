use crate::model::CellValue;

pub(crate) fn set_cell(cells: &mut Vec<Vec<CellValue>>, row: usize, col: usize, value: CellValue) {
    while cells.len() <= row {
        cells.push(Vec::new());
    }
    while cells[row].len() <= col {
        cells[row].push(CellValue::Empty);
    }
    cells[row][col] = value;
}

#[cfg(test)]
mod tests {
    use crate::model::CellValue;

    use super::set_cell;

    #[test]
    fn set_cell_extends_matrix_automatically() {
        let mut cells = Vec::new();
        set_cell(&mut cells, 1, 2, CellValue::Number(42.0));

        assert_eq!(cells.len(), 2);
        assert_eq!(cells[1].len(), 3);
        assert_eq!(cells[1][2], CellValue::Number(42.0));
    }
}
