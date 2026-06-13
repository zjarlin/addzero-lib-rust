use az_derive_aliases::{apply, plain_clone_debug, plain_debug};
use az_model::{Auditable, Identifiable, PageResult, Pageable, SoftDeletable, Timestamped};
use chrono::{DateTime, Utc};

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

#[test]
fn identifiable_returns_entity_id() {
    let user = make_user(42);
    assert_eq!(*user.id(), 42);
}

#[test]
fn timestamped_allows_missing_or_present_timestamps() {
    let user = make_user(1);
    assert!(user.created_at().is_some());
    assert!(user.updated_at().is_some());

    let user_without_timestamps = User {
        id: 2,
        created_at: None,
        updated_at: None,
        deleted_at: None,
        created_by: None,
        updated_by: None,
    };
    assert!(user_without_timestamps.created_at().is_none());
}

#[test]
fn soft_deletable_reports_not_deleted_when_timestamp_missing() {
    let user = make_user(1);
    assert!(!user.is_deleted());
    assert!(user.deleted_at().is_none());
}

#[test]
fn soft_deletable_reports_deleted_when_timestamp_exists() {
    let mut user = make_user(1);
    user.deleted_at = Some(Utc::now());
    assert!(user.is_deleted());
    assert!(user.deleted_at().is_some());
}

#[test]
fn auditable_returns_creator_and_updater() {
    let user = make_user(1);
    assert_eq!(user.created_by(), Some("admin"));
    assert_eq!(user.updated_by(), Some("admin"));
}

#[test]
fn auditable_allows_missing_actor_ids() {
    let user = User {
        id: 3,
        created_at: None,
        updated_at: None,
        deleted_at: None,
        created_by: None,
        updated_by: None,
    };
    assert!(user.created_by().is_none());
    assert!(user.updated_by().is_none());
}

#[test]
fn pageable_offset_uses_one_based_pages() {
    let first_page = Query {
        page: 1,
        page_size: 10,
    };
    assert_eq!(first_page.offset(), 0);

    let third_page = Query {
        page: 3,
        page_size: 20,
    };
    assert_eq!(third_page.offset(), 40);
}

#[test]
fn pageable_offset_saturates_page_zero() {
    let query = Query {
        page: 0,
        page_size: 10,
    };

    // Saturating subtraction prevents underflow for invalid page zero.
    assert_eq!(query.offset(), 0);
}

#[test]
fn page_result_reports_page_state() {
    let page = PageResult::new(vec![1, 2, 3], 100, 1, 10);
    assert_eq!(page.len(), 3);
    assert!(!page.is_empty());
    assert_eq!(page.total_pages(), 10);
    assert!(page.has_next());
    assert!(!page.has_prev());
}

#[test]
fn page_result_last_page_has_previous_without_next() {
    let page = PageResult::new(vec![1, 2], 22, 3, 10);
    assert_eq!(page.total_pages(), 3);
    assert!(!page.has_next());
    assert!(page.has_prev());
}

#[test]
fn page_result_empty_has_no_navigation() {
    let page: PageResult<i32> = PageResult::empty(1, 10);
    assert_eq!(page.len(), 0);
    assert!(page.is_empty());
    assert_eq!(page.total_pages(), 0);
    assert!(!page.has_next());
    assert!(!page.has_prev());
}

#[test]
fn page_result_total_pages_handles_exact_multiple() {
    let page = PageResult::new(vec![1, 2, 3, 4, 5], 20, 1, 5);
    assert_eq!(page.total_pages(), 4);
}

#[test]
fn page_result_zero_page_size_has_zero_total_pages() {
    let page: PageResult<i32> = PageResult::new(vec![], 100, 1, 0);
    assert_eq!(page.total_pages(), 0);
}
