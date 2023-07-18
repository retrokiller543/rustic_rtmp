// This file will contain the main function that starts the server. It should be responsible for setting up the server and starting the main event loop.

mod protocol;
mod stream;
mod message;
mod server;
mod utils;
mod amf;
mod error;

use server::Server;

fn main() {
    let server = Server::new("127.0.0.1:1935".to_string());
    server.run();
}
