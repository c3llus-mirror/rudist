use std::io::{self};
use std::net::TcpListener;
use crate::network::io_multiplexer::IOMultiplexer;
use std::time::{Instant,Duration};

pub struct EventLoop {
    listener: TcpListener,
    io_multiplexer: IOMultiplexer,
    small_sleep_duration: Duration,
    active_expiry_fast_duration: Duration,
    active_expiry_slow_duration: Duration,
}

impl EventLoop {
    pub fn new(
        address: &str, 
        io_multiplexer: IOMultiplexer,
        small_sleep_duration: Duration,
        active_expiry_fast_duration: Duration,
        active_expiry_slow_duration: Duration,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener, io_multiplexer, small_sleep_duration, active_expiry_fast_duration, active_expiry_slow_duration })
    }

    pub fn run(&self) -> io::Result<()> {
        let mut last_fast_cycle = Instant::now();
        let mut last_slow_cycle = Instant::now();

        loop {
            // TODO: better "small sleep" implementation
            std::thread::sleep(self.small_sleep_duration);

            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    // println!("New connection from: {}", addr);
                    self.io_multiplexer.add_connection(stream);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // run active expiry only when there are no new connections
                    if last_fast_cycle.elapsed() > self.active_expiry_fast_duration {
                        self.io_multiplexer.active_expire_cycle_fast();
                        last_fast_cycle = Instant::now();
                    }
                    
                    if last_slow_cycle.elapsed() > self.active_expiry_slow_duration {
                        self.io_multiplexer.active_expire_cycle_slow();
                        last_slow_cycle = Instant::now();
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}
