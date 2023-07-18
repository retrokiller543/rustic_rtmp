// This file will contain the main library code for your server. It should define the main Server struct and its associated methods.

// Path: src/server.rs

mod connection;
use connection::Connection;
use std::net::TcpListener;

pub struct Server {
    address: String,
}

impl Server {
    pub fn new(address: String) -> Server {
        Server { address }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).expect("Could not bind");
        println!("Listening on {}", self.address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Create a new Connection and handle it.
                    let mut connection = Connection::new(stream);
                    match connection.handle() {
                        Ok(_) => {
                            println!("Handled connection");
                        }
                        Err(e) => {
                            println!("Failed to handle connection: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
