// src/engine/executor.rs
use crate::storage::{memory::MemoryStorage, Storage, StorageValue, Command};
use crate::utils::error::Result;

pub struct Executor {
    storage: MemoryStorage,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            storage: MemoryStorage::new(1024 * 1024) // 1MB default
        }
    }

    pub fn execute(&mut self, command: Command) -> Result<String> {
        match command {
            Command::Get(key) => {
                let entry = self.storage.get(&key)?;
                Ok(match &entry.data {
                    StorageValue::String(s) => s.clone(),
                    StorageValue::List(l) => l.join(" ")
                })
            },
            Command::Set(key, value, ttl) => {
                self.storage.set(key, value, ttl)?;
                Ok("OK".to_string())
            },
            Command::Del(key) => {
                let deleted = self.storage.delete(&key)?;
                Ok(if deleted {
                    "OK".to_string()
                } else {
                    "Key not found".to_string()
                })
            },
            Command::Exists(key) => {
                let exists = self.storage.exists(&key)?;
                Ok(exists.to_string())
            },
            Command::Clear => {
                self.storage.clear()?;
                Ok("OK".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_executor() {
        let executor = Executor::new();
        assert!(executor.storage.capacity() == 1024 * 1024);
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut executor = Executor::new();
        let result = executor.execute(Command::Get("nonexistent".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_and_get_string() {
        let mut executor = Executor::new();
        let set_result = executor.execute(Command::Set(
            "key1".to_string(),
            StorageValue::String("value1".to_string()),
            None
        ));
        assert!(set_result.is_ok());
        assert_eq!(set_result.unwrap(), "OK");

        let get_result = executor.execute(Command::Get("key1".to_string()));
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), "value1");
    }

    #[test]
    fn test_set_and_get_list() {
        let mut executor = Executor::new();
        let set_result = executor.execute(Command::Set(
            "list1".to_string(),
            StorageValue::List(vec!["item1".to_string(), "item2".to_string()]),
            None
        ));
        assert!(set_result.is_ok());

        let get_result = executor.execute(Command::Get("list1".to_string()));
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), "item1 item2");
    }

    #[test]
    fn test_delete_existing_key() {
        let mut executor = Executor::new();
        executor.execute(Command::Set(
            "key1".to_string(),
            StorageValue::String("value1".to_string()),
            None
        )).unwrap();

        let del_result = executor.execute(Command::Del("key1".to_string()));
        assert!(del_result.is_ok());
        assert_eq!(del_result.unwrap(), "OK");
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let mut executor = Executor::new();
        let del_result = executor.execute(Command::Del("nonexistent".to_string()));
        assert!(del_result.is_ok());
        assert_eq!(del_result.unwrap(), "Key not found");
    }

    #[test]
    fn test_exists() {
        let mut executor = Executor::new();
        executor.execute(Command::Set(
            "key1".to_string(),
            StorageValue::String("value1".to_string()),
            None
        )).unwrap();

        let exists_result = executor.execute(Command::Exists("key1".to_string()));
        assert!(exists_result.is_ok());
        assert_eq!(exists_result.unwrap(), "true");

        let not_exists_result = executor.execute(Command::Exists("nonexistent".to_string()));
        assert!(not_exists_result.is_ok());
        assert_eq!(not_exists_result.unwrap(), "false");
    }

    #[test]
    fn test_clear() {
        let mut executor = Executor::new();
        executor.execute(Command::Set(
            "key1".to_string(),
            StorageValue::String("value1".to_string()),
            None
        )).unwrap();

        let clear_result = executor.execute(Command::Clear);
        assert!(clear_result.is_ok());
        assert_eq!(clear_result.unwrap(), "OK");

        let get_result = executor.execute(Command::Get("key1".to_string()));
        assert!(get_result.is_err());
    }

    #[test]
    fn test_set_with_ttl() {
        let mut executor = Executor::new();
        let set_result = executor.execute(Command::Set(
            "key1".to_string(),
            StorageValue::String("value1".to_string()),
            Some(std::time::SystemTime::now().checked_add(Duration::from_secs(1)).unwrap())
        ));
        assert!(set_result.is_ok());
        assert_eq!(set_result.unwrap(), "OK");
    }
}