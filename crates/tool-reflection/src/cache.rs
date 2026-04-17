use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
}

#[derive(Debug)]
pub struct ExpiringCache<K, V> {
    expire_after: Duration,
    max_size: usize,
    entries: Mutex<HashMap<K, CacheEntry<V>>>,
}

impl<K, V> ExpiringCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(expire_after: Duration, max_size: usize) -> Self {
        assert!(max_size > 0, "max_size must be greater than zero");
        Self {
            expire_after,
            max_size,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn compute_if_absent<F>(&self, key: K, mapping: F) -> V
    where
        F: FnOnce(&K) -> V,
    {
        {
            let entries = self
                .entries
                .lock()
                .expect("cache mutex should not be poisoned");
            if let Some(entry) = entries.get(&key).filter(|entry| !self.is_expired(entry)) {
                return entry.value.clone();
            }
        }

        let value = mapping(&key);
        let mut entries = self
            .entries
            .lock()
            .expect("cache mutex should not be poisoned");

        if let Some(entry) = entries.get(&key).filter(|entry| !self.is_expired(entry)) {
            return entry.value.clone();
        }

        self.prune_if_needed(&mut entries);
        entries.insert(
            key,
            CacheEntry {
                value: value.clone(),
                created_at: Instant::now(),
            },
        );
        value
    }

    pub fn cleanup_expired(&self) {
        let mut entries = self
            .entries
            .lock()
            .expect("cache mutex should not be poisoned");
        let now = Instant::now();
        entries.retain(|_, entry| now.duration_since(entry.created_at) < self.expire_after);
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("cache mutex should not be poisoned")
            .clear();
    }

    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .expect("cache mutex should not be poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_expired(&self, entry: &CacheEntry<V>) -> bool {
        entry.created_at.elapsed() >= self.expire_after
    }

    fn prune_if_needed(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if entries.len() < self.max_size {
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
}
