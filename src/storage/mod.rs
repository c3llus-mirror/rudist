use crate::utils::error::{Result, RedisError};
use std::time::SystemTime;
use std::fmt;

pub mod memory;
pub mod data_types;
pub mod eviction;

#[derive(Debug)]
pub struct StorageEntry {
    pub data: StorageValue,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug)]
pub enum StorageValue {
    String(String),
    List(Vec<String>),
    // ...
}

pub trait Storage {
    fn set(&mut self, key: String, value: StorageValue, ttl: Option<SystemTime>) -> Result<()>;
    fn get(&self, key: &str) -> Result<&StorageEntry>;
    fn delete(&mut self, key: &str) -> Result<bool>;
    fn exists(&self, key: &str) -> Result<bool>;
    fn clear(&mut self) -> Result<()>;
}
#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, StorageValue, Option<SystemTime>),
    Del(String),
    Exists(String),
    Clear,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Get(key) => write!(f, "GET {}", key),
            Command::Set(key, _, _) => write!(f, "SET {}", key),
            Command::Del(key) => write!(f, "DEL {}", key),
            Command::Exists(key) => write!(f, "EXISTS {}", key),
            Command::Clear => write!(f, "CLEAR"),
        }
    }
}

