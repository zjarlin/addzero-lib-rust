use std::marker::PhantomData;

use crate::fetcher::Fetcher;
use crate::query::Selection;
use serde::{Deserialize, Serialize};

/// Jimmer 风格实体 marker trait。
pub trait Entity: Sized + 'static {
    /// 返回实体的静态元模型定义。
    fn entity() -> EntityDef<Self>;
}

/// 实体元模型。
pub struct EntityDef<E> {
    type_name: &'static str,
    table_name: &'static str,
    fields: &'static [FieldMetadata],
    marker: PhantomData<E>,
}

impl<E> Clone for EntityDef<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for EntityDef<E> {}

impl<E> EntityDef<E> {
    /// 创建实体元模型。
    pub const fn new(
        type_name: &'static str,
        table_name: &'static str,
        fields: &'static [FieldMetadata],
    ) -> Self {
        Self {
            type_name,
            table_name,
            fields,
            marker: PhantomData,
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

    /// 返回实体字段元数据。
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// 查找主键字段。
    pub fn id_field(&self) -> Option<&'static FieldMetadata> {
        self.fields
            .iter()
            .find(|field| field.kind() == FieldKind::Id)
    }
}

/// 表对象，用于承载查询根。
pub struct Table<E> {
    entity: EntityDef<E>,
}

impl<E> Clone for Table<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for Table<E> {}

impl<E> Table<E> {
    /// 创建表对象。
    pub const fn new(entity: EntityDef<E>) -> Self {
        Self { entity }
    }

    /// 返回实体元模型。
    pub const fn entity(&self) -> EntityDef<E> {
        self.entity
    }

    /// 使用 Fetcher 选择当前表的对象形状。
    pub fn fetch(&self, fetcher: Fetcher<E>) -> Selection<E> {
        Selection::fetch(self.entity, fetcher)
    }
}

/// 字段种类。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    /// 主键字段。
    Id,
    /// 业务键字段。
    Key,
    /// 普通标量字段。
    Scalar,
    /// 多对一关联。
    ManyToOne,
    /// 一对多关联。
    OneToMany,
    /// 多对多关联。
    ManyToMany,
    /// 计算字段。
    Transient,
    /// 关联 id 视图字段。
    IdView,
}

impl FieldKind {
    /// 判断字段是否可以直接映射到根表列。
    pub const fn is_column(self) -> bool {
        matches!(
            self,
            FieldKind::Id | FieldKind::Key | FieldKind::Scalar | FieldKind::IdView
        )
    }

    /// 判断字段是否可以作为数据库持久化列写入。
    pub const fn is_persistent_column(self) -> bool {
        matches!(
            self,
            FieldKind::Id
                | FieldKind::Key
                | FieldKind::Scalar
                | FieldKind::ManyToOne
                | FieldKind::IdView
        )
    }

    /// 判断字段是否是关联字段。
    pub const fn is_association(self) -> bool {
        matches!(
            self,
            FieldKind::ManyToOne | FieldKind::OneToMany | FieldKind::ManyToMany
        )
    }
}

/// 字段元数据。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldMetadata {
    rust_name: &'static str,
    column_name: &'static str,
    kind: FieldKind,
}

impl FieldMetadata {
    /// 创建字段元数据。
    pub const fn new(rust_name: &'static str, column_name: &'static str, kind: FieldKind) -> Self {
        Self {
            rust_name,
            column_name,
            kind,
        }
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
}
