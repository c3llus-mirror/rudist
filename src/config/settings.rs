use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Settings {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
    pub timeout: Option<Duration>,
    pub max_memory: usize,      // in bytes
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:6379".parse().unwrap(),
            max_connections: 10_000,
            timeout: None,
            max_memory: 0,  // 0 means unlimited
        }
    }
}