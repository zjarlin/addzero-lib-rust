use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use az_derive_aliases::{apply, plain_debug};

#[apply(plain_debug)]
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
/// recovering poisoned locks instead of panicking.
#[apply(plain_debug)]
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
    /// Returns an error if cache access fails.
    pub fn compute_if_absent<F>(&self, key: K, mapping: F) -> anyhow::Result<V>
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
    /// Returns an error if cache access fails.
    pub fn cleanup_expired(&self) -> anyhow::Result<()> {
        let mut entries = self.lock_entries()?;
        let now = Instant::now();
        entries.retain(|_, entry| now.duration_since(entry.created_at) < self.expire_after);
        Ok(())
    }

    /// Removes all entries from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if cache access fails.
    pub fn clear(&self) -> anyhow::Result<()> {
        self.lock_entries()?.clear();
        Ok(())
    }

    /// Returns the number of entries currently in the cache (including expired
    /// ones that have not yet been cleaned up).
    ///
    /// # Errors
    ///
    /// Returns an error if cache access fails.
    pub fn len(&self) -> anyhow::Result<usize> {
        Ok(self.lock_entries()?.len())
    }

    /// Returns `true` if the cache contains no entries.
    ///
    /// # Errors
    ///
    /// Returns an error if cache access fails.
    pub fn is_empty(&self) -> anyhow::Result<bool> {
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
    fn lock_entries(&self) -> anyhow::Result<std::sync::MutexGuard<'_, HashMap<K, CacheEntry<V>>>> {
        match self.entries.lock() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => Ok(poisoned.into_inner()),
        }
    }
}

#[cfg(test)]
mod cache_tests;
