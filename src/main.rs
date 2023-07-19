// This file will contain the main function that starts the server. It should be responsible for setting up the server and starting the main event loop.

mod protocol;
mod stream;
mod server;
mod utils;
mod error;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new("127.0.0.1:1935".to_owned());
    server.run().await?;
    Ok(())
}
