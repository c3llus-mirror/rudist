use std::io::{self};
use std::net::TcpListener;
use crate::network::io_multiplexer::IOMultiplexer;

pub struct EventLoop {
    listener: TcpListener,
}

impl EventLoop {
    pub fn new(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener })
    }

    pub fn run(&self) -> io::Result<()> {
        let io_multiplexer = IOMultiplexer::new();

        // Accepting connections and processing requests one by one.
        for stream in self.listener.incoming() {
            let stream = stream?;
            println!("New connection from: {}", stream.peer_addr()?);
            io_multiplexer.add_connection(stream);
            io_multiplexer.process_next_request()?;
        }

        Ok(())
    }
}
