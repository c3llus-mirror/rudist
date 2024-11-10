// src/engine/router.rs
use crate::storage::{Command, StorageValue};
use crate::utils::error::Result as CustomResult;
use std::time::{SystemTime, Duration};

use crate::protocol::resp::types::RESPType;

pub struct Router;

impl Router {
    pub fn new() -> Self {
        Self
    }

    pub fn route(&self, resp: &RESPType) -> std::result::Result<Command, String> {
        match resp {
            RESPType::Array(parts) if !parts.is_empty() => {
                let cmd_name = match &parts[0] {
                    RESPType::BulkString(Some(bytes)) => {
                        String::from_utf8(bytes.clone())
                            .map_err(|_| "Invalid UTF-8 in command")?
                            .to_uppercase()
                    },
                    _ => return Err("First array element must be a bulk string".into())
                };
                
                match cmd_name.as_str() {
                    "GET" if parts.len() == 2 => {
                        let key = parts[1].as_bytes()?;
                        Ok(Command::Get(String::from_utf8(key.to_vec())
                            .map_err(|_| "Invalid UTF-8 in key")?))
                    },
                    "SET" if parts.len() >= 3 => {
                        let key = parts[1].as_bytes()?;
                        let value = parts[2].as_bytes()?;
                        let ttl = if parts.len() == 5 {
                            match &parts[3] {
                                RESPType::BulkString(Some(bytes)) if bytes.to_ascii_uppercase() == b"PX" => {
                                    match &parts[4] {
                                        RESPType::BulkString(Some(ms_bytes)) => {
                                            let ms = String::from_utf8(ms_bytes.clone())
                                                .map_err(|_| "Invalid UTF-8 in TTL")?
                                                .parse::<u64>()
                                                .map_err(|_| "Invalid TTL value")?;
                                            Some(SystemTime::now() + Duration::from_millis(ms))
                                        },
                                        _ => return Err("Invalid TTL format".into())
                                    }
                                },
                                _ => return Err("Expected PX for TTL".into())
                            }
                        } else {
                            None
                        };

                        Ok(Command::Set(
                            String::from_utf8(key.to_vec()).map_err(|_| "Invalid UTF-8 in key")?,
                            StorageValue::String(
                                String::from_utf8(value.to_vec()).map_err(|_| "Invalid UTF-8 in value")?
                            ),
                            ttl
                        ))
                    },
                    "DEL" if parts.len() == 2 => {
                        let key = parts[1].as_bytes()?;
                        Ok(Command::Del(String::from_utf8(key.to_vec())
                            .map_err(|_| "Invalid UTF-8 in key")?))
                    },
                    "EXISTS" if parts.len() == 2 => {
                        let key = parts[1].as_bytes()?;
                        Ok(Command::Exists(String::from_utf8(key.to_vec())
                            .map_err(|_| "Invalid UTF-8 in key")?))
                    },
                    _ => Err("Unknown command or wrong number of arguments".into())
                }
            },
            _ => Err("Expected RESP array".into())
        }
    }
}
