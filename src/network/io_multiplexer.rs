use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct IOMultiplexer {
    connections: Arc<Mutex<Vec<TcpStream>>>,
}

impl IOMultiplexer {
    pub fn new() -> Self {
        IOMultiplexer {
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_connection(&self, stream: TcpStream) {
        let mut connections = self.connections.lock().unwrap();
        connections.push(stream);
    }

    pub fn process_next_request(&self) -> io::Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(mut stream) = connections.pop() {
            let mut buffer = [0; 1024];
            // Read the incoming request
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read > 0 {
                // Process request (In this case, just echo the data back)
                stream.write_all(&buffer[..bytes_read])?;
            }
        }
        Ok(())
    }
}
