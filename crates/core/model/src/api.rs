//! 数据库实体、分页和审计的通用模型 trait。
//!
//! 提供可复用的 trait 抽象，对应常见的 ORM 模式：
//!
//! - [`Identifiable`] — 具有主键的实体
//! - [`Timestamped`] — 具有创建/更新时间戳的实体
//! - [`SoftDeletable`] — 支持软删除的实体
//! - [`Auditable`] — 追踪创建/更新者的实体
//! - [`Pageable`] — 分页请求参数
//! - [`PageResult`] — 分页响应容器

use chrono::{DateTime, Utc};

/// 带主键的实体抽象。
pub trait Identifiable {
    /// 主键的类型。
    type Id;

    /// 返回实体主键引用。
    fn id(&self) -> &Self::Id;
}

/// 记录创建时间和最后更新时间的实体抽象。
pub trait Timestamped {
    /// 返回创建时间；未知时返回 `None`。
    fn created_at(&self) -> Option<DateTime<Utc>>;

    /// 返回最后更新时间；未知时返回 `None`。
    fn updated_at(&self) -> Option<DateTime<Utc>>;
}

/// 通过 `deleted_at` 时间戳表达软删除状态的实体抽象。
pub trait SoftDeletable {
    /// 返回删除时间；未删除时返回 `None`。
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// 判断实体是否已被软删除。
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// 记录创建者和最后更新者的实体抽象。
pub trait Auditable: Timestamped {
    /// 返回创建者标识。
    fn created_by(&self) -> Option<&str>;

    /// 返回最后更新者标识。
    fn updated_by(&self) -> Option<&str>;
}

/// 分页请求参数抽象。
pub trait Pageable {
    /// 当前页码，从 1 开始。
    fn page(&self) -> usize;

    /// 每页条目数量。
    fn page_size(&self) -> usize;

    /// 计算面向数据库查询的零基偏移量。
    fn offset(&self) -> usize {
        (self.page().saturating_sub(1)) * self.page_size()
    }
}

/// 分页响应容器。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PageResult<T> {
    /// 当前页的条目列表。
    pub items: Vec<T>,
    /// 所有页面中的总条目数。
    pub total: u64,
    /// 当前页码，从 1 开始。
    pub page: usize,
    /// 每页条目数量。
    pub page_size: usize,
}

impl<T> PageResult<T> {
    /// 创建分页响应。
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, page: usize, page_size: usize) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
        }
    }

    /// 创建指定分页参数下的空分页响应。
    #[must_use]
    pub fn empty(page: usize, page_size: usize) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page,
            page_size,
        }
    }

    /// 计算总页数。
    ///
    /// `page_size` 为 0 时返回 0，避免无效分页参数导致除零。
    #[must_use]
    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            return 0;
        }
        (self.total as usize).div_ceil(self.page_size)
    }

    /// 判断当前页之后是否还有下一页。
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    /// 判断当前页之前是否还有上一页。
    #[must_use]
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    /// 返回当前页实际包含的条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 判断当前页是否没有任何条目。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
