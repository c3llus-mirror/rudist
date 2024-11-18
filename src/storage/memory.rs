use std::collections::HashMap;
use std::time::{SystemTime, Instant, Duration};
use rand::seq::SliceRandom;
use super::{Storage, StorageEntry, StorageValue};
use crate::utils::error::Result;
use crate::utils::error::RedisError;
use std::thread;


// TODO: should we make this configurable?
const ACTIVE_EXPIRE_CYCLE_LOOKUPS_PER_LOOP: usize = 20;  // how many keys to sample per loop
const ACTIVE_EXPIRE_CYCLE_FAST_DURATION: Duration = Duration::from_millis(1);  // fast cycle duration
const ACTIVE_EXPIRE_CYCLE_SLOW_DURATION: Duration = Duration::from_millis(25);  // slow cycle duration
const ACTIVE_EXPIRE_CYCLE_THRESHOLD: f64 = 0.25;  // stop sampling if hit rate drops below 25%

#[derive(Debug)]
pub struct MemoryStorage {
    data: HashMap<String, StorageEntry>,
    max_memory: usize,
    used_memory: usize,
    last_expire_cycle: Instant,
}

impl MemoryStorage {
    pub fn new(max_memory: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_memory,
            used_memory: 0,
            last_expire_cycle: Instant::now(),
        }
    }

    // redis-style active expiration cycle
    pub fn active_expire_cycle(&mut self, cycle_type: ExpireCycleType) -> ExpireStats {
        let mut stats = ExpireStats::default();
        let start = Instant::now();
        let max_duration = match cycle_type {
            ExpireCycleType::Fast => ACTIVE_EXPIRE_CYCLE_FAST_DURATION,
            ExpireCycleType::Slow => ACTIVE_EXPIRE_CYCLE_SLOW_DURATION,
        };

        // get all keys for sampling
        let keys: Vec<String> = self.data.keys().cloned().collect();
        let mut rng = rand::thread_rng();

        // while still within cycle duration
        while start.elapsed() < max_duration {
            stats.total_cycles += 1;
            let mut expired_in_cycle = 0;

            // sample random keys
            for _ in 0..ACTIVE_EXPIRE_CYCLE_LOOKUPS_PER_LOOP {
                if let Some(key) = keys.choose(&mut rng) {
                    stats.keys_checked += 1;
                    
                    if let Some(entry) = self.data.get(key) {
                        if let Some(expiry_time) = entry.expires_at {
                            if expiry_time < SystemTime::now() {
                                if let Some(entry) = self.data.remove(key) {
                                    self.used_memory -= Self::estimate_size(&entry.data);
                                    expired_in_cycle += 1;
                                    stats.keys_expired += 1;
                                }
                            }
                        }
                    }
                }
            }

            // calculate hit rate for this cycle
            let hit_rate = expired_in_cycle as f64 / ACTIVE_EXPIRE_CYCLE_LOOKUPS_PER_LOOP as f64;
            
            // stop if hit rate is too low (redis behavior)
            if hit_rate < ACTIVE_EXPIRE_CYCLE_THRESHOLD {
                stats.stopped_by_threshold = true;
                break;
            }
        }

        self.last_expire_cycle = Instant::now();
        stats.duration = start.elapsed();
        stats
    }

    // passive expiration check (called during get operations)
    fn check_expiry(&mut self, key: &str) -> bool {
        if let Some(entry) = self.data.get(key) {
            if let Some(expiry_time) = entry.expires_at {
                if expiry_time < SystemTime::now() {
                    return true;
                }
            }
        }
        false
    }

    fn estimate_size(value: &StorageValue) -> usize {
        match value {
            StorageValue::String(s) => s.len(),
            StorageValue::List(l) => l.iter().map(|s| s.len()).sum(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.max_memory 
    }

    fn is_expired(&self, key: &str) -> bool {
        if let Some(entry) = self.data.get(key) {
            if let Some(expiry_time) = entry.expires_at {
                return expiry_time < SystemTime::now();
            }
        }
        false
    }

    // separate function to handle lazy deletion
    fn lazy_delete(&mut self, key: &str) -> Result<()> {
        if let Some(entry) = self.data.remove(key) {
            self.used_memory -= Self::estimate_size(&entry.data);
        }
        Ok(())
    }
    
}

#[derive(Debug)]
pub enum ExpireCycleType {
    Fast,  // quick cycle for event loop
    Slow,  // more thorough cycle for maintenance
}

#[derive(Debug, Default)]
pub struct ExpireStats {
    pub keys_checked: usize,
    pub keys_expired: usize,
    pub total_cycles: usize,
    pub stopped_by_threshold: bool,
    pub duration: Duration,
}

impl Storage for MemoryStorage {
    fn get(&mut self, key: &str) -> Result<&StorageEntry> {
        // passive expiration
        if self.check_expiry(key) {
            self.delete(key)?;
            return Err(RedisError::KeyNotFound);
        }

        self.data.get(key).ok_or(RedisError::KeyNotFound)
    }

    fn set(&mut self, key: String, value: StorageValue, ttl: Option<SystemTime>) -> Result<()> {
        let size = Self::estimate_size(&value);
        
        // if key exists, subtract its size first
        if let Some(old_entry) = self.data.get(&key) {
            self.used_memory -= Self::estimate_size(&old_entry.data);
        }

        if self.used_memory + size > self.max_memory {
            return Err(RedisError::OutOfMemory);
        }

        let entry = StorageEntry {
            data: value,
            expires_at: ttl,
        };
        
        self.used_memory += size;
        self.data.insert(key, entry);
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<bool> {
        if let Some(entry) = self.data.remove(key) {
            self.used_memory -= Self::estimate_size(&entry.data);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.data.contains_key(key))
    }

    fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.used_memory = 0;
        Ok(())
    }

    fn expire(&mut self, key: &str, ttl: u64) -> Result<()> {
        if let Some(entry) = self.data.get_mut(key) {
            entry.expires_at = Some(SystemTime::now() + Duration::from_secs(ttl));
            Ok(())
        } else {
            Err(RedisError::KeyNotFound)
        }
    }

    fn incr(&mut self, key: &str) -> Result<i64> {
        let value = match self.data.get_mut(key) {
            Some(entry) => {
                if let StorageValue::String(ref mut s) = entry.data {
                    match s.parse::<i64>() {
                        Ok(mut num) => {
                            num += 1;
                            *s = num.to_string();
                            num
                        }
                        Err(_) => return Err(RedisError::NotInteger),
                    }
                } else {
                    return Err(RedisError::NotInteger);
                }
            }
            None => {
                let entry = StorageEntry {
                    data: StorageValue::String("1".to_string()),
                    expires_at: None,
                };
                self.data.insert(key.to_string(), entry);
                1
            }
        };
        Ok(value)
    }

    fn decr(&mut self, key: &str) -> Result<i64> {
        let value = match self.data.get_mut(key) {
            Some(entry) => {
                if let StorageValue::String(ref mut s) = entry.data {
                    match s.parse::<i64>() {
                        Ok(mut num) => {
                            num -= 1;
                            *s = num.to_string();
                            num
                        }
                        Err(_) => return Err(RedisError::NotInteger),
                    }
                } else {
                    return Err(RedisError::NotInteger);
                }
            }
            None => {
                let entry = StorageEntry {
                    data: StorageValue::String("-1".to_string()),
                    expires_at: None,
                };
                self.data.insert(key.to_string(), entry);
                -1
            }
        };
        Ok(value)
    }

    fn append(&mut self, key: &str, value: &str) -> Result<String> {
        let new_value = match self.data.get_mut(key) {
            Some(entry) => {
                if let StorageValue::String(ref mut s) = entry.data {
                    s.push_str(value);
                    s.clone()
                } else {
                    return Err(RedisError::WrongType);
                }
            }
            None => {
                let entry = StorageEntry {
                    data: StorageValue::String(value.to_string()),
                    expires_at: None,
                };
                self.data.insert(key.to_string(), entry);
                value.to_string()
            }
        };
        Ok(new_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passive_expiration() {
        let mut storage = MemoryStorage::new(1024);
        let now = SystemTime::now();
        let ttl = now + Duration::from_millis(1);
        
        // set value with 1 second ttl
        storage.set("key1".to_string(), 
            StorageValue::String("value1".to_string()), 
            Some(ttl)).unwrap();

        // value should exist initially
        assert!(storage.exists("key1").unwrap());
        
        // wait for ttl to expire
        thread::sleep(Duration::from_millis(5));
        
        // value should be gone after expiry
        assert!(storage.get("key1").is_err());
    }

    #[test]
    fn test_active_expiration() {
        let mut storage = MemoryStorage::new(1024);
        let now = SystemTime::now();
        let ttl = now + Duration::from_millis(5);

        // add multiple entries with ttl
        for i in 0..50 {
            storage.set(
                format!("key{}", i),
                StorageValue::String(format!("value{}", i)),
                Some(ttl)
            ).unwrap();
        }

        // wait for ttl
        thread::sleep(Duration::from_millis(10));

        // run expiration cycle
        let stats = storage.active_expire_cycle(ExpireCycleType::Slow);
        
        assert!(stats.keys_expired > 0);
        assert!(stats.keys_checked > 0);
    }

    #[test]
        fn test_memory_limits() {
            let mut storage = MemoryStorage::new(10);
            
            // should succeed - within limits
            assert!(storage.set(
                "key1".to_string(),
                StorageValue::String("123".to_string()),
                None
            ).is_ok());

            // should fail - exceeds memory limit
            assert!(storage.set(
                "key2".to_string(), 
                StorageValue::String("very long string".to_string()),
                None
            ).is_err());
        }

        #[test]
        fn test_update_memory_usage() {
            let mut storage = MemoryStorage::new(100);
            
            storage.set(
                "key1".to_string(),
                StorageValue::String("short".to_string()),
                None
            ).unwrap();
            let initial_memory = storage.used_memory;

            storage.set(
                "key1".to_string(),
                StorageValue::String("longer string".to_string()),
                None
            ).unwrap();
            
            assert!(storage.used_memory > initial_memory);

            storage.set(
                "key1".to_string(),
                StorageValue::String("tiny".to_string()),
                None
            ).unwrap();

            assert!(storage.used_memory < initial_memory);
        }

        #[test]
        fn test_delete_updates_memory() {
            let mut storage = MemoryStorage::new(100);
            
            storage.set(
                "key1".to_string(),
                StorageValue::String("test value".to_string()),
                None
            ).unwrap();
            
            let pre_delete_memory = storage.used_memory;
            storage.delete("key1").unwrap();
            
            assert_eq!(storage.used_memory, 0);
            assert!(storage.used_memory < pre_delete_memory);
        }

        #[test]
        fn test_expire_cleanup() {
            let mut storage = MemoryStorage::new(100);
            let now = SystemTime::now();
            
            // add expired entry
            storage.set(
                "expired".to_string(),
                StorageValue::String("value".to_string()),
                Some(now)
            ).unwrap();

            // add non-expired entry
            storage.set(
                "valid".to_string(),
                StorageValue::String("value".to_string()),
                Some(now + Duration::from_secs(30))
            ).unwrap();

            assert!(storage.get("expired").is_err());
            assert!(storage.get("valid").is_ok());
        }

        #[test]
        fn test_clear(){
            let mut storage = MemoryStorage::new(100);
            
            storage.set(
                "key1".to_string(),
                StorageValue::String("value1".to_string()),
                None
            ).unwrap();
            
            storage.set(
                "key2".to_string(),
                StorageValue::String("value2".to_string()),
                None
            ).unwrap();
            
            storage.clear().unwrap();
            
            assert!(storage.get("key1").is_err());
            assert!(storage.get("key2").is_err());
        }
}
