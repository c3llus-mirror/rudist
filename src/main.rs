use std::io;
mod network;

use crate::network::server::Server;

fn main() -> io::Result<()> {
    let server = Server::new();
    server.start()
}