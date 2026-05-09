//! Generic model traits for database entities, pagination, and auditing.
//!
//! Provides reusable trait abstractions that map to common ORM patterns:
//!
//! - [`Identifiable`] — entities with a primary key
//! - [`Timestamped`] — entities with created/updated timestamps
//! - [`SoftDeletable`] — entities that support soft deletion
//! - [`Auditable`] — entities that track who created/updated them
//! - [`Pageable`] — pagination request parameters
//! - [`PageResult`] — paginated response container

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An entity with a primary key of type `Id`.
pub trait Identifiable {
    /// The type of the identifier.
    type Id;

    /// Returns a reference to the entity's identifier.
    fn id(&self) -> &Self::Id;
}

/// An entity that tracks creation and last-updated timestamps.
pub trait Timestamped {
    /// Returns the creation timestamp, if known.
    fn created_at(&self) -> Option<DateTime<Utc>>;

    /// Returns the last-updated timestamp, if known.
    fn updated_at(&self) -> Option<DateTime<Utc>>;
}

/// An entity that supports soft deletion via a `deleted_at` timestamp.
pub trait SoftDeletable {
    /// Returns the deletion timestamp, or `None` if not deleted.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Returns `true` if this entity has been soft-deleted.
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// An entity that tracks who created and last updated it.
pub trait Auditable: Timestamped {
    /// Returns the identifier of the user who created this entity.
    fn created_by(&self) -> Option<&str>;

    /// Returns the identifier of the user who last updated this entity.
    fn updated_by(&self) -> Option<&str>;
}

/// Pagination request parameters.
pub trait Pageable {
    /// The current page number (1-indexed).
    fn page(&self) -> usize;

    /// The number of items per page.
    fn page_size(&self) -> usize;

    /// Computes the zero-based offset for database queries.
    fn offset(&self) -> usize {
        (self.page().saturating_sub(1)) * self.page_size()
    }
}

/// A paginated response container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageResult<T> {
    /// The items on the current page.
    pub items: Vec<T>,
    /// Total number of items across all pages.
    pub total: u64,
    /// Current page number (1-indexed).
    pub page: usize,
    /// Number of items per page.
    pub page_size: usize,
}

impl<T> PageResult<T> {
    /// Creates a new `PageResult`.
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, page: usize, page_size: usize) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
        }
    }

    /// Creates an empty `PageResult` for the given page.
    #[must_use]
    pub fn empty(page: usize, page_size: usize) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page,
            page_size,
        }
    }

    /// Computes the total number of pages.
    #[must_use]
    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            return 0;
        }
        (self.total as usize).div_ceil(self.page_size)
    }

    /// Returns `true` if there is a next page.
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    /// Returns `true` if there is a previous page.
    #[must_use]
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    /// Returns the number of items on this page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if this page is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test types ---

    #[derive(Debug, Clone)]
    struct User {
        id: u64,
        name: String,
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

    #[derive(Debug)]
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
            name: format!("user_{id}"),
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
            name: "no_time".into(),
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
            name: "anon".into(),
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
