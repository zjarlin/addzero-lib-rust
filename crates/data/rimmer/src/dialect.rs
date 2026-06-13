/// SQL 方言。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlDialect {
    /// SQLite 和 sqlx Any 默认问号占位符。
    Sqlite,
    /// PostgreSQL 使用 `$1`、`$2` 形式的占位符。
    Postgres,
}

impl SqlDialect {
    /// 从数据库 URL 推导 SQL 方言。
    pub fn from_database_url(database_url: &str) -> anyhow::Result<Self> {
        if database_url.starts_with("postgres:") || database_url.starts_with("postgresql:") {
            return Ok(Self::Postgres);
        }
        if database_url.starts_with("sqlite:") {
            return Ok(Self::Sqlite);
        }
        anyhow::bail!("unsupported database dialect for url: {database_url}")
    }

    /// 将内部统一 SQL 渲染成目标方言 SQL。
    pub fn render_sql(&self, sql: &str) -> String {
        match self {
            Self::Sqlite => sql.to_string(),
            Self::Postgres => render_postgres_placeholders(sql),
        }
    }
}

fn render_postgres_placeholders(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut index = 1_usize;
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                out.push(ch);
                if in_single_quote && chars.peek() == Some(&'\'') {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else {
                    in_single_quote = !in_single_quote;
                }
            }
            '"' if !in_single_quote => {
                out.push(ch);
                if in_double_quote && chars.peek() == Some(&'"') {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else {
                    in_double_quote = !in_double_quote;
                }
            }
            '?' if !in_single_quote && !in_double_quote => {
                out.push('$');
                out.push_str(&index.to_string());
                index += 1;
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_placeholders_should_be_numbered() {
        let sql = r#"SELECT * FROM "BOOK" WHERE "NAME" = ? AND "EDITION" = ?"#;

        let rendered = SqlDialect::Postgres.render_sql(sql);

        // 断言 PostgreSQL 方言使用递增编号占位符。
        assert_eq!(
            rendered,
            r#"SELECT * FROM "BOOK" WHERE "NAME" = $1 AND "EDITION" = $2"#
        );
    }

    #[test]
    fn postgres_placeholders_should_ignore_quoted_question_mark() {
        let sql = r#"SELECT '?' AS "literal?", "NAME" FROM "BOOK" WHERE "ID" = ?"#;

        let rendered = SqlDialect::Postgres.render_sql(sql);

        // 断言字符串和标识符里的问号不会被误判成参数。
        assert_eq!(
            rendered,
            r#"SELECT '?' AS "literal?", "NAME" FROM "BOOK" WHERE "ID" = $1"#
        );
    }
}
