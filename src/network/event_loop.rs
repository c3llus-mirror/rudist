use std::io::{self};
use std::net::TcpListener;
use crate::network::io_multiplexer::IOMultiplexer;

pub struct EventLoop {
    listener: TcpListener,
    io_multiplexer: IOMultiplexer,
}

impl EventLoop {
    pub fn new(address: &str, io_multiplexer: IOMultiplexer) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener, io_multiplexer })
    }

    pub fn run(&self) -> io::Result<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    // println!("New connection from: {}", addr);
                    self.io_multiplexer.add_connection(stream);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
    
            self.io_multiplexer.process_next_request()?;
            
            // TODO: better "small sleep" implementation
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
