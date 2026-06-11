use crate::error::{OrmError, OrmResult};
use crate::expression::{IntoPredicate, Order, Predicate, quote_identifier};
use crate::fetcher::{FetchField, FetchShape, Fetcher};
use crate::metadata::{EntityDef, Table};
use crate::save::SaveCommand;
use crate::value::ScalarValue;

/// Jimmer 风格 SQL 客户端。
#[derive(Clone, Copy, Debug, Default)]
pub struct JimmerClient;

impl JimmerClient {
    /// 创建 SQL 客户端。
    pub const fn new() -> Self {
        Self
    }

    /// 创建查询构建器。
    pub fn create_query<E>(&self, table: Table<E>) -> QueryBuilder<E> {
        QueryBuilder::new(table)
    }

    /// 创建保存命令。
    pub fn save<E>(&self, draft: crate::Draft<E>) -> SaveCommand<E> {
        SaveCommand::new(draft)
    }
}

/// 查询构建器扩展方法。
pub trait QueryBuilderExt<E>: Sized {
    /// 追加动态谓词。
    fn where_(self, predicate: impl IntoPredicate) -> Self;

    /// 追加排序。
    fn order_by(self, order: Order) -> Self;

    /// 设置分页 limit。
    fn limit(self, value: usize) -> Self;

    /// 设置分页 offset。
    fn offset(self, value: usize) -> Self;

    /// 设置查询选择。
    fn select(self, selection: Selection<E>) -> Self;
}

/// SELECT 查询构建器。
pub struct QueryBuilder<E> {
    table: Table<E>,
    predicates: Vec<Predicate>,
    orders: Vec<Order>,
    limit: Option<usize>,
    offset: Option<usize>,
    selection: Option<Selection<E>>,
}

impl<E> QueryBuilder<E> {
    /// 创建查询构建器。
    pub fn new(table: Table<E>) -> Self {
        Self {
            table,
            predicates: Vec::new(),
            orders: Vec::new(),
            limit: None,
            offset: None,
            selection: None,
        }
    }

    /// 返回查询根表。
    pub fn table(&self) -> Table<E> {
        self.table
    }

    /// 构建 SQL plan。
    pub fn build(self) -> OrmResult<QueryPlan> {
        let selection = self.selection.ok_or(OrmError::MissingSelection)?;
        let selected = selection.selected_columns();
        let select_sql = if selected.columns.is_empty() {
            "*".to_string()
        } else {
            selected.columns.join(", ")
        };

        let mut sql = format!(
            "SELECT {} FROM {}",
            select_sql,
            quote_identifier(self.table.entity().table_name())
        );
        for join in &selected.joins {
            sql.push(' ');
            sql.push_str(&join.to_sql(self.table.entity().table_name()));
        }
        let mut params = Vec::new();

        if !self.predicates.is_empty() {
            let where_parts = self
                .predicates
                .iter()
                .map(|predicate| predicate.sql().to_string())
                .collect::<Vec<_>>();
            sql.push_str(" WHERE ");
            sql.push_str(&where_parts.join(" AND "));
            for predicate in &self.predicates {
                params.extend_from_slice(predicate.params());
            }
        }

        if !self.orders.is_empty() {
            let order_parts = self.orders.iter().map(Order::to_sql).collect::<Vec<_>>();
            sql.push_str(" ORDER BY ");
            sql.push_str(&order_parts.join(", "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }

        if let Some(offset) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&offset.to_string());
        }

        Ok(QueryPlan {
            sql,
            params,
            fetch_shape: selection.fetch_shape().cloned(),
        })
    }
}

impl<E> QueryBuilderExt<E> for QueryBuilder<E> {
    fn where_(mut self, predicate: impl IntoPredicate) -> Self {
        if let Some(predicate) = predicate.into_predicate() {
            self.predicates.push(predicate);
        }
        self
    }

    fn order_by(mut self, order: Order) -> Self {
        self.orders.push(order);
        self
    }

    fn limit(mut self, value: usize) -> Self {
        self.limit = Some(value);
        self
    }

    fn offset(mut self, value: usize) -> Self {
        self.offset = Some(value);
        self
    }

    fn select(mut self, selection: Selection<E>) -> Self {
        self.selection = Some(selection);
        self
    }
}

/// 查询选择。
pub struct Selection<E> {
    entity: EntityDef<E>,
    fetcher: Fetcher<E>,
}

impl<E> Selection<E> {
    /// 使用 Fetcher 创建实体选择。
    pub fn fetch(entity: EntityDef<E>, fetcher: Fetcher<E>) -> Self {
        Self { entity, fetcher }
    }

    /// 返回 Fetcher 形状。
    pub fn fetch_shape(&self) -> Option<&FetchShape> {
        Some(self.fetcher.shape())
    }

    fn selected_columns(&self) -> SelectedQueryShape {
        let mut columns = Vec::new();
        let mut joins = Vec::new();
        let has_join = self
            .fetcher
            .shape()
            .fields()
            .iter()
            .any(|field| field.relation().is_some());
        for field in self.entity.fields() {
            if field.kind().is_column() && field.kind() == crate::FieldKind::Id {
                push_column_once(
                    &mut columns,
                    self.entity.table_name(),
                    None,
                    field.column_name(),
                    has_join.then(|| root_column_alias(field.column_name())),
                );
            }
        }
        for field in self.fetcher.shape().fields() {
            if field.visible()
                && let Some(column_name) = field.column_name()
            {
                push_column_once(
                    &mut columns,
                    self.entity.table_name(),
                    None,
                    column_name,
                    has_join.then(|| root_column_alias(column_name)),
                );
            }
            if let Some(join) = JoinClause::from_fetch_field(field) {
                if let Some(child) = field.child() {
                    push_column_once(
                        &mut columns,
                        child.table_name(),
                        Some(&join.alias),
                        &join.target_column,
                        Some(child_column_alias(field.name(), &join.target_column)),
                    );
                    for child_field in child.fields() {
                        if child_field.visible()
                            && let Some(column_name) = child_field.column_name()
                        {
                            push_column_once(
                                &mut columns,
                                child.table_name(),
                                Some(&join.alias),
                                column_name,
                                Some(child_column_alias(field.name(), column_name)),
                            );
                        }
                    }
                }
                joins.push(join);
            }
        }
        SelectedQueryShape { columns, joins }
    }
}

/// 可执行前的 SQL 查询计划。
#[derive(Clone, Debug, PartialEq)]
pub struct QueryPlan {
    /// 参数化 SQL。
    pub sql: String,
    /// SQL 参数。
    pub params: Vec<ScalarValue>,
    /// Fetcher 对象形状。
    pub fetch_shape: Option<FetchShape>,
}

struct SelectedQueryShape {
    columns: Vec<String>,
    joins: Vec<JoinClause>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JoinClause {
    source_column: String,
    target_table: String,
    target_column: String,
    alias: String,
}

impl JoinClause {
    fn from_fetch_field(field: &FetchField) -> Option<Self> {
        let relation = field.relation()?;
        if relation.kind() != crate::FieldKind::ManyToOne {
            return None;
        }
        Some(Self {
            source_column: relation.source_column().to_string(),
            target_table: relation.target_table().to_string(),
            target_column: relation.target_column().to_string(),
            alias: join_alias(field.name()),
        })
    }

    fn to_sql(&self, root_table_name: &str) -> String {
        format!(
            "LEFT JOIN {} {} ON {}.{} = {}.{}",
            quote_identifier(&self.target_table),
            quote_identifier(&self.alias),
            quote_identifier(root_table_name),
            quote_identifier(&self.source_column),
            quote_identifier(&self.alias),
            quote_identifier(&self.target_column)
        )
    }
}

fn push_column_once(
    columns: &mut Vec<String>,
    table_name: &str,
    table_alias: Option<&str>,
    column_name: &str,
    column_alias: Option<String>,
) {
    let table_ref = table_alias.unwrap_or(table_name);
    let mut column = format!(
        "{}.{}",
        quote_identifier(table_ref),
        quote_identifier(column_name)
    );
    if let Some(alias) = column_alias {
        column.push_str(" AS ");
        column.push_str(&quote_identifier(&alias));
    }
    if !columns.iter().any(|existing| existing == &column) {
        columns.push(column);
    }
}

pub(crate) fn root_column_alias(column_name: &str) -> String {
    format!("__rimmer__root__{column_name}")
}

pub(crate) fn child_column_alias(field_name: &str, column_name: &str) -> String {
    format!("__rimmer__{field_name}__{column_name}")
}

pub(crate) fn join_alias(field_name: &str) -> String {
    format!("__rimmer_join__{field_name}")
}
