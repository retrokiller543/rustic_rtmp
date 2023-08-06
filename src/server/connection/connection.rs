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
use crate::server::connection::message::message::{
    ConnectMessage,
    CreateStreamMessage,
    PauseMessage,
    PlayMessage,
    RtmpMessage,
};
use crate::server::connection::define::msg_type_id;
use rand::Rng;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::net::TcpStream;


pub struct Connection {
    stream: TcpStream,
}

#[warn(unreachable_code)]
impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
        }
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Perform the RTMP handshake.
        self.handshake().await?;

        // Handle RTMP messages.
        // ...
        loop {
           println!("Listening for msg");
            let message = self.read_message().await?;

            match message {
                RtmpMessage::Connect(connect_message) => {
                    self.handle_connect(connect_message).await?;
                }
                RtmpMessage::_CreateStream(create_stream_message) => {
                    self.handle_create_stream(create_stream_message).await?;
                }
                RtmpMessage::_Play(play_message) => {
                    self.handle_play(play_message).await?;
                }
                RtmpMessage::_Pause(pause_message) => {
                    self.handle_pause(pause_message).await?;
                }
            }
        }
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
        let _timestamp = u32::from_be_bytes([
            c0_c1_buffer[1],
            c0_c1_buffer[2],
            c0_c1_buffer[3],
            c0_c1_buffer[4],
        ]);

        // Construct S0.
        s0_s1_s2_buffer[0] = 3; // RTMP version

        // Construct S1.
        let server_timestamp = (0u32).to_be_bytes(); // Server uptime in milliseconds
        s0_s1_s2_buffer[1..5].copy_from_slice(&server_timestamp);

        let mut rng = rand::thread_rng();
        rng.fill(&mut s0_s1_s2_buffer[5..3073]);
        let s1_clone;
        {
            let s1 = &s0_s1_s2_buffer[1..1537];
            s1_clone = s1.to_vec(); // Clone s1 into s1_clone
        }
        // Construct S2 by copying C1.
        let c1 = &c0_c1_buffer[1..1537];
        s0_s1_s2_buffer[1537..3073].copy_from_slice(c1);

        // Write S0, S1, and S2 to the client.
        self.stream.write_all(&s0_s1_s2_buffer).await?;

        // Read C2 from the client.
        self.stream.read_exact(&mut c2_buffer).await?;

        // Check that C2 matches S1.
        if c2_buffer != s1_clone.as_slice() {
            return Err("C2 does not match S1".into());
        }

        Ok(())
    }

    async fn handle_connect(
        &mut self,
        msg: ConnectMessage
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Connect message.
        // ...
        println!("Connect message: {:?}", msg);

        Ok(())
    }

    async fn handle_create_stream(
        &mut self,
        msg: CreateStreamMessage
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        // println!("Read {} bytes", size);

        // Parse the data into an RtmpMessage.
        let message = Self::parse_message(&buffer[0..size])?;

        Ok(message)
    }

    // This is a placeholder function for parsing a message from the client.
    fn parse_message(data: &[u8]) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        // Parse the data into an RtmpMessage.
        // This will involve examining the data to determine the type of the message,
        // and then parsing the rest of the data based on that type.

        println!("Parsing message: {:?}", data);
        if data.len() < 1 {
            return Err("No message to read".into());
        }

        let header = ChunkBasicHeader::new(&data[0]);
        println!("fmt: {}, cs: {}", header.fmt, header.cs);
        let mut marker = 0;
        match ChunkFmt::from_u8(header.fmt) 
        {
            Some(ChunkFmt::Type0) => 
            {
                let chunk_message_header = ChunkMessageHeader::type0(&data[1..12]);
                match chunk_message_header.message_type_id {
                    Some(msg_type_id::SET_CHUNK_SIZE) => {
                        println!("Message type: Set Chunk Size");
                        let chunk_size = Self::read_set_chunk(&data[12..16])?;
                        println!("chunk_size: {}", chunk_size);
                        marker = 16;
                    }
                    Some(msg_type_id::ABORT) => {
                        println!("Message type: Abort");
                    }
                    Some(msg_type_id::ACKNOWLEDGEMENT) => {
                        println!("Message type: Acknowledgement");
                    }
                    Some(msg_type_id::USER_CONTROL_EVENT) => {
                        println!("Message type: User Control");
                    }
                    Some(msg_type_id::WIN_ACKNOWLEDGEMENT_SIZE) => {
                        println!("Message type: Window Acknowledgement Size");
                    }
                    Some(msg_type_id::SET_PEER_BANDWIDTH) => {
                        println!("Message type: Set Peer Bandwidth");
                    }
                    Some(msg_type_id::AUDIO) => {
                        println!("Message type: Audio");
                    }
                    Some(msg_type_id::VIDEO) => {
                        println!("Message type: Video");
                    }
                    Some(msg_type_id::COMMAND_AMF3) => {
                        println!("Message type: Command AMF3");
                    }
                    Some(msg_type_id::DATA_AMF3) => {
                        println!("Message type: Data AMF3");
                    }
                    Some(msg_type_id::SHARED_OBJ_AMF3) => {
                        println!("Message type: Shared Object AMF3");
                    }
                    Some(msg_type_id::DATA_AMF0) => {
                        println!("Message type: Data AMF0");
                    }
                    Some(msg_type_id::SHARED_OBJ_AMF0) => {
                        println!("Message type: Shared Object AMF0");
                    }
                    Some(msg_type_id::AGGREGATE) => {
                        println!("Message type: Aggregate");
                    }
                    Some(msg_type_id::COMMAND_AMF0) => {
                        println!("Message type: Command AMF0");
                        let message = ConnectMessage::parse(&data[12..])?;
                        return Ok(RtmpMessage::Connect(message));
                    }
                    _ => {
                        println!("Message type: Unknown");
                    }
                }
            }
            Some(ChunkFmt::Type1) => 
            {
                let _chunk_message_header = ChunkMessageHeader::type1(&data[1..12]);
                marker = 12;
            }
            Some(ChunkFmt::Type2) => 
            {
               println!("Message type: Type2");
            }
            Some(ChunkFmt::Type3) => 
            {
                println!("Message type: Type3");
            }
            _ => 
            {
                println!("Unknown chunk format");
            }
        }

        println!("marker: {}", marker);
        // check for more msg types
        if &data[marker] != &0 {
            let message = Self::parse_message(&data[marker..])?;
            return Ok(message);
        }

        //let message_body = &data[28..];

        //if header.cs == MESSAGE_TYPE_CONNECT {
            // Parse the data into a ConnectMessage and return it.
            //let message = ConnectMessage::parse(message_body)?;
            //return Ok(RtmpMessage::Connect(message));
        //}

        // Add similar code for other message types.

        // If the data does not match any known message type, return an error.
        Err("Unknown message type".into())
    }

    fn read_set_chunk(data: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
        let tmp_data = (data[0] << 1) >> 1;
        let chunk_size = u32::from_be_bytes([tmp_data, data[1], data[2], data[3]]);
        Ok(chunk_size)
    }
}

enum ChunkFmt {
    Type0,
    Type1,
    Type2,
    Type3,
}

impl ChunkFmt {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Type0),
            1 => Some(Self::Type1),
            2 => Some(Self::Type2),
            3 => Some(Self::Type3),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct ChunkBasicHeader {
    fmt: u8,
    cs: u8,
}

impl ChunkBasicHeader {
    fn new(byte: &u8) -> ChunkBasicHeader {
        // split into the chunk header and the message body
        let cs = byte & 0b_00111111;
        let fmt = (byte >> 6) & 0b_00000011;

        ChunkBasicHeader { fmt: fmt, cs: cs }
    }
}

struct ChunkMessageHeader {
    timestamp: Option<u32>,
    timestamp_delta: Option<u32>,
    message_length: Option<u32>,
    message_type_id: Option<u8>,
    message_stream_id: Option<u32>,
}

impl ChunkMessageHeader {
    fn default() -> ChunkMessageHeader {
        ChunkMessageHeader {
            timestamp: None,
            timestamp_delta: None,
            message_length: None,
            message_type_id: None,
            message_stream_id: None,
        }
    }

    fn type0(bytes: &[u8]) -> ChunkMessageHeader {
        let mut chunk_message_header = ChunkMessageHeader::default();

        let timestamp = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let message_length = u32::from_be_bytes([0, bytes[3], bytes[4], bytes[5]]); // maybe switch 0 to end of arguments?
        let message_type_id = bytes[6];
        let message_stream_id = u32::from_be_bytes([bytes[10], bytes[9], bytes[8], bytes[7]]);

        chunk_message_header.timestamp = Some(timestamp);
        chunk_message_header.message_length = Some(message_length);
        chunk_message_header.message_type_id = Some(message_type_id);
        chunk_message_header.message_stream_id = Some(message_stream_id);

        println!("timestamp: {}", timestamp);
        println!("message_length: {}", message_length);
        println!("message_type_id: {}", message_type_id);
        println!("message_stream_id: {}", message_stream_id);

        chunk_message_header
    }

    fn type1(bytes: &[u8]) -> ChunkMessageHeader {
        /*
           0                   1                   2                   3
           0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
           +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
           |                timestamp delta                |message length |
           +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
           |     message length (cont)     |message type id|
           +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

                   Chunk Message Header - Type 1
        */

        let mut chunk_message_header = ChunkMessageHeader::default();

        let timestamp_delta = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let message_length = u32::from_be_bytes([0, bytes[3], bytes[4], bytes[5]]);
        let message_type_id = bytes[6];

        chunk_message_header.timestamp_delta = Some(timestamp_delta);
        chunk_message_header.message_length = Some(message_length);
        chunk_message_header.message_type_id = Some(message_type_id);

        println!("timestamp_delta: {}", timestamp_delta);
        println!("message_length: {}", message_length);
        println!("message_type_id: {}", message_type_id);

        chunk_message_header
    }

    fn _type2(bytes: &[u8]) -> ChunkMessageHeader {
        let mut chunk_message_header = ChunkMessageHeader::default();

        chunk_message_header.timestamp_delta = Some(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]));

        chunk_message_header
    }

    
}
