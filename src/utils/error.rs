use std::fmt;
use std::io;
use std::result;

#[derive(Debug)]
pub enum RedisError {
    // Storage errors
    KeyNotFound,
    WrongType,
    NotInteger,
    OutOfMemory,
    
    // Protocol errors
    ParseError(String),
    InvalidCommand(String),
    InvalidArgumentCount { cmd: String, expected: usize, got: usize },
    
    // System errors
    IOError(io::Error),
    Internal(String),
}

pub type Result<T> = result::Result<T, RedisError>;

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisError::KeyNotFound => write!(f, "ERR no such key"),
            RedisError::WrongType => write!(f, "WRONGTYPE Operation against a key holding the wrong kind of value"),
            RedisError::OutOfMemory => write!(f, "OOM command not allowed when used memory > 'maxmemory'"),
            RedisError::ParseError(msg) => write!(f, "ERR Protocol error: {}", msg),
            RedisError::InvalidCommand(cmd) => write!(f, "ERR unknown command '{}'", cmd),
            RedisError::InvalidArgumentCount { cmd, expected, got } => 
                write!(f, "ERR wrong number of arguments for '{}' command: expected {}, got {}", cmd, expected, got),
            RedisError::IOError(err) => write!(f, "ERR IO error: {}", err),
            RedisError::Internal(msg) => write!(f, "ERR internal error: {}", msg),
            RedisError::NotInteger => write!(f, "ERR value is not an integer"),
        }
    }
}

impl std::error::Error for RedisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RedisError::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for RedisError {
    fn from(err: io::Error) -> Self {
        RedisError::IOError(err)
    }
}

impl From<std::num::ParseIntError> for RedisError {
    fn from(_: std::num::ParseIntError) -> Self {
        RedisError::NotInteger
    }
}

// Convenience methods
impl RedisError {
    pub fn invalid_command(cmd: impl Into<String>) -> Self {
        RedisError::InvalidCommand(cmd.into())
    }
    
    pub fn wrong_arg_count(cmd: impl Into<String>, expected: usize, got: usize) -> Self {
        RedisError::InvalidArgumentCount { 
            cmd: cmd.into(), 
            expected, 
            got 
        }
    }
}