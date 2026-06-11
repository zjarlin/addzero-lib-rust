use crate::expression::Field;
use crate::metadata::{EntityDef, FieldKind, FieldMetadata};
use crate::value::{ScalarValue, ToScalarValue};
use std::marker::PhantomData;

/// 创建 Draft 的入口函数。
pub fn new_draft<E>(entity: EntityDef<E>) -> DraftCreator<E> {
    DraftCreator::new(entity)
}

/// Draft 创建器。
pub struct DraftCreator<E> {
    entity: EntityDef<E>,
}

impl<E> DraftCreator<E> {
    /// 创建 Draft 创建器。
    pub const fn new(entity: EntityDef<E>) -> Self {
        Self { entity }
    }

    /// 使用闭包配置 Draft。
    pub fn by<F>(self, block: F) -> Draft<E>
    where
        F: FnOnce(Draft<E>) -> Draft<E>,
    {
        block(Draft::new(self.entity))
    }
}

/// Jimmer 风格 Draft，用于表达部分对象保存。
pub struct Draft<E> {
    entity: EntityDef<E>,
    fields: Vec<DraftField>,
    collections: Vec<DraftCollection>,
}

impl<E> Draft<E> {
    /// 创建空 Draft。
    pub fn new(entity: EntityDef<E>) -> Self {
        Self {
            entity,
            fields: Vec::new(),
            collections: Vec::new(),
        }
    }

    /// 返回实体元模型。
    pub fn entity(&self) -> EntityDef<E> {
        self.entity
    }

    /// 返回已显式指定的字段。
    pub fn fields(&self) -> &[DraftField] {
        &self.fields
    }

    /// 返回已显式指定的集合子图。
    pub fn collections(&self) -> &[DraftCollection] {
        &self.collections
    }

    /// 设置字段值。
    pub fn set<V, T>(mut self, field: Field<E, V>, value: T) -> Self
    where
        T: ToScalarValue,
    {
        self.set_raw(
            field.rust_name(),
            field.column_name(),
            field.kind(),
            value.to_scalar_value(),
        );
        self
    }

    /// 显式设置字段为 SQL NULL。
    pub fn set_null<V>(mut self, field: Field<E, V>) -> Self {
        self.set_raw(
            field.rust_name(),
            field.column_name(),
            field.kind(),
            ScalarValue::Null,
        );
        self
    }

    /// 判断字段是否已经显式指定。
    pub fn is_specified<V>(&self, field: Field<E, V>) -> bool {
        self.fields
            .iter()
            .any(|item| item.rust_name() == field.rust_name())
    }

    /// 设置一对多子图，保存时会为子对象自动补充反向外键。
    pub fn one_to_many<T, SV, TV>(
        mut self,
        name: &'static str,
        source_field: Field<E, SV>,
        target_field: Field<T, TV>,
        children: Vec<Draft<T>>,
    ) -> Self {
        let children = children
            .into_iter()
            .map(Draft::into_erased)
            .collect::<Vec<_>>();
        self.collections.push(DraftCollection::new(
            name,
            source_field.column_name(),
            target_field.rust_name(),
            target_field.column_name(),
            target_field.kind(),
            children,
        ));
        self
    }

    /// 转成类型擦除 Draft，供图保存计划复用。
    pub fn into_erased(self) -> ErasedDraft {
        ErasedDraft {
            entity: ErasedEntityDef::from_entity(self.entity),
            fields: self.fields,
            collections: self.collections,
        }
    }

    fn set_raw(
        &mut self,
        rust_name: &'static str,
        column_name: &'static str,
        kind: FieldKind,
        value: ScalarValue,
    ) {
        if let Some(existing) = self
            .fields
            .iter_mut()
            .find(|existing| existing.rust_name == rust_name)
        {
            existing.value = value;
            return;
        }
        self.fields
            .push(DraftField::new(rust_name, column_name, kind, value));
    }
}

/// 类型擦除的 Draft，用于保存任意实体组成的对象图。
#[derive(Clone, Debug, PartialEq)]
pub struct ErasedDraft {
    entity: ErasedEntityDef,
    fields: Vec<DraftField>,
    collections: Vec<DraftCollection>,
}

impl ErasedDraft {
    /// 返回实体元数据。
    pub fn entity(&self) -> ErasedEntityDef {
        self.entity
    }

    /// 返回已显式指定的字段。
    pub fn fields(&self) -> &[DraftField] {
        &self.fields
    }

    /// 返回已显式指定的集合子图。
    pub fn collections(&self) -> &[DraftCollection] {
        &self.collections
    }

    /// 查找指定列的 Draft 字段。
    pub fn field_by_column(&self, column_name: &str) -> Option<&DraftField> {
        self.fields
            .iter()
            .find(|field| field.column_name() == column_name)
    }

    /// 设置或覆盖字段值。
    pub fn set_raw(
        &mut self,
        rust_name: &'static str,
        column_name: &'static str,
        kind: FieldKind,
        value: ScalarValue,
    ) {
        if let Some(existing) = self
            .fields
            .iter_mut()
            .find(|existing| existing.rust_name == rust_name)
        {
            existing.value = value;
            return;
        }
        self.fields
            .push(DraftField::new(rust_name, column_name, kind, value));
    }
}

/// 类型擦除的实体元数据。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErasedEntityDef {
    type_name: &'static str,
    table_name: &'static str,
    fields: &'static [FieldMetadata],
}

impl ErasedEntityDef {
    /// 从类型化实体元数据创建擦除实体元数据。
    pub fn from_entity<E>(entity: EntityDef<E>) -> Self {
        Self {
            type_name: entity.type_name(),
            table_name: entity.table_name(),
            fields: entity.fields(),
        }
    }

    /// 返回 Rust 侧实体类型名。
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// 返回数据库表名。
    pub const fn table_name(&self) -> &'static str {
        self.table_name
    }

    /// 查找主键字段。
    pub fn id_field(&self) -> Option<&'static FieldMetadata> {
        self.fields
            .iter()
            .find(|field| field.kind() == FieldKind::Id)
    }
}

/// Draft 中的集合子图。
#[derive(Clone, Debug, PartialEq)]
pub struct DraftCollection {
    name: &'static str,
    source_column: &'static str,
    target_rust_name: &'static str,
    target_column: &'static str,
    target_kind: FieldKind,
    children: Vec<ErasedDraft>,
    marker: PhantomData<()>,
}

impl DraftCollection {
    /// 创建集合子图。
    pub fn new(
        name: &'static str,
        source_column: &'static str,
        target_rust_name: &'static str,
        target_column: &'static str,
        target_kind: FieldKind,
        children: Vec<ErasedDraft>,
    ) -> Self {
        Self {
            name,
            source_column,
            target_rust_name,
            target_column,
            target_kind,
            children,
            marker: PhantomData,
        }
    }

    /// 返回集合字段名。
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 返回父对象关联列名。
    pub fn source_column(&self) -> &'static str {
        self.source_column
    }

    /// 返回子对象外键 Rust 字段名。
    pub fn target_rust_name(&self) -> &'static str {
        self.target_rust_name
    }

    /// 返回子对象外键列名。
    pub fn target_column(&self) -> &'static str {
        self.target_column
    }

    /// 返回子对象外键字段种类。
    pub fn target_kind(&self) -> FieldKind {
        self.target_kind
    }

    /// 返回集合子对象 Draft。
    pub fn children(&self) -> &[ErasedDraft] {
        &self.children
    }
}

/// Draft 中已显式指定的字段。
#[derive(Clone, Debug, PartialEq)]
pub struct DraftField {
    rust_name: &'static str,
    column_name: &'static str,
    kind: FieldKind,
    value: ScalarValue,
}

impl DraftField {
    /// 创建 Draft 字段。
    pub fn new(
        rust_name: &'static str,
        column_name: &'static str,
        kind: FieldKind,
        value: ScalarValue,
    ) -> Self {
        Self {
            rust_name,
            column_name,
            kind,
            value,
        }
    }

    /// 返回 Rust 字段名。
    pub fn rust_name(&self) -> &'static str {
        self.rust_name
    }

    /// 返回数据库列名。
    pub fn column_name(&self) -> &'static str {
        self.column_name
    }

    /// 返回字段种类。
    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    /// 返回字段值。
    pub fn value(&self) -> &ScalarValue {
        &self.value
    }
}
