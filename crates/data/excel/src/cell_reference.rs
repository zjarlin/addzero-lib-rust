use anyhow::{Context, Result, bail};

pub(crate) fn encode_cell_reference(row: usize, col: usize) -> String {
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

pub(crate) fn parse_cell_reference(reference: &str) -> Result<(usize, usize)> {
    let mut letters = String::new();
    let mut numbers = String::new();

    for character in reference.chars() {
        if character.is_ascii_alphabetic() {
            if !numbers.is_empty() {
                bail!("cell reference `{reference}` is invalid");
            }
            letters.push(character.to_ascii_uppercase());
        } else if character.is_ascii_digit() {
            numbers.push(character);
        } else {
            bail!("cell reference `{reference}` is invalid");
        }
    }

    if letters.is_empty() || numbers.is_empty() {
        bail!("cell reference `{reference}` is invalid");
    }

    let col = decode_column_name(&letters)?;
    let row = numbers
        .parse::<usize>()
        .with_context(|| format!("cell reference `{reference}` is invalid"))?;
    if row == 0 {
        bail!("cell reference `{reference}` is invalid");
    }

    Ok((row - 1, col))
}

fn decode_column_name(column: &str) -> Result<usize> {
    let mut value = 0usize;
    for character in column.chars() {
        if !character.is_ascii_uppercase() {
            bail!("cell reference `{column}` is invalid");
        }
        value = value * 26 + (character as usize - 'A' as usize + 1);
    }
    Ok(value - 1)
}

#[cfg(test)]
mod tests {
    use super::{encode_cell_reference, parse_cell_reference};

    #[test]
    fn encode_cell_reference_single_letter() {
        assert_eq!(encode_cell_reference(0, 0), "A1");
        assert_eq!(encode_cell_reference(0, 25), "Z1");
        assert_eq!(encode_cell_reference(4, 0), "A5");
    }

    #[test]
    fn encode_cell_reference_multi_letter() {
        assert_eq!(encode_cell_reference(0, 26), "AA1");
        assert_eq!(encode_cell_reference(0, 51), "AZ1");
        assert_eq!(encode_cell_reference(0, 702), "AAA1");
    }

    #[test]
    fn parse_cell_reference_roundtrip() {
        let cases = [(0, 0), (0, 25), (0, 26), (0, 51), (4, 2), (0, 702)];
        for (row, col) in cases {
            let reference = encode_cell_reference(row, col);
            let (parsed_row, parsed_col) = parse_cell_reference(&reference).unwrap();
            assert_eq!(
                (parsed_row, parsed_col),
                (row, col),
                "roundtrip failed for ({row}, {col}) -> {reference}"
            );
        }
    }

    #[test]
    fn parse_cell_reference_rejects_invalid_input() {
        assert!(parse_cell_reference("").is_err());
        assert!(parse_cell_reference("123").is_err());
        assert!(parse_cell_reference("ABC").is_err());
        assert!(parse_cell_reference("A0").is_err());
    }
}
