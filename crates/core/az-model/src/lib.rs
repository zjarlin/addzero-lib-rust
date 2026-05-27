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

use az_derive_aliases::{apply, serde_eq};
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
#[apply(serde_eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use az_derive_aliases::{apply, plain_clone_debug, plain_debug};

    // --- Test types ---

    #[apply(plain_clone_debug)]
    struct User {
        id: u64,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        deleted_at: Option<DateTime<Utc>>,
        created_by: Option<String>,
        updated_by: Option<String>,
    }

    impl Identifiable for User {
        type Id = u64;
        fn id(&self) -> &u64 {
            &self.id
        }
    }

    impl Timestamped for User {
        fn created_at(&self) -> Option<DateTime<Utc>> {
            self.created_at
        }
        fn updated_at(&self) -> Option<DateTime<Utc>> {
            self.updated_at
        }
    }

    impl SoftDeletable for User {
        fn deleted_at(&self) -> Option<DateTime<Utc>> {
            self.deleted_at
        }
    }

    impl Auditable for User {
        fn created_by(&self) -> Option<&str> {
            self.created_by.as_deref()
        }
        fn updated_by(&self) -> Option<&str> {
            self.updated_by.as_deref()
        }
    }

    #[apply(plain_debug)]
    struct Query {
        page: usize,
        page_size: usize,
    }

    impl Pageable for Query {
        fn page(&self) -> usize {
            self.page
        }
        fn page_size(&self) -> usize {
            self.page_size
        }
    }

    fn make_user(id: u64) -> User {
        User {
            id,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            deleted_at: None,
            created_by: Some("admin".to_owned()),
            updated_by: Some("admin".to_owned()),
        }
    }

    // --- Tests ---

    #[test]
    fn test_identifiable() {
        let u = make_user(42);
        assert_eq!(*u.id(), 42);
    }

    #[test]
    fn test_timestamped() {
        let u = make_user(1);
        assert!(u.created_at().is_some());
        assert!(u.updated_at().is_some());

        let u2 = User {
            id: 2,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: None,
            updated_by: None,
        };
        assert!(u2.created_at().is_none());
    }

    #[test]
    fn test_soft_deletable_not_deleted() {
        let u = make_user(1);
        assert!(!u.is_deleted());
        assert!(u.deleted_at().is_none());
    }

    #[test]
    fn test_soft_deletable_deleted() {
        let mut u = make_user(1);
        u.deleted_at = Some(Utc::now());
        assert!(u.is_deleted());
        assert!(u.deleted_at().is_some());
    }

    #[test]
    fn test_auditable() {
        let u = make_user(1);
        assert_eq!(u.created_by(), Some("admin"));
        assert_eq!(u.updated_by(), Some("admin"));
    }

    #[test]
    fn test_auditable_none() {
        let u = User {
            id: 3,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: None,
            updated_by: None,
        };
        assert!(u.created_by().is_none());
        assert!(u.updated_by().is_none());
    }

    #[test]
    fn test_pageable_offset() {
        let q = Query {
            page: 1,
            page_size: 10,
        };
        assert_eq!(q.offset(), 0);

        let q2 = Query {
            page: 3,
            page_size: 20,
        };
        assert_eq!(q2.offset(), 40);
    }

    #[test]
    fn test_pageable_offset_page_zero() {
        let q = Query {
            page: 0,
            page_size: 10,
        };
        // saturating_sub prevents underflow
        assert_eq!(q.offset(), 0);
    }

    #[test]
    fn test_page_result_basic() {
        let items = vec![1, 2, 3];
        let page = PageResult::new(items, 100, 1, 10);
        assert_eq!(page.len(), 3);
        assert!(!page.is_empty());
        assert_eq!(page.total_pages(), 10);
        assert!(page.has_next());
        assert!(!page.has_prev());
    }

    #[test]
    fn test_page_result_last_page() {
        let items = vec![1, 2];
        let page = PageResult::new(items, 22, 3, 10);
        // total = 22, page_size = 10 -> 3 pages
        assert_eq!(page.total_pages(), 3);
        assert!(!page.has_next());
        assert!(page.has_prev());
    }

    #[test]
    fn test_page_result_empty() {
        let page: PageResult<i32> = PageResult::empty(1, 10);
        assert_eq!(page.len(), 0);
        assert!(page.is_empty());
        assert_eq!(page.total_pages(), 0);
        assert!(!page.has_next());
        assert!(!page.has_prev());
    }

    #[test]
    fn test_page_result_exact_multiple() {
        let page = PageResult::new(vec![1, 2, 3, 4, 5], 20, 1, 5);
        assert_eq!(page.total_pages(), 4);
    }

    #[test]
    fn test_page_result_zero_page_size() {
        let page: PageResult<i32> = PageResult::new(vec![], 100, 1, 0);
        assert_eq!(page.total_pages(), 0);
    }
}
