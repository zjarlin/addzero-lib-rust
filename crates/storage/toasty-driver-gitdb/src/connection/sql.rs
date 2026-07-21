use super::value::to_json_value;
use anyhow::anyhow;
use az_str::transformation::quote_sql_string_literal;
use toasty_core::driver::operation::TypedValue;

pub(crate) fn inline_indexed_params(sql: &str, params: &[TypedValue]) -> anyhow::Result<String> {
    let literals = params
        .iter()
        .map(param_to_sql_literal)
        .collect::<Result<Vec<_>, _>>()?;

    let mut out =
        String::with_capacity(sql.len() + literals.iter().map(String::len).sum::<usize>());
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            out.push(ch);
            if in_single_quote && chars.peek() == Some(&'\'') {
                if let Some(quote) = chars.next() {
                    out.push(quote);
                }
            } else {
                in_single_quote = !in_single_quote;
            }
            continue;
        }

        if !in_single_quote && ch == '?' {
            let mut digits = String::new();
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() {
                    digits.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }

            if digits.is_empty() {
                out.push(ch);
                continue;
            }

            let index = digits
                .parse::<usize>()
                .map_err(|_| anyhow!("invalid result: invalid placeholder ?{digits}"))?;
            let literal = literals.get(index.saturating_sub(1)).ok_or_else(|| {
                anyhow!("invalid result: missing parameter for placeholder ?{index}")
            })?;
            out.push_str(literal);
            continue;
        }

        out.push(ch);
    }

    Ok(out)
}

fn param_to_sql_literal(param: &TypedValue) -> anyhow::Result<String> {
    let json = to_json_value(&param.value)?;
    Ok(json_to_sql_literal(&json))
}

fn json_to_sql_literal(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(true) => "TRUE".to_string(),
        serde_json::Value::Bool(false) => "FALSE".to_string(),
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::String(text) => quote_sql_string_literal(text),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            quote_sql_string_literal(&value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toasty_core::driver::operation::TypedValue;
    use toasty_core::schema::db;
    use toasty_core::stmt::Value;

    #[test]
    fn replace_indexed_placeholders() {
        let params = vec![
            TypedValue {
                value: Value::String("Alice".into()),
                ty: db::Type::Text,
            },
            TypedValue {
                value: Value::I64(7),
                ty: db::Type::Integer(8),
            },
        ];

        let sql = "INSERT INTO users (name, age) VALUES (?1, ?2);";
        let inlined = inline_indexed_params(sql, &params).unwrap();
        assert_eq!(
            inlined,
            "INSERT INTO users (name, age) VALUES ('Alice', 7);"
        );
    }

    #[test]
    fn ignore_placeholders_inside_strings() {
        let params = vec![TypedValue {
            value: Value::String("done".into()),
            ty: db::Type::Text,
        }];

        let sql = "SELECT '?1' AS literal, ?1 AS value;";
        let inlined = inline_indexed_params(sql, &params).unwrap();
        assert_eq!(inlined, "SELECT '?1' AS literal, 'done' AS value;");
    }
}
