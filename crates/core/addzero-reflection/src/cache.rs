use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Errors that can occur when constructing or operating on an [`ExpiringCache`].
#[derive(Debug)]
pub enum CacheError {
    /// A poisoned mutex was encountered; the cache could not be locked.
    /// This happens when another thread panicked while holding the lock.
    Poisoned,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Poisoned => write!(f, "cache mutex is poisoned"),
        }
    }
}

impl std::error::Error for CacheError {}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
}

/// A thread-safe in-memory cache with per-entry time-to-live expiration.
///
/// Entries are automatically pruned when the cache reaches its maximum size,
/// removing the oldest entry to make room. Expired entries can be cleaned up
/// with [`cleanup_expired`](ExpiringCache::cleanup_expired).
///
/// # Thread Safety
///
/// Internally uses a [`Mutex`] and recovers from poisoned locks gracefully,
/// returning [`CacheError::Poisoned`] instead of panicking.
#[derive(Debug)]
pub struct ExpiringCache<K, V> {
    expire_after: Duration,
    max_size: NonZeroUsize,
    entries: Mutex<HashMap<K, CacheEntry<V>>>,
}

impl<K, V> ExpiringCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new cache with the given expiration duration and maximum number
    /// of entries.
    ///
    /// Uses [`NonZeroUsize`] for `max_size` to guarantee at compile time that
    /// the capacity is always positive.
    pub fn new(expire_after: Duration, max_size: NonZeroUsize) -> Self {
        Self {
            expire_after,
            max_size,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the value associated with `key` if it exists and has not expired.
    /// Otherwise, calls `mapping` to compute the value, stores it, and returns it.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Poisoned`] if the internal mutex is poisoned.
    pub fn compute_if_absent<F>(&self, key: K, mapping: F) -> Result<V, CacheError>
    where
        F: FnOnce(&K) -> V,
    {
        {
            let entries = self.lock_entries()?;
            if let Some(entry) = entries.get(&key).filter(|entry| !self.is_expired(entry)) {
                return Ok(entry.value.clone());
            }
        }

        let value = mapping(&key);
        let mut entries = self.lock_entries()?;

        if let Some(entry) = entries.get(&key).filter(|entry| !self.is_expired(entry)) {
            return Ok(entry.value.clone());
        }

        self.prune_if_needed(&mut entries);
        entries.insert(
            key,
            CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
            },
        );
        Ok(value)
    }

    /// Removes all entries whose TTL has elapsed.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Poisoned`] if the internal mutex is poisoned.
    pub fn cleanup_expired(&self) -> Result<(), CacheError> {
        let mut entries = self.lock_entries()?;
        let now = Instant::now();
        entries.retain(|_, entry| now.duration_since(entry.created_at) < self.expire_after);
        Ok(())
    }

    /// Removes all entries from the cache.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Poisoned`] if the internal mutex is poisoned.
    pub fn clear(&self) -> Result<(), CacheError> {
        self.lock_entries()?.clear();
        Ok(())
    }

    /// Returns the number of entries currently in the cache (including expired
    /// ones that have not yet been cleaned up).
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Poisoned`] if the internal mutex is poisoned.
    pub fn len(&self) -> Result<usize, CacheError> {
        Ok(self.lock_entries()?.len())
    }

    /// Returns `true` if the cache contains no entries.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Poisoned`] if the internal mutex is poisoned.
    pub fn is_empty(&self) -> Result<bool, CacheError> {
        Ok(self.len()? == 0)
    }

    fn is_expired(&self, entry: &CacheEntry<V>) -> bool {
        entry.created_at.elapsed() >= self.expire_after
    }

    fn prune_if_needed(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if entries.len() < self.max_size.get() {
            return;
        }

        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone())
        {
            entries.remove(&oldest_key);
        }
    }

    /// Acquires the entries lock, recovering from poisoned mutexes gracefully.
    fn lock_entries(&self) -> Result<std::sync::MutexGuard<'_, HashMap<K, CacheEntry<V>>>, CacheError> {
        match self.entries.lock() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => Ok(poisoned.into_inner()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_compute_if_absent_inserts_and_returns() {
        let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
        let result = cache.compute_if_absent("key1", |_| 42).unwrap();
        assert_eq!(result, 42);
        assert_eq!(cache.len().unwrap(), 1);
    }

    #[test]
    fn test_compute_if_absent_returns_cached_value() {
        let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
        cache.compute_if_absent("key1", |_| 42).unwrap();
        let result = cache.compute_if_absent("key1", |_| 99).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_expired_entry_is_replaced() {
        let cache = ExpiringCache::new(Duration::from_millis(1), NonZeroUsize::new(10).unwrap());
        cache.compute_if_absent("key1", |_| "old").unwrap();
        std::thread::sleep(Duration::from_millis(10));
        let result = cache.compute_if_absent("key1", |_| "new").unwrap();
        assert_eq!(result, "new");
    }

    #[test]
    fn test_prune_on_max_size() {
        let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(2).unwrap());
        cache.compute_if_absent("a", |_| 1).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        cache.compute_if_absent("b", |_| 2).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        cache.compute_if_absent("c", |_| 3).unwrap();
        assert_eq!(cache.len().unwrap(), 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = ExpiringCache::new(Duration::from_millis(1), NonZeroUsize::new(10).unwrap());
        cache.compute_if_absent("key1", |_| 1).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        cache.cleanup_expired().unwrap();
        assert_eq!(cache.len().unwrap(), 0);
    }

    #[test]
    fn test_clear() {
        let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(10).unwrap());
        cache.compute_if_absent("a", |_| 1).unwrap();
        cache.compute_if_absent("b", |_| 2).unwrap();
        cache.clear().unwrap();
        assert!(cache.is_empty().unwrap());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;

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
}
