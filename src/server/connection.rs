// This file will handle individual client connections. It should define a Connection struct that represents a single client connection and handles reading from and writing to the socket.

// 1) The client sends a 'C0' packet containing the RTMP version it wants to use (usually 3), followed by a 'C1' packet containing a timestamp and some random bytes.
// 2) The server responds with a 'S0' packet also containing the RTMP version, a 'S1' packet containing a timestamp and some random bytes, and a 'S2' packet that mirrors most of the data from the client's 'C1' packet.
// 3) The client completes the handshake by sending a 'C2' packet that mirrors the server's 'S1' packet.

// Path: src/connection.rs

use std::net::TcpStream;
use std::io::{Read, Write};
use rand::Rng;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection { stream }
    }

    pub fn handle(&mut self) -> std::io::Result<()> {
        // Perform the RTMP handshake.
        self.handshake()?;

        // Handle RTMP messages.
        // ...

        Ok(())
    }

    fn handshake(&mut self) -> std::io::Result<()> {
        let mut c0_c1_buffer = [0; 1537]; // Size for C0 + C1
        let mut s0_s1_s2_buffer = [0; 3073]; // Size for S0 + S1 + S2
        let mut c2_buffer = [0; 1536]; // Size for C2
    
        // Read C0 and C1 from the client.
        self.stream.read_exact(&mut c0_c1_buffer)?;

        // Check the RTMP version in C0.
        let version = c0_c1_buffer[0];
        if version != 3 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported RTMP version"));
        }
    
        // Check the timestamp in C1.
        let timestamp = u32::from_be_bytes([c0_c1_buffer[1], c0_c1_buffer[2], c0_c1_buffer[3], c0_c1_buffer[4]]);
        println!("Client timestamp: {}", timestamp);
    
        // Construct S0.
        s0_s1_s2_buffer[0] = 3; // RTMP version
    
        // Construct S1.
        let server_timestamp = 0u32.to_be_bytes(); // Server uptime in milliseconds
        s0_s1_s2_buffer[1..5].copy_from_slice(&server_timestamp);
        
        let mut rng = rand::thread_rng();
        rng.fill(&mut s0_s1_s2_buffer[5..3073]);
        let s1_clone;
        {
            let s1 = &s0_s1_s2_buffer[1..1537];
            println!("Server S1 at 0-10: {:?}", &s1[0..10]);
            s1_clone = s1.to_vec(); // Clone s1 into s1_clone
        }
        // Construct S2 by copying C1.
        let c1 = &c0_c1_buffer[1..1537];
        s0_s1_s2_buffer[1537..3073].copy_from_slice(c1);


    
        // Write S0, S1, and S2 to the client.
        self.stream.write_all(&s0_s1_s2_buffer)?;
        // println!("S1: {:?}", &buffer[1537..3072]);

        // Read C2 from the client.
        self.stream.read_exact(&mut c2_buffer)?;
        println!("Client C2 at 0-10: {:?}", &c2_buffer[0..10]);
        // println!("Client C2: {:?}", &buffer[4611..6147]);

        // Check that C2 matches S1.
        if c2_buffer != s1_clone.as_slice() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "C2 does not match S1"));
        }
    
        Ok(())
    }
    
}