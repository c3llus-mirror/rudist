use std::collections::HashMap;
use std::time::SystemTime;
use super::{Storage, StorageEntry, StorageValue};
use crate::utils::error::Result;
use crate::utils::error::RedisError;

#[derive(Debug)]
pub struct MemoryStorage {
    data: HashMap<String, StorageEntry>,
    max_memory: usize,
    used_memory: usize,
}

impl MemoryStorage {
    pub fn new(max_memory: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_memory,
            used_memory: 0,
        }
    }

    pub fn memory_usage(&self) -> usize {
        self.used_memory
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

}

impl Storage for MemoryStorage {
    fn set(&mut self, key: String, value: StorageValue, ttl: Option<SystemTime>) -> Result<()> {
        let size = Self::estimate_size(&value);
        
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
    
    fn get(&self, key: &str) -> Result<&StorageEntry> {
        let entry = self.data.get(key).ok_or(RedisError::KeyNotFound)?;
        
        if let Some(expires_at) = entry.expires_at {
            if SystemTime::now() > expires_at {
                return Err(RedisError::KeyNotFound);
            }
        }
        
        Ok(entry)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, Duration};
    use std::thread;

    #[test]
    fn test_new_storage() {
        let storage = MemoryStorage::new(1000);
        assert_eq!(storage.capacity(), 1000);
        assert_eq!(storage.memory_usage(), 0);
    }

    #[test]
    fn test_basic_operations() -> Result<()> {
        let mut storage = MemoryStorage::new(1000);
        
        storage.set("key1".to_string(), StorageValue::String("value1".to_string()), None)?;
        
        let entry = storage.get("key1")?;
        assert!(matches!(entry.data, StorageValue::String(ref s) if s == "value1"));
        
        assert!(storage.exists("key1")?);
        assert!(!storage.exists("nonexistent")?);
        
        assert!(storage.delete("key1")?);
        assert!(!storage.delete("nonexistent")?);
        
        Ok(())
    }

    #[test]
    fn test_memory_management() -> Result<()> {
        let mut storage = MemoryStorage::new(10);
        
        // should succeed (size = 5)
        storage.set("key1".to_string(), StorageValue::String("12345".to_string()), None)?;
        
        // should fail (size = 6)
        let result = storage.set("key2".to_string(), StorageValue::String("123456".to_string()), None);
        assert!(matches!(result, Err(RedisError::OutOfMemory)));
        
        // should succeed after deletion
        storage.delete("key1")?;
        storage.set("key2".to_string(), StorageValue::String("123".to_string()), None)?;
        
        Ok(())
    }

    #[test]
    fn test_ttl() -> Result<()> {
        let mut storage = MemoryStorage::new(1000);
        
        // set with TTL
        let ttl = SystemTime::now() + Duration::from_millis(1);
        storage.set("key1".to_string(), StorageValue::String("value1".to_string()), Some(ttl))?;
        
        // should exist immediately
        assert!(storage.get("key1").is_ok());
        
        // wait for expiration
        thread::sleep(Duration::from_millis(5));
        
        // should be expired
        assert!(matches!(storage.get("key1"), Err(RedisError::KeyNotFound)));
        
        Ok(())
    }

    #[test]
    fn test_list_operations() -> Result<()> {
        let mut storage = MemoryStorage::new(1000);
        
        let list = vec!["item1".to_string(), "item2".to_string()];
        storage.set("list1".to_string(), StorageValue::List(list.clone()), None)?;
        
        if let StorageValue::List(stored_list) = &storage.get("list1")?.data {
            assert_eq!(stored_list, &list);
        } else {
            panic!("Wrong type stored");
        }
        
        Ok(())
    }

    #[test]
    fn test_clear() -> Result<()> {
        let mut storage = MemoryStorage::new(1000);
        
        storage.set("key1".to_string(), StorageValue::String("value1".to_string()), None)?;
        storage.set("key2".to_string(), StorageValue::String("value2".to_string()), None)?;
        
        storage.clear()?;
        assert_eq!(storage.memory_usage(), 0);
        assert!(storage.get("key1").is_err());
        assert!(storage.get("key2").is_err());
        
        Ok(())
    }
}