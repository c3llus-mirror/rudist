use std::io;
use crate::network::event_loop::EventLoop;
use crate::network::io_multiplexer::IOMultiplexer;
use crate::protocol::handler::Handler;
use crate::engine::Engine;


pub struct Server {
    address: String,
    event_loop: EventLoop,
}

impl Server {
    pub fn new() -> io::Result<Self> {
        let address = "127.0.0.1:6379".to_string();
        let engine = Engine::new();
        let handler = Handler::new(engine);
        let io_multiplexer = IOMultiplexer::new(handler);
        let event_loop = EventLoop::new(&address, io_multiplexer)?;
        Ok(Server { address, event_loop })
    }

    pub fn start(&self) -> io::Result<()> {
        println!("Starting server on {}", self.address);
        self.event_loop.run()
    }
}