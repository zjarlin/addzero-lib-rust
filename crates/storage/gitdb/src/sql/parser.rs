//! SQL parser implementation.
//!
//! Converts SQL strings to our internal AST using sqlparser.

use sqlparser::ast as sp;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser as SqlParser;

use super::ast::*;
use super::error;

/// SQL parser for GitDB.
pub struct Parser;

impl Parser {
    /// Parse a SQL string into a statement.
    pub fn parse(sql: &str) -> anyhow::Result<Statement> {
        let sql = sql.trim();
        if sql.is_empty() {
            let error = error::empty_query();

            return Err(error);
        }

        // Handle special commands not supported by sqlparser
        let upper = sql.to_uppercase();
        if upper == "BEGIN" || upper == "BEGIN TRANSACTION" || upper == "START TRANSACTION" {
            return Ok(Statement::Begin);
        }
        if upper == "COMMIT" {
            return Ok(Statement::Commit);
        }
        if upper == "ROLLBACK" {
            return Ok(Statement::Rollback);
        }
        if upper == "SHOW TABLES" {
            return Ok(Statement::ShowTables);
        }
        if upper.starts_with("DESCRIBE ") || upper.starts_with("DESC ") {
            let table = sql
                .split_whitespace()
                .nth(1)
                .ok_or_else(|| error::missing_clause("table name"))?;
            return Ok(Statement::Describe(table.to_string()));
        }

        let dialect = GenericDialect {};
        let statements = SqlParser::parse_sql(&dialect, sql)?;

        if statements.is_empty() {
            let error = error::empty_query();

            return Err(error);
        }
        if statements.len() > 1 {
            let error = error::multiple_statements();

            return Err(error);
        }

        Self::convert_statement(&statements[0])
    }

    /// Parse multiple SQL statements.
    pub fn parse_multi(sql: &str) -> anyhow::Result<Vec<Statement>> {
        let dialect = GenericDialect {};
        let statements = SqlParser::parse_sql(&dialect, sql)?;
        statements.iter().map(Self::convert_statement).collect()
    }

    fn convert_statement(stmt: &sp::Statement) -> anyhow::Result<Statement> {
        match stmt {
            sp::Statement::CreateTable(create) => Self::convert_create_table(create),
            sp::Statement::Drop {
                object_type,
                names,
                if_exists,
                ..
            } => Self::convert_drop(object_type, names, *if_exists),
            sp::Statement::Query(query) => Self::convert_query(query),
            sp::Statement::Insert(insert) => Self::convert_insert(insert),
            sp::Statement::Update {
                table,
                assignments,
                selection,
                ..
            } => Self::convert_update(table, assignments, selection),
            sp::Statement::Delete(delete) => Self::convert_delete(delete),
            sp::Statement::StartTransaction { .. } => Ok(Statement::Begin),
            sp::Statement::Commit { .. } => Ok(Statement::Commit),
            sp::Statement::Rollback { .. } => Ok(Statement::Rollback),
            other => Err(error::unsupported_statement(format!("{:?}", other))),
        }
    }

    fn convert_create_table(create: &sp::CreateTable) -> anyhow::Result<Statement> {
        let name = Self::extract_table_name(&create.name)?;
        let columns = create
            .columns
            .iter()
            .map(Self::convert_column_def)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Statement::CreateTable(CreateTable {
            name,
            columns,
            if_not_exists: create.if_not_exists,
        }))
    }

    fn convert_column_def(col: &sp::ColumnDef) -> anyhow::Result<ColumnDef> {
        let data_type = Self::convert_data_type(&col.data_type)?;
        let constraints = col
            .options
            .iter()
            .filter_map(|opt| Self::convert_column_option(&opt.option).transpose())
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(ColumnDef {
            name: col.name.value.clone(),
            data_type,
            constraints,
        })
    }

    fn convert_data_type(dt: &sp::DataType) -> anyhow::Result<SqlDataType> {
        match dt {
            sp::DataType::Text
            | sp::DataType::Varchar(_)
            | sp::DataType::CharVarying(_)
            | sp::DataType::Character(_)
            | sp::DataType::Char(_)
            | sp::DataType::String(_) => Ok(SqlDataType::Text),

            sp::DataType::Int(_)
            | sp::DataType::Integer(_)
            | sp::DataType::BigInt(_)
            | sp::DataType::SmallInt(_)
            | sp::DataType::TinyInt(_) => Ok(SqlDataType::Integer),

            sp::DataType::Float(_)
            | sp::DataType::Real
            | sp::DataType::Double(_)
            | sp::DataType::DoublePrecision
            | sp::DataType::Decimal(_)
            | sp::DataType::Numeric(_) => Ok(SqlDataType::Float),

            sp::DataType::Boolean | sp::DataType::Bool => Ok(SqlDataType::Boolean),

            sp::DataType::JSON | sp::DataType::JSONB => Ok(SqlDataType::Json),

            sp::DataType::Timestamp(_, _) | sp::DataType::Datetime(_) | sp::DataType::Date => {
                Ok(SqlDataType::Timestamp)
            }

            sp::DataType::Uuid => Ok(SqlDataType::Uuid),

            other => Err(error::unsupported_data_type(format!("{:?}", other))),
        }
    }

    fn convert_column_option(opt: &sp::ColumnOption) -> anyhow::Result<Option<ColumnConstraint>> {
        match opt {
            sp::ColumnOption::Null => Ok(None), // Nullable by default
            sp::ColumnOption::NotNull => Ok(Some(ColumnConstraint::NotNull)),
            sp::ColumnOption::Unique { is_primary, .. } => {
                if *is_primary {
                    Ok(Some(ColumnConstraint::PrimaryKey))
                } else {
                    Ok(Some(ColumnConstraint::Unique))
                }
            }
            sp::ColumnOption::Default(expr) => {
                let e = Self::convert_expr(expr)?;
                Ok(Some(ColumnConstraint::Default(e)))
            }
            _ => Ok(None), // Ignore other constraints for now
        }
    }

    fn convert_drop(
        object_type: &sp::ObjectType,
        names: &[sp::ObjectName],
        if_exists: bool,
    ) -> anyhow::Result<Statement> {
        match object_type {
            sp::ObjectType::Table => {
                if names.len() != 1 {
                    let error = error::unsupported_statement("DROP multiple tables not supported");

                    return Err(error);
                }
                let name = Self::extract_table_name(&names[0])?;
                Ok(Statement::DropTable(DropTable { name, if_exists }))
            }
            other => Err(error::unsupported_statement(format!(
                "DROP {:?} not supported",
                other
            ))),
        }
    }

    fn convert_query(query: &sp::Query) -> anyhow::Result<Statement> {
        let body = &query.body;
        let select = match body.as_ref() {
            sp::SetExpr::Select(s) => s,
            other => {
                let message = format!("Unsupported query type: {:?}", other);
                let error = error::unsupported_statement(message);

                return Err(error);
            }
        };

        // FROM clause
        let from = if select.from.len() != 1 {
            let error = error::unsupported_statement("Exactly one table in FROM required");

            return Err(error);
        } else {
            Self::extract_from_table(&select.from[0])?
        };

        // SELECT columns
        let columns = Self::convert_projection(&select.projection)?;

        // WHERE clause
        let where_clause = select
            .selection
            .as_ref()
            .map(Self::convert_expr)
            .transpose()?;

        // ORDER BY
        let order_by = query
            .order_by
            .as_ref()
            .map(|ob| Self::extract_order_by_exprs(ob))
            .transpose()?
            .unwrap_or_default();

        // LIMIT
        let limit = query.limit.as_ref().and_then(|l| Self::expr_to_usize(l));

        // OFFSET
        let offset = query
            .offset
            .as_ref()
            .and_then(|o| Self::expr_to_usize(&o.value));

        Ok(Statement::Select(Select {
            columns,
            from,
            where_clause,
            order_by,
            limit,
            offset,
        }))
    }

    fn convert_projection(items: &[sp::SelectItem]) -> anyhow::Result<Vec<SelectColumn>> {
        items
            .iter()
            .map(|item| {
                match item {
                    sp::SelectItem::Wildcard(_) => Ok(SelectColumn::Wildcard),
                    sp::SelectItem::UnnamedExpr(expr) => {
                        if let sp::Expr::Identifier(ident) = expr {
                            Ok(SelectColumn::Column(ident.value.clone()))
                        } else {
                            let e = Self::convert_expr(expr)?;
                            Ok(SelectColumn::Expr {
                                expr: e,
                                alias: None,
                            })
                        }
                    }
                    sp::SelectItem::ExprWithAlias { expr, alias } => {
                        let e = Self::convert_expr(expr)?;
                        Ok(SelectColumn::Expr {
                            expr: e,
                            alias: Some(alias.value.clone()),
                        })
                    }
                    sp::SelectItem::QualifiedWildcard(name, _) => {
                        // table.* - we treat as wildcard for now
                        Err(error::unsupported_expression(format!(
                            "Qualified wildcard: {:?}",
                            name
                        )))
                    }
                }
            })
            .collect()
    }

    fn extract_order_by_exprs(ob: &sp::OrderBy) -> anyhow::Result<Vec<OrderBy>> {
        match &ob.kind {
            sp::OrderByKind::All(_) => Ok(vec![]),
            sp::OrderByKind::Expressions(exprs) => {
                exprs.iter().map(Self::convert_order_by_expr).collect()
            }
        }
    }

    fn convert_order_by_expr(expr: &sp::OrderByExpr) -> anyhow::Result<OrderBy> {
        let column = match &expr.expr {
            sp::Expr::Identifier(id) => id.value.clone(),
            other => {
                let message = format!("ORDER BY expression: {:?}", other);
                let error = error::unsupported_expression(message);

                return Err(error);
            }
        };
        let ascending = expr.options.asc.unwrap_or(true);
        Ok(OrderBy { column, ascending })
    }

    fn convert_insert(insert: &sp::Insert) -> anyhow::Result<Statement> {
        let table = Self::extract_table_from_object(&insert.table)?;

        let columns = if insert.columns.is_empty() {
            None
        } else {
            Some(insert.columns.iter().map(|c| c.value.clone()).collect())
        };

        let values = match insert.source.as_ref().map(|s| s.body.as_ref()) {
            Some(sp::SetExpr::Values(sp::Values { rows, .. })) => rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(Self::convert_expr)
                        .collect::<anyhow::Result<Vec<_>>>()
            })
                .collect::<anyhow::Result<Vec<_>>>()?,
            _ => {
                let error = error::unsupported_statement("INSERT ... SELECT not supported");

                return Err(error);
            }
        };

        Ok(Statement::Insert(Insert {
            table,
            columns,
            values,
        }))
    }

    fn convert_update(
        table: &sp::TableWithJoins,
        assignments: &[sp::Assignment],
        selection: &Option<sp::Expr>,
    ) -> anyhow::Result<Statement> {
        let table_name = Self::extract_from_table(table)?;

        let assigns = assignments
            .iter()
            .map(|a| {
                let column = Self::extract_assignment_target(&a.target)?;
                let value = Self::convert_expr(&a.value)?;
                Ok(Assignment { column, value })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let where_clause = selection.as_ref().map(Self::convert_expr).transpose()?;

        Ok(Statement::Update(Update {
            table: table_name,
            assignments: assigns,
            where_clause,
        }))
    }

    fn extract_assignment_target(target: &sp::AssignmentTarget) -> anyhow::Result<String> {
        match target {
            sp::AssignmentTarget::ColumnName(parts) => {
                // ObjectName has .0 field which is Vec<ObjectNamePart>
                Ok(parts
                    .0
                    .iter()
                    .map(|p| {
                        p.as_ident()
                            .map(|id| id.value.clone())
                            .unwrap_or_else(|| p.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("."))
            }
            sp::AssignmentTarget::Tuple(parts) => {
                // Tuple contains Vec<ObjectName>
                Ok(parts
                    .iter()
                    .flat_map(|obj| obj.0.iter())
                    .map(|p| {
                        p.as_ident()
                            .map(|id| id.value.clone())
                            .unwrap_or_else(|| p.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("."))
            }
        }
    }

    fn convert_delete(delete: &sp::Delete) -> anyhow::Result<Statement> {
        let from = &delete.from;
        let tables = match from {
            sp::FromTable::WithFromKeyword(tables) => tables,
            sp::FromTable::WithoutKeyword(tables) => tables,
        };

        if tables.len() != 1 {
            let error = error::unsupported_statement("DELETE from multiple tables not supported");

            return Err(error);
        }

        let table = Self::extract_from_table(&tables[0])?;
        let where_clause = delete
            .selection
            .as_ref()
            .map(Self::convert_expr)
            .transpose()?;

        Ok(Statement::Delete(Delete {
            table,
            where_clause,
        }))
    }

    fn convert_expr(expr: &sp::Expr) -> anyhow::Result<Expr> {
        match expr {
            sp::Expr::Identifier(id) => Ok(Expr::Column(id.value.clone())),

            sp::Expr::CompoundIdentifier(parts) => {
                // table.column - just use column for now
                let col = parts.last().map(|p| p.value.clone()).ok_or_else(|| {
                    error::invalid_identifier("empty compound identifier")
                })?;
                Ok(Expr::Column(col))
            }

            sp::Expr::Value(v) => Ok(Expr::Literal(Self::convert_value(v)?)),

            sp::Expr::BinaryOp { left, op, right } => {
                let l = Self::convert_expr(left)?;
                let r = Self::convert_expr(right)?;
                let o = Self::convert_binary_op(op)?;
                Ok(Expr::BinaryOp {
                    left: Box::new(l),
                    op: o,
                    right: Box::new(r),
                })
            }

            sp::Expr::UnaryOp { op, expr } => {
                let e = Self::convert_expr(expr)?;
                let o = Self::convert_unary_op(op)?;
                Ok(Expr::UnaryOp {
                    op: o,
                    expr: Box::new(e),
                })
            }

            sp::Expr::IsNull(e) => {
                let inner = Self::convert_expr(e)?;
                Ok(Expr::IsNull {
                    expr: Box::new(inner),
                    negated: false,
                })
            }

            sp::Expr::IsNotNull(e) => {
                let inner = Self::convert_expr(e)?;
                Ok(Expr::IsNull {
                    expr: Box::new(inner),
                    negated: true,
                })
            }

            sp::Expr::InList {
                expr,
                list,
                negated,
            } => {
                let e = Self::convert_expr(expr)?;
                let items = list
                    .iter()
                    .map(Self::convert_expr)
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(Expr::InList {
                    expr: Box::new(e),
                    list: items,
                    negated: *negated,
                })
            }

            sp::Expr::Between {
                expr,
                low,
                high,
                negated,
            } => {
                let e = Self::convert_expr(expr)?;
                let l = Self::convert_expr(low)?;
                let h = Self::convert_expr(high)?;
                Ok(Expr::Between {
                    expr: Box::new(e),
                    low: Box::new(l),
                    high: Box::new(h),
                    negated: *negated,
                })
            }

            sp::Expr::Like {
                expr,
                pattern,
                negated,
                ..
            } => {
                let e = Self::convert_expr(expr)?;
                let pat = Self::extract_string_from_expr(pattern)?;
                Ok(Expr::Like {
                    expr: Box::new(e),
                    pattern: pat,
                    negated: *negated,
                })
            }

            sp::Expr::Function(f) => {
                let name = f.name.to_string();
                let args = match &f.args {
                    sp::FunctionArguments::List(list) => list
                        .args
                        .iter()
                        .filter_map(|arg| match arg {
                            sp::FunctionArg::Unnamed(sp::FunctionArgExpr::Expr(e)) => {
                                Some(Self::convert_expr(e))
                            }
                            _ => None,
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?,
                    _ => vec![],
                };
                Ok(Expr::Function { name, args })
            }

            sp::Expr::Nested(inner) => {
                let e = Self::convert_expr(inner)?;
                Ok(Expr::Nested(Box::new(e)))
            }

            other => Err(error::unsupported_expression(format!("{:?}", other))),
        }
    }

    fn convert_value(v: &sp::ValueWithSpan) -> anyhow::Result<LiteralValue> {
        match &v.value {
            sp::Value::Null => Ok(LiteralValue::Null),
            sp::Value::Boolean(b) => Ok(LiteralValue::Boolean(*b)),
            sp::Value::Number(s, _) => {
                if let Ok(i) = s.parse::<i64>() {
                    Ok(LiteralValue::Integer(i))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(LiteralValue::Float(f))
                } else {
                    Err(error::unsupported_expression(format!(
                        "Invalid number: {}",
                        s
                    )))
                }
            }
            sp::Value::SingleQuotedString(s) => Ok(LiteralValue::String(s.clone())),
            sp::Value::DoubleQuotedString(s) => Ok(LiteralValue::String(s.clone())),
            other => Err(error::unsupported_expression(format!(
                "Unsupported value: {:?}",
                other
            ))),
        }
    }

    fn extract_string_from_expr(expr: &sp::Expr) -> anyhow::Result<String> {
        match expr {
            sp::Expr::Value(v) => match &v.value {
                sp::Value::SingleQuotedString(s) => Ok(s.clone()),
                sp::Value::DoubleQuotedString(s) => Ok(s.clone()),
                _ => Err(error::unsupported_expression("expected string")),
            },
            _ => Err(error::unsupported_expression("expected string literal")),
        }
    }

    fn convert_binary_op(op: &sp::BinaryOperator) -> anyhow::Result<BinaryOperator> {
        match op {
            sp::BinaryOperator::Eq => Ok(BinaryOperator::Eq),
            sp::BinaryOperator::NotEq => Ok(BinaryOperator::NotEq),
            sp::BinaryOperator::Lt => Ok(BinaryOperator::Lt),
            sp::BinaryOperator::LtEq => Ok(BinaryOperator::LtEq),
            sp::BinaryOperator::Gt => Ok(BinaryOperator::Gt),
            sp::BinaryOperator::GtEq => Ok(BinaryOperator::GtEq),
            sp::BinaryOperator::And => Ok(BinaryOperator::And),
            sp::BinaryOperator::Or => Ok(BinaryOperator::Or),
            sp::BinaryOperator::Plus => Ok(BinaryOperator::Plus),
            sp::BinaryOperator::Minus => Ok(BinaryOperator::Minus),
            sp::BinaryOperator::Multiply => Ok(BinaryOperator::Multiply),
            sp::BinaryOperator::Divide => Ok(BinaryOperator::Divide),
            sp::BinaryOperator::Modulo => Ok(BinaryOperator::Modulo),
            sp::BinaryOperator::StringConcat => Ok(BinaryOperator::Concat),
            other => Err(error::unsupported_expression(format!(
                "Unsupported operator: {:?}",
                other
            ))),
        }
    }

    fn convert_unary_op(op: &sp::UnaryOperator) -> anyhow::Result<UnaryOperator> {
        match op {
            sp::UnaryOperator::Not => Ok(UnaryOperator::Not),
            sp::UnaryOperator::Minus => Ok(UnaryOperator::Minus),
            sp::UnaryOperator::Plus => Ok(UnaryOperator::Plus),
            other => Err(error::unsupported_expression(format!(
                "Unsupported unary operator: {:?}",
                other
            ))),
        }
    }

    fn extract_table_name(name: &sp::ObjectName) -> anyhow::Result<String> {
        // Use just the table name, ignoring schema
        name.0
            .last()
            .map(|i| {
                i.as_ident()
                    .map(|id| id.value.clone())
                    .unwrap_or_else(|| i.to_string())
            })
            .ok_or_else(|| error::invalid_identifier("empty table name"))
    }

    fn extract_table_from_object(table: &sp::TableObject) -> anyhow::Result<String> {
        match table {
            sp::TableObject::TableName(name) => Self::extract_table_name(name),
            sp::TableObject::TableFunction(_) => Err(error::unsupported_statement(
                "table function not supported",
            )),
        }
    }

    fn extract_from_table(from: &sp::TableWithJoins) -> anyhow::Result<String> {
        match &from.relation {
            sp::TableFactor::Table { name, .. } => Self::extract_table_name(name),
            other => Err(error::unsupported_statement(format!(
                "Unsupported FROM clause: {:?}",
                other
            ))),
        }
    }

    fn expr_to_usize(expr: &sp::Expr) -> Option<usize> {
        match expr {
            sp::Expr::Value(v) => match &v.value {
                sp::Value::Number(s, _) => s.parse().ok(),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table() {
        let sql = "CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT NOT NULL, age INTEGER)";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.name, "users");
                assert_eq!(ct.columns.len(), 3);
                assert!(!ct.if_not_exists);

                assert_eq!(ct.columns[0].name, "id");
                assert!(
                    ct.columns[0]
                        .constraints
                        .contains(&ColumnConstraint::PrimaryKey)
                );

                assert_eq!(ct.columns[1].name, "name");
                assert!(
                    ct.columns[1]
                        .constraints
                        .contains(&ColumnConstraint::NotNull)
                );
            }
            _ => panic!("Expected CreateTable"),
        }
    }

    #[test]
    fn test_parse_create_table_if_not_exists() {
        let sql = "CREATE TABLE IF NOT EXISTS items (id INTEGER)";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::CreateTable(ct) => {
                assert!(ct.if_not_exists);
            }
            _ => panic!("Expected CreateTable"),
        }
    }

    #[test]
    fn test_parse_drop_table() {
        let sql = "DROP TABLE users";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::DropTable(dt) => {
                assert_eq!(dt.name, "users");
                assert!(!dt.if_exists);
            }
            _ => panic!("Expected DropTable"),
        }
    }

    #[test]
    fn test_parse_select_all() {
        let sql = "SELECT * FROM users";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert_eq!(s.from, "users");
                assert_eq!(s.columns.len(), 1);
                assert!(matches!(s.columns[0], SelectColumn::Wildcard));
                assert!(s.where_clause.is_none());
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_select_columns() {
        let sql = "SELECT id, name FROM users";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 2);
                assert!(matches!(&s.columns[0], SelectColumn::Column(c) if c == "id"));
                assert!(matches!(&s.columns[1], SelectColumn::Column(c) if c == "name"));
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_select_where() {
        let sql = "SELECT * FROM users WHERE age > 21";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert!(s.where_clause.is_some());
                match s.where_clause.unwrap() {
                    Expr::BinaryOp { left, op, right } => {
                        assert!(matches!(*left, Expr::Column(c) if c == "age"));
                        assert_eq!(op, BinaryOperator::Gt);
                        assert!(matches!(*right, Expr::Literal(LiteralValue::Integer(21))));
                    }
                    _ => panic!("Expected BinaryOp"),
                }
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_select_order_limit() {
        let sql = "SELECT * FROM users ORDER BY name DESC LIMIT 10 OFFSET 5";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert_eq!(s.order_by.len(), 1);
                assert_eq!(s.order_by[0].column, "name");
                assert!(!s.order_by[0].ascending);
                assert_eq!(s.limit, Some(10));
                assert_eq!(s.offset, Some(5));
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let sql = "INSERT INTO users (id, name) VALUES ('1', 'Alice')";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.columns, Some(vec!["id".into(), "name".into()]));
                assert_eq!(i.values.len(), 1);
                assert_eq!(i.values[0].len(), 2);
            }
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_parse_update() {
        let sql = "UPDATE users SET name = 'Bob' WHERE id = '1'";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Update(u) => {
                assert_eq!(u.table, "users");
                assert_eq!(u.assignments.len(), 1);
                assert_eq!(u.assignments[0].column, "name");
                assert!(u.where_clause.is_some());
            }
            _ => panic!("Expected Update"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let sql = "DELETE FROM users WHERE id = '1'";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
                assert!(d.where_clause.is_some());
            }
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_parse_transaction_commands() {
        assert!(matches!(Parser::parse("BEGIN").unwrap(), Statement::Begin));
        assert!(matches!(
            Parser::parse("BEGIN TRANSACTION").unwrap(),
            Statement::Begin
        ));
        assert!(matches!(
            Parser::parse("COMMIT").unwrap(),
            Statement::Commit
        ));
        assert!(matches!(
            Parser::parse("ROLLBACK").unwrap(),
            Statement::Rollback
        ));
    }

    #[test]
    fn test_parse_show_tables() {
        assert!(matches!(
            Parser::parse("SHOW TABLES").unwrap(),
            Statement::ShowTables
        ));
    }

    #[test]
    fn test_parse_describe() {
        match Parser::parse("DESCRIBE users").unwrap() {
            Statement::Describe(table) => assert_eq!(table, "users"),
            _ => panic!("Expected Describe"),
        }
    }

    #[test]
    fn test_parse_complex_where() {
        let sql = "SELECT * FROM users WHERE age >= 18 AND (status = 'active' OR role = 'admin')";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert!(s.where_clause.is_some());
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_in_list() {
        let sql = "SELECT * FROM users WHERE status IN ('active', 'pending')";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => match s.where_clause {
                Some(Expr::InList { list, negated, .. }) => {
                    assert_eq!(list.len(), 2);
                    assert!(!negated);
                }
                _ => panic!("Expected InList"),
            },
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_like() {
        let sql = "SELECT * FROM users WHERE name LIKE 'A%'";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => match s.where_clause {
                Some(Expr::Like {
                    pattern, negated, ..
                }) => {
                    assert_eq!(pattern, "A%");
                    assert!(!negated);
                }
                _ => panic!("Expected Like"),
            },
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_between() {
        let sql = "SELECT * FROM users WHERE age BETWEEN 18 AND 65";
        let stmt = Parser::parse(sql).unwrap();

        match stmt {
            Statement::Select(s) => {
                assert!(matches!(
                    s.where_clause,
                    Some(Expr::Between { negated: false, .. })
                ));
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_empty_query() {
        assert_eq!(Parser::parse("").unwrap_err().to_string(), "empty query");
        assert_eq!(Parser::parse("   ").unwrap_err().to_string(), "empty query");
    }
}
