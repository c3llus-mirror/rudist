use crate::utils::error::{Result, RedisError};
use std::time::SystemTime;
use std::fmt;
use crate::storage::data_types::string::RedisString;

pub mod memory;
pub mod data_types;
pub mod eviction;
// pub mod expiration;

#[derive(Debug,Clone)]
pub struct StorageEntry {
    pub data: StorageValue,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug,Clone)]
pub enum StorageValue {
    String(String),
    List(Vec<String>),
    // ...
}

pub trait Storage {
    fn set(&mut self, key: String, value: StorageValue, ttl: Option<SystemTime>) -> Result<()>;
    fn get(&mut self, key: &str) -> Result<&StorageEntry>;
    fn delete(&mut self, key: &str) -> Result<bool>;
    fn exists(&self, key: &str) -> Result<bool>;
    fn clear(&mut self) -> Result<()>;
    fn incr(&mut self, key: &str) -> Result<i64>;
    fn decr(&mut self, key: &str) -> Result<i64>;
    fn expire(&mut self, key: &str, ttl: u64) -> Result<()>;
    fn append(&mut self, key: &str, value: &str) -> Result<String>;
}

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, StorageValue, Option<SystemTime>),
    Del(String),
    Exists(String),
    Expire(String, u64),
    Incr(String),
    Decr(String),
    Append(String, String),
    FlushDB,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Get(key) => write!(f, "GET {}", key),
            Command::Set(key, _, _) => write!(f, "SET {}", key),
            Command::Del(key) => write!(f, "DEL {}", key),
            Command::Exists(key) => write!(f, "EXISTS {}", key),
            Command::FlushDB => write!(f, "CLEAR"),
            Command::Expire(key, ttl) => write!(f, "EXPIRE {} {}", key, ttl),
            Command::Incr(key) => write!(f, "INCR {}", key),
            Command::Decr(key) => write!(f, "DECR {}", key),
            Command::Append(key, value) => write!(f, "APPEND {} {}", key, value),
        }
    }
}

