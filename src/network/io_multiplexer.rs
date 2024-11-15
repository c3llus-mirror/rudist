use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::protocol::handler::Handler;
use crate::protocol::resp::types::RESPType;
use crate::protocol::resp::parser::parse_resp;
use crate::storage::memory::ExpireCycleType;

pub struct IOMultiplexer {
    connections: Arc<Mutex<Vec<TcpStream>>>,
    handler: Arc<Mutex<Handler>>,
}

impl IOMultiplexer {
    pub fn new(
        handler: Handler,
    ) -> Self {
        IOMultiplexer {
            connections: Arc::new(Mutex::new(Vec::new())),
            handler: Arc::new(Mutex::new(handler)),
        }
    }

    pub fn add_connection(&self, stream: TcpStream) {
        stream.set_nonblocking(true).unwrap();
        let mut connections = self.connections.lock().unwrap();
        connections.push(stream);
    }

    pub fn process_next_request(&self) -> io::Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(mut stream) = connections.pop() {
            let mut buffer = Vec::new();
            let mut temp_buffer = [0; 1024];
            
            loop {
                match stream.read(&mut temp_buffer) {
                    Ok(0) => break, // conn clozed
                    Ok(n) => {
                        buffer.extend_from_slice(&temp_buffer[..n]);
                        if buffer.contains(&b'\n') { break; }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e),
                }
            }
    
            if !buffer.is_empty() {
                let response = {
                    let mut handler = self.handler.lock().unwrap();
                    handler.handle(&buffer)
                };
    
                stream.write_all(response.encode().as_bytes())?;
            }
            
            connections.push(stream);
        }
        Ok(())
    }

    pub fn active_expire_cycle_fast(&self){
        let mut handler = self.handler.lock().unwrap();
        handler.active_expire_cycle_fast();
    }

    pub fn active_expire_cycle_slow(&self){
        let mut handler = self.handler.lock().unwrap();
        handler.active_expire_cycle_slow();
    }
}
