use std::io;
mod network;
mod protocol;
mod storage;
mod utils;
mod engine;

use crate::network::server::Server;

fn main() -> io::Result<()> {
    let server = Server::new()?;
    server.start()
}