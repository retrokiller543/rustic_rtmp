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
// Path: src/server/connection.rs
use crate::server::connection::message::message::{RtmpMessage, ConnectMessage, CreateStreamMessage, PlayMessage, PauseMessage};
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MESSAGE_TYPE_CONNECT: u8 = 2; // This is just an example. Use the correct value from the RTMP specification.


pub struct Connection {
    stream: TcpStream,
}

#[warn(unreachable_code)]
impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection { stream }
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Perform the RTMP handshake.
        self.handshake().await?;

        // Handle RTMP messages.
        // ...
        loop {
            let message = self.read_message().await?;

            match message {
                RtmpMessage::Connect(connect_message) => {
                    self.handle_connect(connect_message).await?;
                },
                RtmpMessage::CreateStream(create_stream_message) => {
                    self.handle_create_stream(create_stream_message).await?;
                },
                RtmpMessage::Play(play_message) => {
                    self.handle_play(play_message).await?;
                },
                RtmpMessage::Pause(pause_message) => {
                    self.handle_pause(pause_message).await?;
                },
            }
        }
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

    async fn handle_connect(&mut self, msg: ConnectMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Connect message.
        // ...
        println!("Connect message: {:?}", msg);
        Ok(())
    }
    
    async fn handle_create_stream(&mut self, msg: CreateStreamMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a CreateStream message.
        // ...
        println!("CreateStream message: {:?}", msg);
        Ok(())
    }
    
    async fn handle_play(&mut self, msg: PlayMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Play message.
        // ...
        println!("Play message: {:?}", msg);
        Ok(())
    }
    
    async fn handle_pause(&mut self, msg: PauseMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Pause message.
        // ...
        println!("Pause message: {:?}", msg);
        Ok(())
    }

    async fn read_message(&mut self) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        // Create a buffer to hold the message data.
        let mut buffer = [0; 4096]; // Adjust the size as needed.
    
        // Read data from the client into the buffer.
        let size = self.stream.read(&mut buffer).await?;
        println!("Read {} bytes", size);
    
        let _chunk_header = &buffer[0..28];
        println!("Chunk header: {:?}", _chunk_header);
        // Parse the data into an RtmpMessage.
        let message = Self::parse_message(&buffer[28..size])?;
    
        Ok(message)
    }
    
    // This is a placeholder function for parsing a message from the client.
    // You will need to implement this according to the requirements of your application and the RTMP protocol.
    fn parse_message(data: &[u8]) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        // Parse the data into an RtmpMessage.
        // This will involve examining the data to determine the type of the message,
        // and then parsing the rest of the data based on that type.
    
        println!("Message data: {:?}", data);
        
        if data[0] == MESSAGE_TYPE_CONNECT {
            // Parse the data into a ConnectMessage and return it.
            let message = ConnectMessage::parse(data)?;
            return Ok(RtmpMessage::Connect(message));
        }
    
        // Add similar code for other message types.
    
        // If the data does not match any known message type, return an error.
        Err("Unknown message type".into())
    }
}
