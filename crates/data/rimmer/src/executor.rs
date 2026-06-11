use serde_json::{Map, Number, Value};
use sqlx::any::{AnyPoolOptions, AnyRow, install_default_drivers};
use sqlx::{AnyPool, Column, Row};

use crate::dialect::SqlDialect;
use crate::draft::Draft;
use crate::error::{OrmError, OrmResult};
use crate::expression::{IntoPredicate, Order};
use crate::metadata::Table;
use crate::query::{QueryBuilder, QueryBuilderExt, QueryPlan, Selection, child_column_alias};
use crate::save::{SaveCommand, SaveMode, SavePlan};
use crate::value::ScalarValue;

/// 持有 sqlx 连接池的 Jimmer 风格客户端。
#[derive(Clone, Debug)]
pub struct SqlxJimmerClient {
    pool: AnyPool,
    dialect: SqlDialect,
}

impl SqlxJimmerClient {
    /// 使用数据库 URL 创建客户端。
    pub async fn connect(database_url: &str) -> OrmResult<Self> {
        install_default_drivers();
        let dialect = SqlDialect::from_database_url(database_url)?;
        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(database_error)?;
        Ok(Self { pool, dialect })
    }

    /// 使用已有 AnyPool 创建客户端。
    pub fn from_pool(pool: AnyPool) -> Self {
        Self {
            pool,
            dialect: SqlDialect::Sqlite,
        }
    }

    /// 使用已有 AnyPool 和明确方言创建客户端。
    pub fn from_pool_with_dialect(pool: AnyPool, dialect: SqlDialect) -> Self {
        Self { pool, dialect }
    }

    /// 返回底层连接池。
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    /// 返回当前客户端使用的 SQL 方言。
    pub fn dialect(&self) -> SqlDialect {
        self.dialect
    }

    /// 创建可执行查询构建器。
    pub fn create_query<E>(&self, table: Table<E>) -> SqlxQueryBuilder<E> {
        SqlxQueryBuilder::new(self.pool.clone(), self.dialect, table)
    }

    /// 创建可执行保存命令。
    pub fn save<E>(&self, draft: Draft<E>) -> SqlxSaveCommand<E> {
        SqlxSaveCommand::new(self.pool.clone(), self.dialect, draft)
    }
}

/// 持有连接池的 SELECT 查询构建器。
pub struct SqlxQueryBuilder<E> {
    pool: AnyPool,
    dialect: SqlDialect,
    inner: QueryBuilder<E>,
}

impl<E> SqlxQueryBuilder<E> {
    /// 创建可执行查询构建器。
    pub fn new(pool: AnyPool, dialect: SqlDialect, table: Table<E>) -> Self {
        Self {
            pool,
            dialect,
            inner: QueryBuilder::new(table),
        }
    }

    /// 追加动态谓词。
    pub fn where_(mut self, predicate: impl IntoPredicate) -> Self {
        self.inner = self.inner.where_(predicate);
        self
    }

    /// 追加排序。
    pub fn order_by(mut self, order: Order) -> Self {
        self.inner = self.inner.order_by(order);
        self
    }

    /// 设置分页 limit。
    pub fn limit(mut self, value: usize) -> Self {
        self.inner = self.inner.limit(value);
        self
    }

    /// 设置分页 offset。
    pub fn offset(mut self, value: usize) -> Self {
        self.inner = self.inner.offset(value);
        self
    }

    /// 设置查询选择。
    pub fn select(mut self, selection: Selection<E>) -> Self {
        self.inner = self.inner.select(selection);
        self
    }

    /// 构建 SQL plan。
    pub fn build(self) -> OrmResult<QueryPlan> {
        self.inner.build()
    }

    /// 执行查询并返回 JSON 行。
    pub async fn execute_json(self) -> OrmResult<JsonQueryResult> {
        let plan = self.inner.build()?;
        execute_query_plan_json(&self.pool, self.dialect, plan).await
    }
}

/// 持有连接池的保存命令。
pub struct SqlxSaveCommand<E> {
    pool: AnyPool,
    dialect: SqlDialect,
    inner: SaveCommand<E>,
}

impl<E> SqlxSaveCommand<E> {
    /// 创建可执行保存命令。
    pub fn new(pool: AnyPool, dialect: SqlDialect, draft: Draft<E>) -> Self {
        Self {
            pool,
            dialect,
            inner: SaveCommand::new(draft),
        }
    }

    /// 设置保存模式。
    pub fn set_mode(mut self, mode: SaveMode) -> Self {
        self.inner = self.inner.set_mode(mode);
        self
    }

    /// 构建 SQL plan。
    pub fn build(self) -> OrmResult<SavePlan> {
        self.inner.build()
    }

    /// 执行保存命令。
    pub async fn execute(self) -> OrmResult<SaveExecution> {
        let plan = self.inner.build()?;
        execute_save_plan(&self.pool, self.dialect, plan).await
    }
}

/// JSON 查询执行结果。
#[derive(Clone, Debug, PartialEq)]
pub struct JsonQueryResult {
    /// 查询使用的 Fetcher 形状。
    pub fetch_shape: Option<crate::FetchShape>,
    /// 查询返回的 JSON 行。
    pub rows: Vec<Value>,
}

/// 保存执行结果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveExecution {
    /// 数据库返回的影响行数。
    pub rows_affected: u64,
    /// 实际保存模式。
    pub mode: SaveMode,
}

impl QueryPlan {
    /// 使用 sqlx AnyPool 执行查询并返回 JSON 行。
    pub async fn execute_json(self, pool: &AnyPool) -> OrmResult<JsonQueryResult> {
        execute_query_plan_json(pool, SqlDialect::Sqlite, self).await
    }

    /// 使用 sqlx AnyPool 和明确方言执行查询并返回 JSON 行。
    pub async fn execute_json_with_dialect(
        self,
        pool: &AnyPool,
        dialect: SqlDialect,
    ) -> OrmResult<JsonQueryResult> {
        execute_query_plan_json(pool, dialect, self).await
    }
}

impl SavePlan {
    /// 使用 sqlx AnyPool 执行保存命令。
    pub async fn execute(self, pool: &AnyPool) -> OrmResult<SaveExecution> {
        execute_save_plan(pool, SqlDialect::Sqlite, self).await
    }

    /// 使用 sqlx AnyPool 和明确方言执行保存命令。
    pub async fn execute_with_dialect(
        self,
        pool: &AnyPool,
        dialect: SqlDialect,
    ) -> OrmResult<SaveExecution> {
        execute_save_plan(pool, dialect, self).await
    }
}

async fn execute_query_plan_json(
    pool: &AnyPool,
    dialect: SqlDialect,
    plan: QueryPlan,
) -> OrmResult<JsonQueryResult> {
    let sql = dialect.render_sql(&plan.sql);
    let mut query = sqlx::query(&sql);
    for param in &plan.params {
        query = bind_scalar(query, param);
    }
    let rows = query.fetch_all(pool).await.map_err(database_error)?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let value = match plan.fetch_shape.as_ref() {
            Some(shape) if shape_has_relation(shape) => row_to_nested_json(&row, shape)?,
            _ => row_to_json(&row)?,
        };
        values.push(value);
    }
    if let Some(shape) = plan.fetch_shape.as_ref() {
        load_collection_relations(pool, dialect, shape, &mut values).await?;
    }
    Ok(JsonQueryResult {
        fetch_shape: plan.fetch_shape,
        rows: values,
    })
}

async fn execute_save_plan(
    pool: &AnyPool,
    dialect: SqlDialect,
    plan: SavePlan,
) -> OrmResult<SaveExecution> {
    let root_mode = plan.mode;
    let mut rows_affected = 0;
    let mut stack = vec![plan];
    while let Some(mut plan) = stack.pop() {
        let children = std::mem::take(&mut plan.children);
        rows_affected += execute_single_save_plan(pool, dialect, &plan).await?;
        stack.extend(children.into_iter().rev());
    }
    Ok(SaveExecution {
        rows_affected,
        mode: root_mode,
    })
}

async fn execute_single_save_plan(
    pool: &AnyPool,
    dialect: SqlDialect,
    plan: &SavePlan,
) -> OrmResult<u64> {
    let sql = dialect.render_sql(&plan.sql);
    let mut query = sqlx::query(&sql);
    for param in &plan.params {
        query = bind_scalar(query, param);
    }
    let result = query.execute(pool).await.map_err(database_error)?;
    Ok(result.rows_affected())
}

fn bind_scalar<'q>(
    query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
    value: &'q ScalarValue,
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    match value {
        ScalarValue::Null => query.bind(Option::<String>::None),
        ScalarValue::Text(value) => query.bind(value),
        ScalarValue::I64(value) => query.bind(*value),
        ScalarValue::U64(value) => query.bind(*value as i64),
        ScalarValue::F64(value) => query.bind(*value),
        ScalarValue::Bool(value) => query.bind(*value),
    }
}

fn row_to_json(row: &AnyRow) -> OrmResult<Value> {
    let mut object = Map::new();
    for (index, column) in row.columns().iter().enumerate() {
        let value = decode_cell(row, index)?;
        object.insert(column.name().to_string(), value);
    }
    Ok(Value::Object(object))
}

fn row_to_nested_json(row: &AnyRow, shape: &crate::FetchShape) -> OrmResult<Value> {
    let mut object = Map::new();
    for column in row.columns() {
        if let Some(column_name) = column.name().strip_prefix("__rimmer__root__") {
            let value = decode_cell_by_name(row, column.name())?;
            object.insert(column_name.to_string(), value);
        }
    }
    for field in shape.fields() {
        let Some(relation) = field.relation() else {
            continue;
        };
        if matches!(
            relation.kind(),
            crate::FieldKind::OneToMany | crate::FieldKind::ManyToMany
        ) {
            object.insert(field.name().to_string(), Value::Array(Vec::new()));
            continue;
        }
        let target_alias = child_column_alias(field.name(), relation.target_column());
        let target_id = decode_cell_by_name(row, &target_alias)?;
        if target_id == Value::Null {
            object.insert(field.name().to_string(), Value::Null);
            continue;
        }
        let mut child_object = Map::new();
        child_object.insert(relation.target_column().to_string(), target_id);
        if let Some(child) = field.child() {
            for child_field in child.fields() {
                if child_field.visible()
                    && let Some(column_name) = child_field.column_name()
                {
                    let alias = child_column_alias(field.name(), column_name);
                    let value = decode_cell_by_name(row, &alias)?;
                    child_object.insert(column_name.to_string(), value);
                }
            }
        }
        object.insert(field.name().to_string(), Value::Object(child_object));
    }
    Ok(Value::Object(object))
}

async fn load_collection_relations(
    pool: &AnyPool,
    dialect: SqlDialect,
    shape: &crate::FetchShape,
    parents: &mut [Value],
) -> OrmResult<()> {
    for field in shape.fields() {
        let Some(relation) = field.relation() else {
            continue;
        };
        let Some(child_shape) = field.child() else {
            continue;
        };
        match relation.kind() {
            crate::FieldKind::OneToMany => {
                load_one_to_many_relation(
                    pool,
                    dialect,
                    field.name(),
                    relation,
                    child_shape,
                    field.collection_options(),
                    parents,
                )
                .await?;
            }
            crate::FieldKind::ManyToMany => {
                load_many_to_many_relation(
                    pool,
                    dialect,
                    field.name(),
                    relation,
                    child_shape,
                    field.collection_options(),
                    parents,
                )
                .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn load_one_to_many_relation(
    pool: &AnyPool,
    dialect: SqlDialect,
    field_name: &str,
    relation: &crate::FetchRelation,
    child_shape: &crate::FetchShape,
    options: &crate::CollectionFetchOptions,
    parents: &mut [Value],
) -> OrmResult<()> {
    let parent_ids = collect_parent_ids(parents, relation.source_column());
    if parent_ids.is_empty() {
        for parent in parents {
            assign_child_array(parent, field_name, Vec::new());
        }
        return Ok(());
    }

    let collection_sql =
        build_one_to_many_sql(field_name, relation, child_shape, options, parent_ids.len());
    let sql = dialect.render_sql(&collection_sql.sql);
    let mut query = sqlx::query(&sql);
    for id in &parent_ids {
        query = bind_scalar(query, id);
    }
    for param in &collection_sql.params {
        query = bind_scalar(query, param);
    }
    let rows = query.fetch_all(pool).await.map_err(database_error)?;
    let grouped = group_child_rows(field_name, child_shape, rows)?;

    for parent in parents {
        let Some(parent_id) = object_field(parent, relation.source_column()) else {
            assign_child_array(parent, field_name, Vec::new());
            continue;
        };
        let key = json_key(parent_id);
        let children = grouped.get(&key).cloned().unwrap_or_default();
        assign_child_array(parent, field_name, children);
    }
    Ok(())
}

async fn load_many_to_many_relation(
    pool: &AnyPool,
    dialect: SqlDialect,
    field_name: &str,
    relation: &crate::FetchRelation,
    child_shape: &crate::FetchShape,
    options: &crate::CollectionFetchOptions,
    parents: &mut [Value],
) -> OrmResult<()> {
    let parent_ids = collect_parent_ids(parents, relation.source_column());
    if parent_ids.is_empty() {
        for parent in parents {
            assign_child_array(parent, field_name, Vec::new());
        }
        return Ok(());
    }

    let collection_sql =
        build_many_to_many_sql(field_name, relation, child_shape, options, parent_ids.len())?;
    let sql = dialect.render_sql(&collection_sql.sql);
    let mut query = sqlx::query(&sql);
    for id in &parent_ids {
        query = bind_scalar(query, id);
    }
    for param in &collection_sql.params {
        query = bind_scalar(query, param);
    }
    let rows = query.fetch_all(pool).await.map_err(database_error)?;
    let grouped = group_child_rows(field_name, child_shape, rows)?;

    for parent in parents {
        let Some(parent_id) = object_field(parent, relation.source_column()) else {
            assign_child_array(parent, field_name, Vec::new());
            continue;
        };
        let key = json_key(parent_id);
        let children = grouped.get(&key).cloned().unwrap_or_default();
        assign_child_array(parent, field_name, children);
    }
    Ok(())
}

fn collect_parent_ids(parents: &[Value], source_column: &str) -> Vec<ScalarValue> {
    let mut values = Vec::new();
    for parent in parents {
        let Some(value) = object_field(parent, source_column) else {
            continue;
        };
        if let Some(scalar) = json_to_scalar(value)
            && !values.iter().any(|existing| existing == &scalar)
        {
            values.push(scalar);
        }
    }
    values
}

fn build_one_to_many_sql(
    field_name: &str,
    relation: &crate::FetchRelation,
    child_shape: &crate::FetchShape,
    options: &crate::CollectionFetchOptions,
    parent_count: usize,
) -> CollectionSql {
    let parent_column = quoted_column(relation.target_table(), relation.target_column());
    let mut columns = vec![format!(
        "{} AS {}",
        parent_column,
        crate::expression::quote_identifier(&parent_alias(field_name))
    )];
    for child_field in child_shape.fields() {
        if child_field.visible()
            && let Some(column_name) = child_field.column_name()
        {
            columns.push(format!(
                "{} AS {}",
                quoted_column(relation.target_table(), column_name),
                crate::expression::quote_identifier(&child_column_alias(field_name, column_name))
            ));
        }
    }
    let placeholders = vec!["?"; parent_count].join(", ");
    let from_sql = crate::expression::quote_identifier(relation.target_table());
    let mut where_parts = vec![format!("{} IN ({})", parent_column, placeholders)];
    let mut params = Vec::new();
    push_filter(options, &mut where_parts, &mut params);
    build_collection_select(CollectionSelectInput {
        field_name,
        columns,
        from_sql,
        where_parts,
        parent_column,
        fallback_order: None,
        options,
        params,
    })
}

struct CollectionSql {
    sql: String,
    params: Vec<ScalarValue>,
}

struct CollectionSelectInput<'a> {
    field_name: &'a str,
    columns: Vec<String>,
    from_sql: String,
    where_parts: Vec<String>,
    parent_column: String,
    fallback_order: Option<String>,
    options: &'a crate::CollectionFetchOptions,
    params: Vec<ScalarValue>,
}

fn build_collection_select(input: CollectionSelectInput<'_>) -> CollectionSql {
    let CollectionSelectInput {
        field_name,
        columns,
        from_sql,
        where_parts,
        parent_column,
        fallback_order,
        options,
        params,
    } = input;
    let where_sql = where_parts.join(" AND ");
    let order_parts = collection_order_parts(options, fallback_order);
    let has_window = options.limit_value().is_some() || options.offset_value().unwrap_or(0) > 0;
    if has_window {
        let row_alias = row_number_alias(field_name);
        let window_order = if order_parts.is_empty() {
            parent_column.clone()
        } else {
            order_parts.join(", ")
        };
        let mut inner_columns = columns;
        inner_columns.push(format!(
            "ROW_NUMBER() OVER (PARTITION BY {} ORDER BY {}) AS {}",
            parent_column,
            window_order,
            crate::expression::quote_identifier(&row_alias)
        ));
        let inner_sql = format!(
            "SELECT {} FROM {} WHERE {}",
            inner_columns.join(", "),
            from_sql,
            where_sql
        );
        let row_conditions = row_window_conditions(options, &row_alias);
        return CollectionSql {
            sql: format!(
                "SELECT * FROM ({}) WHERE {} ORDER BY {}, {}",
                inner_sql,
                row_conditions.join(" AND "),
                crate::expression::quote_identifier(&parent_alias(field_name)),
                crate::expression::quote_identifier(&row_alias)
            ),
            params,
        };
    }
    let mut sql = format!(
        "SELECT {} FROM {} WHERE {}",
        columns.join(", "),
        from_sql,
        where_sql
    );
    let mut final_order_parts = vec![crate::expression::quote_identifier(&parent_alias(
        field_name,
    ))];
    final_order_parts.extend(order_parts);
    if !final_order_parts.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(&final_order_parts.join(", "));
    }
    CollectionSql { sql, params }
}

fn push_filter(
    options: &crate::CollectionFetchOptions,
    where_parts: &mut Vec<String>,
    params: &mut Vec<ScalarValue>,
) {
    if let Some(predicate) = options.filter_predicate() {
        where_parts.push(predicate.sql().to_string());
        params.extend_from_slice(predicate.params());
    }
}

fn collection_order_parts(
    options: &crate::CollectionFetchOptions,
    fallback_order: Option<String>,
) -> Vec<String> {
    let mut parts = options
        .orders()
        .iter()
        .map(crate::Order::to_sql)
        .collect::<Vec<_>>();
    if parts.is_empty()
        && let Some(fallback_order) = fallback_order
    {
        parts.push(fallback_order);
    }
    parts
}

fn row_window_conditions(options: &crate::CollectionFetchOptions, row_alias: &str) -> Vec<String> {
    let row_ref = crate::expression::quote_identifier(row_alias);
    let offset = options.offset_value().unwrap_or(0);
    let mut conditions = Vec::new();
    if offset > 0 {
        conditions.push(format!("{} > {}", row_ref, offset));
    }
    if let Some(limit) = options.limit_value() {
        conditions.push(format!("{} <= {}", row_ref, offset + limit));
    }
    conditions
}

fn row_number_alias(field_name: &str) -> String {
    format!("__rimmer__row__{field_name}")
}

fn build_many_to_many_sql(
    field_name: &str,
    relation: &crate::FetchRelation,
    child_shape: &crate::FetchShape,
    options: &crate::CollectionFetchOptions,
    parent_count: usize,
) -> OrmResult<CollectionSql> {
    let join_table = relation
        .join_table()
        .ok_or_else(|| OrmError::InvalidFetcherRelation {
            message: format!("many-to-many relation '{field_name}' requires join table metadata"),
        })?;
    let join_alias = many_to_many_join_alias(field_name);
    let parent_column = qualified_column(&join_alias, join_table.source_column());
    let target_join_column = qualified_column(&join_alias, join_table.target_column());
    let target_id_column = quoted_column(relation.target_table(), relation.target_column());
    let mut columns = vec![format!(
        "{} AS {}",
        parent_column,
        crate::expression::quote_identifier(&parent_alias(field_name))
    )];
    for child_field in child_shape.fields() {
        if child_field.visible()
            && let Some(column_name) = child_field.column_name()
        {
            columns.push(format!(
                "{} AS {}",
                quoted_column(relation.target_table(), column_name),
                crate::expression::quote_identifier(&child_column_alias(field_name, column_name))
            ));
        }
    }
    let placeholders = vec!["?"; parent_count].join(", ");
    let from_sql = format!(
        "{} {} JOIN {} ON {} = {}",
        crate::expression::quote_identifier(join_table.table_name()),
        crate::expression::quote_identifier(&join_alias),
        crate::expression::quote_identifier(relation.target_table()),
        target_join_column,
        target_id_column
    );
    let mut where_parts = vec![format!("{} IN ({})", parent_column, placeholders)];
    let mut params = Vec::new();
    push_filter(options, &mut where_parts, &mut params);
    Ok(build_collection_select(CollectionSelectInput {
        field_name,
        columns,
        from_sql,
        where_parts,
        parent_column,
        fallback_order: Some(target_id_column),
        options,
        params,
    }))
}

fn group_child_rows(
    field_name: &str,
    child_shape: &crate::FetchShape,
    rows: Vec<AnyRow>,
) -> OrmResult<std::collections::BTreeMap<String, Vec<Value>>> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<Value>>::new();
    let parent_alias = parent_alias(field_name);
    for row in rows {
        let parent_id = decode_cell_by_name(&row, &parent_alias)?;
        let key = json_key(&parent_id);
        let child = child_row_to_json(field_name, child_shape, &row)?;
        grouped.entry(key).or_default().push(child);
    }
    Ok(grouped)
}

fn child_row_to_json(
    field_name: &str,
    child_shape: &crate::FetchShape,
    row: &AnyRow,
) -> OrmResult<Value> {
    let mut object = Map::new();
    for child_field in child_shape.fields() {
        if child_field.visible()
            && let Some(column_name) = child_field.column_name()
        {
            let alias = child_column_alias(field_name, column_name);
            let value = decode_cell_by_name(row, &alias)?;
            object.insert(column_name.to_string(), value);
        }
    }
    Ok(Value::Object(object))
}

fn shape_has_relation(shape: &crate::FetchShape) -> bool {
    shape
        .fields()
        .iter()
        .any(|field| field.relation().is_some())
}

fn quoted_column(table_name: &str, column_name: &str) -> String {
    qualified_column(table_name, column_name)
}

fn qualified_column(table_ref: &str, column_name: &str) -> String {
    format!(
        "{}.{}",
        crate::expression::quote_identifier(table_ref),
        crate::expression::quote_identifier(column_name)
    )
}

fn object_field<'a>(value: &'a Value, field_name: &str) -> Option<&'a Value> {
    match value {
        Value::Object(object) => object.get(field_name),
        _ => None,
    }
}

fn assign_child_array(parent: &mut Value, field_name: &str, children: Vec<Value>) {
    if let Value::Object(object) = parent {
        object.insert(field_name.to_string(), Value::Array(children));
    }
}

fn json_to_scalar(value: &Value) -> Option<ScalarValue> {
    match value {
        Value::Number(number) => number.as_i64().map(ScalarValue::I64),
        Value::String(value) => Some(ScalarValue::Text(value.clone())),
        Value::Bool(value) => Some(ScalarValue::Bool(*value)),
        _ => None,
    }
}

fn json_key(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn parent_alias(field_name: &str) -> String {
    format!("__rimmer__parent__{field_name}")
}

fn many_to_many_join_alias(field_name: &str) -> String {
    format!("__rimmer_m2m_join__{field_name}")
}

fn decode_cell(row: &AnyRow, index: usize) -> OrmResult<Value> {
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return Ok(value.map_or(Value::Null, |value| Value::Number(value.into())));
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return Ok(value.map_or(Value::Null, number_to_json));
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return Ok(value.map_or(Value::Null, Value::Bool));
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return Ok(value.map_or(Value::Null, Value::String));
    }
    Err(OrmError::RowDecode {
        message: format!("unsupported sqlx Any value at column index {index}"),
    })
}

fn decode_cell_by_name(row: &AnyRow, name: &str) -> OrmResult<Value> {
    if let Ok(value) = row.try_get::<Option<i64>, _>(name) {
        return Ok(value.map_or(Value::Null, |value| Value::Number(value.into())));
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(name) {
        return Ok(value.map_or(Value::Null, number_to_json));
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(name) {
        return Ok(value.map_or(Value::Null, Value::Bool));
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(name) {
        return Ok(value.map_or(Value::Null, Value::String));
    }
    Err(OrmError::RowDecode {
        message: format!("unsupported sqlx Any value at column '{name}'"),
    })
}

fn number_to_json(value: f64) -> Value {
    Number::from_f64(value).map_or(Value::Null, Value::Number)
}

fn database_error(source: sqlx::Error) -> OrmError {
    OrmError::Database {
        message: source.to_string(),
    }
}
