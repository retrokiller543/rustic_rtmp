// This file will contain the main library code for your server. It should define the main Server struct and its associated methods.

// Path: src/server.rs

mod connection;
use connection::Connection;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;

pub struct Server {
    address: String,
}

impl Server {
    pub fn new(address: String) -> Server {
        Server { address }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("Listening on {}", self.address);

        loop {
            let (stream, _) = listener.accept().await?;

            task::spawn_blocking(move || {
                if let Err(err) = tokio::runtime::Runtime::new().unwrap().block_on(Self::handle_connection(stream)) {
                    println!("Failed to handle connection: {}", err);
                } else {
                    println!("Connection handled successfully");
                }
            });
        }
    }

    async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection = Connection::new(stream);
        connection.handle().await?;
        Ok(())
    }
}
