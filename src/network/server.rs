use std::io;
use crate::network::event_loop::EventLoop;

pub struct Server {
    address: String,
}

impl Server {
    pub fn new() -> Self {
        let address = "127.0.0.1:6379".to_string();
        Server { address }
    }

    pub fn start(&self) -> io::Result<()> {
        println!("Starting server on {}", self.address);
        let event_loop = EventLoop::new(&self.address)?;
        event_loop.run()
    }
}