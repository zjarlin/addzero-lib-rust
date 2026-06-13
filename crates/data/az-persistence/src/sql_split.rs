pub(crate) fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let bytes = sql.as_bytes();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut dollar_tag: Option<String> = None;

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if let Some(tag) = dollar_tag.as_deref() {
            if ch == '$' && sql[index..].starts_with(tag) {
                index += tag.len();
                dollar_tag = None;
            } else {
                index += 1;
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }
            index += 1;
            continue;
        }

        match ch {
            '\'' => {
                in_single_quote = true;
                index += 1;
            }
            '"' => {
                in_double_quote = true;
                index += 1;
            }
            '$' => {
                if let Some(tag) = read_dollar_tag(&sql[index..]) {
                    index += tag.len();
                    dollar_tag = Some(tag);
                } else {
                    index += 1;
                }
            }
            ';' => {
                let statement = sql[start..index].trim();
                if !statement.is_empty() {
                    statements.push(statement.to_string());
                }
                index += 1;
                start = index;
            }
            _ => {
                index += 1;
            }
        }
    }

    let statement = sql[start..].trim();
    if !statement.is_empty() {
        statements.push(statement.to_string());
    }

    statements
}

fn read_dollar_tag(input: &str) -> Option<String> {
    let rest = input.strip_prefix('$')?;
    let end = rest.find('$')?;
    let tag = &rest[..end];
    if tag
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        Some(format!("${tag}$"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::split_sql_statements;

    #[test]
    fn split_sql_statements_keeps_dollar_quoted_blocks_together() {
        let sql = r#"
            DO $$
            BEGIN
                IF TRUE THEN
                    RAISE NOTICE 'contains; semicolon';
                END IF;
            END $$;
            CREATE TABLE IF NOT EXISTS demo (id INTEGER);
        "#;

        let statements = split_sql_statements(sql);

        assert_eq!(
            statements.len(),
            2,
            "the DO block must remain one executable statement"
        );
        assert!(statements[0].contains("RAISE NOTICE"));
        assert!(statements[1].starts_with("CREATE TABLE"));
    }
}
