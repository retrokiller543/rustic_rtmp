// This file will handle individual client connections. It should define a Connection struct that represents a single client connection and handles reading from and writing to the socket.

// 1) The client sends a 'C0' packet containing the RTMP version it wants to use (usually 3), followed by a 'C1' packet containing a timestamp and some random bytes.
// 2) The server responds with a 'S0' packet also containing the RTMP version, a 'S1' packet containing a timestamp and some random bytes, and a 'S2' packet that mirrors most of the data from the client's 'C1' packet.
// 3) The client completes the handshake by sending a 'C2' packet that mirrors the server's 'S1' packet.
    /* 
    0 1 2 3 4 5 6 7
    +-+-+-+-+-+-+-+-+
    |    version    |
    +-+-+-+-+-+-+-+-+

    C0 and S0 bits
    */

    /*
    
    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        time (4 bytes)                         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        zero (4 bytes)                         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        random bytes                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        random bytes                           |
    |                           (cont)                              |
    |                            ....                               |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

                            C1 and S1 bits
    
    */

    /*
    
    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        time (4 bytes)                         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                       time2 (4 bytes)                         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                         random echo                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                         random echo                           |
    |                            (cont)                             |
    |                             ....                              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

                            C2 and S2 bits
    
     */
// Path: src/connection.rs
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection { stream }
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Perform the RTMP handshake.
        self.handshake().await?;

        // Handle RTMP messages.
        // ...

        Ok(())
    }

    async fn handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut c0_c1_buffer = [0; 1537]; // Size for C0 + C1
        let mut s0_s1_s2_buffer = [0; 3073]; // Size for S0 + S1 + S2
        let mut c2_buffer = [0; 1536]; // Size for C2
    
        // Read C0 and C1 from the client.
        self.stream.read_exact(&mut c0_c1_buffer).await?;

        // Check the RTMP version in C0.
        let version = c0_c1_buffer[0];
        if version != 3 {
            return Err("Unsupported RTMP version".into());
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
        self.stream.write_all(&s0_s1_s2_buffer).await?;
        
        // Read C2 from the client.
        self.stream.read_exact(&mut c2_buffer).await?;
        println!("Client C2 at 0-10: {:?}", &c2_buffer[0..10]);

        // Check that C2 matches S1.
        if c2_buffer != s1_clone.as_slice() {
            return Err("C2 does not match S1".into());
        }
    
        Ok(())
    }
}
