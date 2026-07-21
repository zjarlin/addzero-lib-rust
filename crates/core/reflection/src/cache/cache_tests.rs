use crate::cache::ExpiringCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn compute_if_absent_inserts_and_returns_value() {
    let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
    let result = cache.compute_if_absent("key1", |_| 42).unwrap();

    assert_eq!(result, 42);
    assert_eq!(cache.len().unwrap(), 1);
}

#[test]
fn compute_if_absent_returns_cached_value() {
    let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
    cache.compute_if_absent("key1", |_| 42).unwrap();
    let result = cache.compute_if_absent("key1", |_| 99).unwrap();

    assert_eq!(result, 42);
}

#[test]
fn expired_entry_is_replaced() {
    let cache = ExpiringCache::new(Duration::from_millis(1), NonZeroUsize::new(10).unwrap());
    cache.compute_if_absent("key1", |_| "old").unwrap();
    std::thread::sleep(Duration::from_millis(10));
    let result = cache.compute_if_absent("key1", |_| "new").unwrap();

    assert_eq!(result, "new");
}

#[test]
fn cache_prunes_oldest_entry_when_max_size_is_reached() {
    let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(2).unwrap());
    cache.compute_if_absent("a", |_| 1).unwrap();
    std::thread::sleep(Duration::from_millis(5));
    cache.compute_if_absent("b", |_| 2).unwrap();
    std::thread::sleep(Duration::from_millis(5));
    cache.compute_if_absent("c", |_| 3).unwrap();

    assert_eq!(cache.len().unwrap(), 2);
}

#[test]
fn cleanup_expired_removes_ttl_elapsed_entries() {
    let cache = ExpiringCache::new(Duration::from_millis(1), NonZeroUsize::new(10).unwrap());
    cache.compute_if_absent("key1", |_| 1).unwrap();
    std::thread::sleep(Duration::from_millis(10));
    cache.cleanup_expired().unwrap();

    assert_eq!(cache.len().unwrap(), 0);
}

#[test]
fn clear_removes_all_entries() {
    let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
    cache.compute_if_absent("a", |_| 1).unwrap();
    cache.compute_if_absent("b", |_| 2).unwrap();
    cache.clear().unwrap();

    assert!(cache.is_empty().unwrap());
}

#[test]
fn concurrent_access_populates_cache_without_lost_entries() {
    let cache = Arc::new(ExpiringCache::new(
        Duration::from_secs(60),
        NonZeroUsize::new(100).unwrap(),
    ));

    let mut handles = Vec::new();
    for i in 0..10 {
        let cache = Arc::clone(&cache);
        handles.push(std::thread::spawn(move || {
            cache.compute_if_absent(i, |_| i * 2).unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(cache.len().unwrap(), 10);
}
