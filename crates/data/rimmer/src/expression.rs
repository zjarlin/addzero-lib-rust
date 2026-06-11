use std::marker::PhantomData;

use crate::fetcher::ManyToManyJoin;
use crate::metadata::{EntityDef, FieldKind};
use crate::value::{ScalarValue, ToScalarValue};
use serde::{Deserialize, Serialize};

/// 实体字段表达式。
pub struct Field<E, V> {
    entity: EntityDef<E>,
    rust_name: &'static str,
    column_name: &'static str,
    kind: FieldKind,
    marker: PhantomData<V>,
}

impl<E, V> Clone for Field<E, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E, V> Copy for Field<E, V> {}

impl<E, V> Field<E, V> {
    /// 创建字段表达式。
    pub const fn new(
        entity: EntityDef<E>,
        rust_name: &'static str,
        column_name: &'static str,
        kind: FieldKind,
    ) -> Self {
        Self {
            entity,
            rust_name,
            column_name,
            kind,
            marker: PhantomData,
        }
    }

    /// 返回字段所属实体。
    pub const fn entity(&self) -> EntityDef<E> {
        self.entity
    }

    /// 返回 Rust 字段名。
    pub const fn rust_name(&self) -> &'static str {
        self.rust_name
    }

    /// 返回数据库列名。
    pub const fn column_name(&self) -> &'static str {
        self.column_name
    }

    /// 返回字段种类。
    pub const fn kind(&self) -> FieldKind {
        self.kind
    }

    /// 创建等值谓词。
    pub fn eq<T>(&self, value: T) -> Predicate
    where
        T: ToScalarValue,
    {
        Predicate::new(
            format!("{} = ?", self.sql_ref()),
            vec![value.to_scalar_value()],
        )
    }

    /// 当值存在时创建等值谓词。
    pub fn eq_if<T>(&self, value: Option<T>) -> Option<Predicate>
    where
        T: ToScalarValue,
    {
        value.map(|actual| self.eq(actual))
    }

    /// 当字符串存在且非空白时创建等值谓词。
    pub fn eq_if_not_blank<T>(&self, value: Option<T>) -> Option<Predicate>
    where
        T: AsRef<str>,
    {
        value.and_then(|actual| {
            let text = actual.as_ref().trim();
            if text.is_empty() {
                None
            } else {
                Some(self.eq(text.to_string()))
            }
        })
    }

    /// 创建 LIKE 谓词。
    pub fn ilike<T>(&self, value: T) -> Predicate
    where
        T: ToScalarValue,
    {
        Predicate::new(
            format!("{} ILIKE ?", self.sql_ref()),
            vec![value.to_scalar_value()],
        )
    }

    /// 当字符串存在且非空白时创建 LIKE 谓词。
    pub fn ilike_if_not_blank<T>(&self, value: Option<T>) -> Option<Predicate>
    where
        T: AsRef<str>,
    {
        value.and_then(|actual| {
            let text = actual.as_ref().trim();
            if text.is_empty() {
                None
            } else {
                Some(self.ilike(format!("%{}%", text)))
            }
        })
    }

    /// 创建 BETWEEN 谓词。
    pub fn between<T>(&self, min: T, max: T) -> Predicate
    where
        T: ToScalarValue,
    {
        Predicate::new(
            format!("{} BETWEEN ? AND ?", self.sql_ref()),
            vec![min.to_scalar_value(), max.to_scalar_value()],
        )
    }

    /// 当左右边界同时存在时创建 BETWEEN 谓词。
    pub fn between_if<T>(&self, min: Option<T>, max: Option<T>) -> Option<Predicate>
    where
        T: ToScalarValue,
    {
        match (min, max) {
            (Some(min), Some(max)) => Some(self.between(min, max)),
            _ => None,
        }
    }

    /// 创建 IS NULL 谓词。
    pub fn is_null(&self) -> Predicate {
        Predicate::new(format!("{} IS NULL", self.sql_ref()), Vec::new())
    }

    /// 创建升序排序。
    pub fn asc(&self) -> Order {
        Order::new(self.sql_ref(), OrderDirection::Asc)
    }

    /// 创建降序排序。
    pub fn desc(&self) -> Order {
        Order::new(self.sql_ref(), OrderDirection::Desc)
    }

    /// 以当前字段作为本方列，创建一对多集合关联路径。
    pub fn one_to_many<T, TV>(&self, target_field: Field<T, TV>) -> CollectionRelation<E, T> {
        CollectionRelation::one_to_many(*self, target_field)
    }

    /// 以当前字段作为本方列，创建多对多集合关联路径。
    pub fn many_to_many<T, TV>(&self, join: ManyToManyJoin<T, TV>) -> CollectionRelation<E, T> {
        CollectionRelation::many_to_many(*self, join)
    }

    pub(crate) fn sql_ref(&self) -> String {
        format!(
            "{}.{}",
            quote_identifier(self.entity.table_name()),
            quote_identifier(self.column_name)
        )
    }
}

/// 查询谓词。
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Predicate {
    sql: String,
    params: Vec<ScalarValue>,
}

impl Predicate {
    /// 创建查询谓词。
    pub fn new(sql: String, params: Vec<ScalarValue>) -> Self {
        Self { sql, params }
    }

    /// 返回谓词 SQL。
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// 返回谓词参数。
    pub fn params(&self) -> &[ScalarValue] {
        &self.params
    }

    /// 使用 AND 合并另一个谓词。
    pub fn and(self, other: impl IntoPredicate) -> Self {
        self.combine("AND", other)
    }

    /// 使用 OR 合并另一个谓词。
    pub fn or(self, other: impl IntoPredicate) -> Self {
        self.combine("OR", other)
    }

    /// 对当前谓词取反。
    pub fn negate(self) -> Self {
        Self {
            sql: format!("NOT ({})", self.sql),
            params: self.params,
        }
    }

    fn combine(mut self, op: &str, other: impl IntoPredicate) -> Self {
        let Some(other) = other.into_predicate() else {
            return self;
        };
        self.sql = format!("({}) {} ({})", self.sql, op, other.sql);
        self.params.extend(other.params);
        self
    }
}

impl std::ops::Not for Predicate {
    type Output = Predicate;

    fn not(self) -> Self::Output {
        self.negate()
    }
}

/// 可选动态谓词转换。
pub trait IntoPredicate {
    /// 转换成可追加到查询的谓词。
    fn into_predicate(self) -> Option<Predicate>;
}

impl IntoPredicate for Predicate {
    fn into_predicate(self) -> Option<Predicate> {
        Some(self)
    }
}

impl IntoPredicate for Option<Predicate> {
    fn into_predicate(self) -> Option<Predicate> {
        self
    }
}

/// 集合关联路径，用于生成 Jimmer 风格隐式子查询谓词。
pub struct CollectionRelation<E, T> {
    source_table: &'static str,
    source_column: &'static str,
    target_table: &'static str,
    target_column: &'static str,
    join_table: Option<CollectionJoinTable>,
    marker: PhantomData<(E, T)>,
}

impl<E, T> CollectionRelation<E, T> {
    /// 创建一对多集合关联路径。
    pub fn one_to_many<SV, TV>(source_field: Field<E, SV>, target_field: Field<T, TV>) -> Self {
        Self {
            source_table: source_field.entity().table_name(),
            source_column: source_field.column_name(),
            target_table: target_field.entity().table_name(),
            target_column: target_field.column_name(),
            join_table: None,
            marker: PhantomData,
        }
    }

    /// 创建多对多集合关联路径。
    pub fn many_to_many<SV, TV>(source_field: Field<E, SV>, join: ManyToManyJoin<T, TV>) -> Self {
        Self {
            source_table: source_field.entity().table_name(),
            source_column: source_field.column_name(),
            target_table: join.target_field.entity().table_name(),
            target_column: join.target_field.column_name(),
            join_table: Some(CollectionJoinTable {
                table_name: join.table_name,
                source_column: join.source_column,
                target_column: join.target_column,
            }),
            marker: PhantomData,
        }
    }

    /// 创建“至少存在一个关联对象”的谓词。
    pub fn exists_any(&self) -> Predicate {
        self.exists_with_child_predicate(None, false)
    }

    /// 创建“存在满足条件的关联对象”的谓词。
    pub fn exists(&self, predicate: impl IntoPredicate) -> Predicate {
        self.exists_with_child_predicate(predicate.into_predicate(), false)
    }

    /// 创建“不存在任何关联对象”的谓词。
    pub fn not_exists_any(&self) -> Predicate {
        self.exists_with_child_predicate(None, true)
    }

    /// 创建“不存在满足条件的关联对象”的谓词。
    pub fn not_exists(&self, predicate: impl IntoPredicate) -> Predicate {
        self.exists_with_child_predicate(predicate.into_predicate(), true)
    }

    fn exists_with_child_predicate(
        &self,
        predicate: Option<Predicate>,
        negated: bool,
    ) -> Predicate {
        let mut params = Vec::new();
        let mut where_parts = Vec::new();
        let from_sql = match self.join_table {
            Some(join_table) => {
                where_parts.push(format!(
                    "{} = {}",
                    qualified_column(many_to_many_join_alias(), join_table.source_column),
                    qualified_column(self.source_table, self.source_column)
                ));
                format!(
                    "{} {} JOIN {} ON {} = {}",
                    quote_identifier(join_table.table_name),
                    quote_identifier(many_to_many_join_alias()),
                    quote_identifier(self.target_table),
                    qualified_column(many_to_many_join_alias(), join_table.target_column),
                    qualified_column(self.target_table, self.target_column)
                )
            }
            None => {
                where_parts.push(format!(
                    "{} = {}",
                    qualified_column(self.target_table, self.target_column),
                    qualified_column(self.source_table, self.source_column)
                ));
                quote_identifier(self.target_table)
            }
        };
        if let Some(predicate) = predicate {
            where_parts.push(predicate.sql);
            params.extend(predicate.params);
        }
        let operator = if negated { "NOT EXISTS" } else { "EXISTS" };
        Predicate::new(
            format!(
                "{} (SELECT 1 FROM {} WHERE {})",
                operator,
                from_sql,
                where_parts.join(" AND ")
            ),
            params,
        )
    }
}

#[derive(Clone, Copy)]
struct CollectionJoinTable {
    table_name: &'static str,
    source_column: &'static str,
    target_column: &'static str,
}

/// 排序表达式。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    expression: String,
    direction: OrderDirection,
}

impl Order {
    /// 创建排序表达式。
    fn new(expression: String, direction: OrderDirection) -> Self {
        Self {
            expression,
            direction,
        }
    }

    /// 返回排序 SQL。
    pub fn to_sql(&self) -> String {
        format!("{} {}", self.expression, self.direction.as_sql())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    fn as_sql(self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

pub(crate) fn quote_identifier(identifier: &str) -> String {
    let escaped = identifier.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

fn qualified_column(table_ref: &str, column_name: &str) -> String {
    format!(
        "{}.{}",
        quote_identifier(table_ref),
        quote_identifier(column_name)
    )
}

fn many_to_many_join_alias() -> &'static str {
    "__rimmer_exists_join"
}
