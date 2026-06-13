use std::marker::PhantomData;

use crate::expression::{Field, IntoPredicate, Order, Predicate};
use crate::metadata::{EntityDef, FieldKind};
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 创建 Fetcher 的入口函数。
pub fn new_fetcher<E>(entity: EntityDef<E>) -> FetcherCreator<E> {
    FetcherCreator::new(entity)
}

/// Fetcher 创建器。
pub struct FetcherCreator<E> {
    entity: EntityDef<E>,
}

impl<E> FetcherCreator<E> {
    /// 创建 Fetcher 创建器。
    pub const fn new(entity: EntityDef<E>) -> Self {
        Self { entity }
    }

    /// 使用闭包配置 Fetcher。
    pub fn by<F>(self, block: F) -> Fetcher<E>
    where
        F: FnOnce(FetcherBuilder<E>) -> FetcherBuilder<E>,
    {
        block(FetcherBuilder::new(self.entity)).build()
    }
}

/// Jimmer 风格 Fetcher，用于描述返回对象形状。
pub struct Fetcher<E> {
    shape: FetchShape,
    marker: PhantomData<E>,
}

impl<E> std::fmt::Debug for Fetcher<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Fetcher")
            .field("shape", &self.shape)
            .finish()
    }
}

impl<E> Clone for Fetcher<E> {
    fn clone(&self) -> Self {
        Self {
            shape: self.shape.clone(),
            marker: PhantomData,
        }
    }
}

impl<E> Fetcher<E> {
    /// 创建 Fetcher。
    pub fn new(shape: FetchShape) -> Self {
        Self {
            shape,
            marker: PhantomData,
        }
    }

    /// 从通用 Fetcher 形状创建类型化 Fetcher。
    pub fn from_shape(entity: EntityDef<E>, shape: FetchShape) -> anyhow::Result<Self> {
        if shape.entity_name() != entity.type_name() {
            bail!(
                "fetcher shape targets '{}', expected '{}'",
                shape.entity_name(),
                entity.type_name()
            );
        }
        if shape.table_name() != entity.table_name() {
            bail!(
                "fetcher shape targets '{}', expected '{}'",
                shape.table_name(),
                entity.table_name()
            );
        }
        validate_shape(entity, &shape)?;
        Ok(Self::new(shape))
    }

    /// 从 JSON 形状创建类型化 Fetcher。
    pub fn from_json(entity: EntityDef<E>, json: &str) -> anyhow::Result<Self> {
        let shape: FetchShape =
            serde_json::from_str(json).context("failed to deserialize fetcher shape")?;
        Self::from_shape(entity, shape)
    }

    /// 从 JSON Value 形状创建类型化 Fetcher。
    pub fn from_json_value(entity: EntityDef<E>, value: Value) -> anyhow::Result<Self> {
        let shape: FetchShape =
            serde_json::from_value(value).context("failed to deserialize fetcher shape")?;
        Self::from_shape(entity, shape)
    }

    /// 返回 Fetcher 形状。
    pub fn shape(&self) -> &FetchShape {
        &self.shape
    }

    /// 将 Fetcher 形状序列化为紧凑 JSON。
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(&self.shape).context("failed to serialize fetcher shape")
    }

    /// 将 Fetcher 形状序列化为 JSON Value。
    pub fn to_json_value(&self) -> anyhow::Result<Value> {
        serde_json::to_value(&self.shape).context("failed to serialize fetcher shape")
    }

    /// 将 Fetcher 形状序列化为易读 JSON。
    pub fn to_pretty_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(&self.shape).context("failed to serialize fetcher shape")
    }
}

fn validate_shape<E>(entity: EntityDef<E>, shape: &FetchShape) -> anyhow::Result<()> {
    for field in shape.fields() {
        validate_column_field(entity, field)?;
        validate_relation_field(entity, field)?;
    }
    Ok(())
}

fn validate_column_field<E>(entity: EntityDef<E>, field: &FetchField) -> anyhow::Result<()> {
    let Some(column_name) = field.column_name() else {
        return Ok(());
    };
    let Some(metadata) = entity
        .fields()
        .iter()
        .find(|metadata| metadata.rust_name() == field.name())
    else {
        bail!(
            "invalid fetcher shape: field '{}' does not exist on entity '{}'",
            field.name(),
            entity.type_name()
        );
    };
    if metadata.column_name() != column_name || metadata.kind() != field.kind() {
        bail!(
            "invalid fetcher shape: field '{}' maps to '{}:{:?}', expected '{}:{:?}'",
            field.name(),
            column_name,
            field.kind(),
            metadata.column_name(),
            metadata.kind()
        );
    }
    Ok(())
}

fn validate_relation_field<E>(entity: EntityDef<E>, field: &FetchField) -> anyhow::Result<()> {
    let Some(relation) = field.relation() else {
        return Ok(());
    };
    if !entity
        .fields()
        .iter()
        .any(|metadata| metadata.column_name() == relation.source_column())
    {
        bail!(
            "invalid fetcher shape: relation '{}' source column '{}' does not exist on entity '{}'",
            field.name(),
            relation.source_column(),
            entity.type_name()
        );
    }
    if let Some(child) = field.child()
        && child.table_name() != relation.target_table()
    {
        bail!(
            "invalid fetcher shape: relation '{}' targets table '{}', but child shape targets '{}'",
            field.name(),
            relation.target_table(),
            child.table_name()
        );
    }
    Ok(())
}

/// Fetcher 构建器。
pub struct FetcherBuilder<E> {
    entity: EntityDef<E>,
    fields: Vec<FetchField>,
}

impl<E> FetcherBuilder<E> {
    /// 创建 Fetcher 构建器。
    pub fn new(entity: EntityDef<E>) -> Self {
        Self {
            entity,
            fields: Vec::new(),
        }
    }

    /// 追加标量字段。
    pub fn field<V>(mut self, field: Field<E, V>) -> Self {
        self.push_field(FetchField::column(
            field.rust_name(),
            field.column_name(),
            field.kind(),
            true,
        ));
        self
    }

    /// 追加隐藏字段。
    pub fn field_hidden<V>(mut self, field: Field<E, V>) -> Self {
        self.push_field(FetchField::column(
            field.rust_name(),
            field.column_name(),
            field.kind(),
            false,
        ));
        self
    }

    /// 追加所有根实体标量字段。
    pub fn all_scalar_fields(mut self) -> Self {
        for field in self.entity.fields() {
            if field.kind().is_column() {
                self.push_field(FetchField::column(
                    field.rust_name(),
                    field.column_name(),
                    field.kind(),
                    true,
                ));
            }
        }
        self
    }

    /// 追加关联子对象形状。
    pub fn association<T>(mut self, name: &'static str, child: Fetcher<T>) -> Self {
        self.push_field(FetchField::association(name, child.shape().clone()));
        self
    }

    /// 追加多对一关联子对象形状，并声明 join 元数据。
    pub fn many_to_one<T, SV, TV>(
        mut self,
        name: &'static str,
        source_field: Field<E, SV>,
        target_field: Field<T, TV>,
        child: Fetcher<T>,
    ) -> Self {
        self.push_field(FetchField::many_to_one(
            name,
            source_field.column_name(),
            target_field.entity().table_name(),
            target_field.column_name(),
            child.shape().clone(),
        ));
        self
    }

    /// 追加一对多关联子对象形状，并声明批量加载元数据。
    pub fn one_to_many<T, SV, TV>(
        self,
        name: &'static str,
        source_field: Field<E, SV>,
        target_field: Field<T, TV>,
        child: Fetcher<T>,
    ) -> Self {
        self.one_to_many_with_options(
            name,
            source_field,
            target_field,
            child,
            CollectionFetchOptions::default(),
        )
    }

    /// 追加带关联级配置的一对多关联子对象形状。
    pub fn one_to_many_with_options<T, SV, TV>(
        mut self,
        name: &'static str,
        source_field: Field<E, SV>,
        target_field: Field<T, TV>,
        child: Fetcher<T>,
        options: CollectionFetchOptions,
    ) -> Self {
        self.push_field(FetchField::one_to_many(
            name,
            source_field.column_name(),
            target_field.entity().table_name(),
            target_field.column_name(),
            child.shape().clone(),
            options,
        ));
        self
    }

    /// 追加多对多关联子对象形状，并声明中间表批量加载元数据。
    pub fn many_to_many<T, SV, TV>(
        self,
        name: &'static str,
        source_field: Field<E, SV>,
        join: ManyToManyJoin<T, TV>,
        child: Fetcher<T>,
    ) -> Self {
        self.many_to_many_with_options(
            name,
            source_field,
            join,
            child,
            CollectionFetchOptions::default(),
        )
    }

    /// 追加带关联级配置的多对多关联子对象形状。
    pub fn many_to_many_with_options<T, SV, TV>(
        mut self,
        name: &'static str,
        source_field: Field<E, SV>,
        join: ManyToManyJoin<T, TV>,
        child: Fetcher<T>,
        options: CollectionFetchOptions,
    ) -> Self {
        self.push_field(FetchField::many_to_many(
            name,
            source_field.column_name(),
            join.target_field.entity().table_name(),
            join.target_field.column_name(),
            FetchJoinTable::new(join.table_name, join.source_column, join.target_column),
            child.shape().clone(),
            options,
        ));
        self
    }

    /// 追加递归关联字段。
    pub fn recursive(mut self, name: &'static str) -> Self {
        self.push_field(FetchField::recursive(name));
        self
    }

    /// 完成 Fetcher 构建。
    pub fn build(self) -> Fetcher<E> {
        Fetcher::new(FetchShape::new(
            self.entity.type_name().to_string(),
            self.entity.table_name().to_string(),
            self.fields,
        ))
    }

    fn push_field(&mut self, field: FetchField) {
        if let Some(existing) = self
            .fields
            .iter_mut()
            .find(|existing| existing.name == field.name)
        {
            *existing = field;
            return;
        }
        self.fields.push(field);
    }
}

/// Fetcher 的可序列化形状。
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchShape {
    entity_name: String,
    table_name: String,
    fields: Vec<FetchField>,
}

impl FetchShape {
    /// 创建 Fetcher 形状。
    pub fn new(entity_name: String, table_name: String, fields: Vec<FetchField>) -> Self {
        Self {
            entity_name,
            table_name,
            fields,
        }
    }

    /// 返回实体类型名。
    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    /// 返回表名。
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// 返回字段形状。
    pub fn fields(&self) -> &[FetchField] {
        &self.fields
    }
}

/// Fetcher 字段形状。
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchField {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_name: Option<String>,
    kind: FieldKind,
    visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    child: Option<Box<FetchShape>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relation: Option<FetchRelation>,
    #[serde(default, skip_serializing_if = "CollectionFetchOptions::is_empty")]
    collection_options: CollectionFetchOptions,
    recursive: bool,
}

impl FetchField {
    /// 创建根表列字段。
    pub fn column(
        name: &'static str,
        column_name: &'static str,
        kind: FieldKind,
        visible: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            column_name: Some(column_name.to_string()),
            kind,
            visible,
            child: None,
            relation: None,
            collection_options: CollectionFetchOptions::default(),
            recursive: false,
        }
    }

    /// 创建关联字段。
    pub fn association(name: &'static str, child: FetchShape) -> Self {
        Self {
            name: name.to_string(),
            column_name: None,
            kind: FieldKind::ManyToOne,
            visible: true,
            child: Some(Box::new(child)),
            relation: None,
            collection_options: CollectionFetchOptions::default(),
            recursive: false,
        }
    }

    /// 创建多对一关联字段。
    pub fn many_to_one(
        name: &'static str,
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
        child: FetchShape,
    ) -> Self {
        Self {
            name: name.to_string(),
            column_name: None,
            kind: FieldKind::ManyToOne,
            visible: true,
            child: Some(Box::new(child)),
            relation: Some(FetchRelation::many_to_one(
                source_column,
                target_table,
                target_column,
            )),
            collection_options: CollectionFetchOptions::default(),
            recursive: false,
        }
    }

    /// 创建一对多关联字段。
    pub fn one_to_many(
        name: &'static str,
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
        child: FetchShape,
        options: CollectionFetchOptions,
    ) -> Self {
        Self {
            name: name.to_string(),
            column_name: None,
            kind: FieldKind::OneToMany,
            visible: true,
            child: Some(Box::new(child)),
            relation: Some(FetchRelation::one_to_many(
                source_column,
                target_table,
                target_column,
            )),
            collection_options: options,
            recursive: false,
        }
    }

    /// 创建多对多关联字段。
    pub fn many_to_many(
        name: &'static str,
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
        join_table: FetchJoinTable,
        child: FetchShape,
        options: CollectionFetchOptions,
    ) -> Self {
        Self {
            name: name.to_string(),
            column_name: None,
            kind: FieldKind::ManyToMany,
            visible: true,
            child: Some(Box::new(child)),
            relation: Some(FetchRelation::many_to_many(
                source_column,
                target_table,
                target_column,
                join_table,
            )),
            collection_options: options,
            recursive: false,
        }
    }

    /// 创建递归关联字段。
    pub fn recursive(name: &'static str) -> Self {
        Self {
            name: name.to_string(),
            column_name: None,
            kind: FieldKind::OneToMany,
            visible: true,
            child: None,
            relation: None,
            collection_options: CollectionFetchOptions::default(),
            recursive: true,
        }
    }

    /// 返回字段名。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回列名。
    pub fn column_name(&self) -> Option<&str> {
        self.column_name.as_deref()
    }

    /// 返回字段种类。
    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    /// 判断字段是否对外可见。
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// 返回子对象形状。
    pub fn child(&self) -> Option<&FetchShape> {
        self.child.as_deref()
    }

    /// 返回关联 join 元数据。
    pub fn relation(&self) -> Option<&FetchRelation> {
        self.relation.as_ref()
    }

    /// 返回集合关联级配置。
    pub fn collection_options(&self) -> &CollectionFetchOptions {
        &self.collection_options
    }

    /// 判断是否是递归关联。
    pub fn recursive_enabled(&self) -> bool {
        self.recursive
    }
}

/// Fetcher 关联 join 元数据。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRelation {
    kind: FieldKind,
    source_column: String,
    target_table: String,
    target_column: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    join_table: Option<FetchJoinTable>,
}

impl FetchRelation {
    /// 创建多对一关联 join 元数据。
    pub fn many_to_one(
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
    ) -> Self {
        Self {
            kind: FieldKind::ManyToOne,
            source_column: source_column.to_string(),
            target_table: target_table.to_string(),
            target_column: target_column.to_string(),
            join_table: None,
        }
    }

    /// 创建一对多关联批量加载元数据。
    pub fn one_to_many(
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
    ) -> Self {
        Self {
            kind: FieldKind::OneToMany,
            source_column: source_column.to_string(),
            target_table: target_table.to_string(),
            target_column: target_column.to_string(),
            join_table: None,
        }
    }

    /// 创建多对多关联批量加载元数据。
    pub fn many_to_many(
        source_column: &'static str,
        target_table: &'static str,
        target_column: &'static str,
        join_table: FetchJoinTable,
    ) -> Self {
        Self {
            kind: FieldKind::ManyToMany,
            source_column: source_column.to_string(),
            target_table: target_table.to_string(),
            target_column: target_column.to_string(),
            join_table: Some(join_table),
        }
    }

    /// 返回关联种类。
    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    /// 返回本方关联列。
    pub fn source_column(&self) -> &str {
        &self.source_column
    }

    /// 返回目标表名。
    pub fn target_table(&self) -> &str {
        &self.target_table
    }

    /// 返回目标表关联列。
    pub fn target_column(&self) -> &str {
        &self.target_column
    }

    /// 返回多对多中间表元数据。
    pub fn join_table(&self) -> Option<&FetchJoinTable> {
        self.join_table.as_ref()
    }
}

/// 集合 Fetcher 的关联级配置。
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionFetchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    orders: Vec<Order>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<usize>,
}

impl CollectionFetchOptions {
    /// 创建空的集合关联级配置。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置集合关联级过滤条件。
    pub fn filter(mut self, predicate: impl IntoPredicate) -> Self {
        self.filter = predicate.into_predicate();
        self
    }

    /// 追加集合关联级排序。
    pub fn order_by(mut self, order: Order) -> Self {
        self.orders.push(order);
        self
    }

    /// 设置每个父对象下集合的 limit。
    pub fn limit(mut self, value: usize) -> Self {
        self.limit = Some(value);
        self
    }

    /// 设置每个父对象下集合的 offset。
    pub fn offset(mut self, value: usize) -> Self {
        self.offset = Some(value);
        self
    }

    /// 返回集合关联级过滤条件。
    pub fn filter_predicate(&self) -> Option<&Predicate> {
        self.filter.as_ref()
    }

    /// 返回集合关联级排序。
    pub fn orders(&self) -> &[Order] {
        &self.orders
    }

    /// 返回每个父对象下集合的 limit。
    pub fn limit_value(&self) -> Option<usize> {
        self.limit
    }

    /// 返回每个父对象下集合的 offset。
    pub fn offset_value(&self) -> Option<usize> {
        self.offset
    }

    fn is_empty(&self) -> bool {
        self.filter.is_none()
            && self.orders.is_empty()
            && self.limit.is_none()
            && self.offset.is_none()
    }
}

/// 多对多中间表描述，用结构体字面量传给 Fetcher。
#[derive(Clone, Copy)]
pub struct ManyToManyJoin<T, V> {
    /// 中间表名。
    pub table_name: &'static str,
    /// 中间表中指向本方的列。
    pub source_column: &'static str,
    /// 中间表中指向目标方的列。
    pub target_column: &'static str,
    /// 目标实体的关联列。
    pub target_field: Field<T, V>,
}

/// Fetcher 多对多中间表元数据。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchJoinTable {
    table_name: String,
    source_column: String,
    target_column: String,
}

impl FetchJoinTable {
    /// 创建多对多中间表元数据。
    pub fn new(
        table_name: &'static str,
        source_column: &'static str,
        target_column: &'static str,
    ) -> Self {
        Self {
            table_name: table_name.to_string(),
            source_column: source_column.to_string(),
            target_column: target_column.to_string(),
        }
    }

    /// 返回中间表名。
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// 返回中间表中指向本方的列。
    pub fn source_column(&self) -> &str {
        &self.source_column
    }

    /// 返回中间表中指向目标方的列。
    pub fn target_column(&self) -> &str {
        &self.target_column
    }
}
