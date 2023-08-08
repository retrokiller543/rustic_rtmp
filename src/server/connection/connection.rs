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
    CommandObject,
    ResultObject,
    SetChunkSizeMessage,
    AcknowledgementMessage,
    BasicCommand,
    ReleaseStream
};
use crate::server::connection::define::msg_type_id;

use rand::Rng;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::net::TcpStream;
use log::{info, error, warn};

pub const WINDOW_ACKNOWLEDGEMENT_SIZE: u32 = 4096;
pub const SET_BANDWIDTH_SIZE: u32 = 4096;

pub struct Connection {
    stream: TcpStream,
    marker: usize
}

#[warn(unreachable_code)]
impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            marker: 0
        }
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Perform the RTMP handshake.
        self.handshake().await?;

        // Handle RTMP messages.
        // ...
        loop {
           warn!("Listening for msg");
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
                _ => {
                    error!("Unhandled message: {:?}", message);
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
            error!("Unsupported RTMP version: {}", version);
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
            error!("C2 does not match S1");
            return Err("C2 does not match S1".into());
        }

        Ok(())
    }

    pub fn write_header(&mut self, msg_type_id: u8, msg_len: u32, timestamp: u32, stream_id: u32, chunk_basic_header: u8) -> [u8; 12] {
        pub fn insert_bytes(dst: &mut[u8; 12], data: u32, start_idx: usize, end_idx: usize)
        {
            let data_as_bytes = data.to_be_bytes();

            let mut i: usize = 0;
            if end_idx - start_idx == 3
            {
                i = 1;
            }

            for idx in start_idx..end_idx
            {
                dst[idx] = data_as_bytes[i];
                i += 1;
            } 
        }
        
        let mut header = [0; 12];
        header[0] = chunk_basic_header;
        insert_bytes(&mut header, timestamp, 1, 4);
        insert_bytes(&mut header, msg_len, 4, 7);
        header[7] = msg_type_id;
        insert_bytes(&mut header, stream_id, 8, 12);
        
        info!("header: {:?}", header);
        header
    }

    async fn handle_connect(
        &mut self,
        msg: ConnectMessage
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Connect message.
        // ...
        // TODO: Handle connect message
        // send win ack size
        // connect to app
        // send set peer bandwidth
        // read ack
        // make and send StreamBegin
        // make and send _result
        info!("==========Start Connect msg Handle==========");
        info!("Connect message: {:?}", msg);

        let ack_header = self.write_header(5, 4, 0, 0, 2);       
        let mut ack_msg: [u8; 16] = [0; 16];

        ack_msg[0..12].copy_from_slice(&ack_header);
        ack_msg[12..16].copy_from_slice(&WINDOW_ACKNOWLEDGEMENT_SIZE.to_be_bytes());
        self.stream.write_all(&ack_msg).await?;

        let bandwidth_header = self.write_header(6, 5, 0, 0, 2);
        let mut bandwidth_msg: [u8; 17] = [0; 17];

        bandwidth_msg[0..12].copy_from_slice(&bandwidth_header);

        let bandwidth_as_bytes = SET_BANDWIDTH_SIZE.to_be_bytes();

        bandwidth_msg[12..16].copy_from_slice(&bandwidth_as_bytes);
        bandwidth_msg[16] = 2;
        let e = CommandObject::new("FMS/3,0,1,123".to_string(), 31);
        let result_obj = ResultObject::new("_result".to_string(), 1, e, 0);
        let command = result_obj.parse()?;
        let command_vec: Vec<u8> = command.freeze().to_vec();
        
        let command_header = self.write_header(20, command_vec.len() as u32, 0, 0, 2);
        let mut command_msg = Vec::new();
        command_msg.extend_from_slice(&command_header);
        command_msg.extend_from_slice(&command_vec);
        
        let mut set_peer_bandwidth  = Vec::new();
        set_peer_bandwidth.extend_from_slice(&bandwidth_msg);
        set_peer_bandwidth.extend_from_slice(&command_msg);
        info!("set peer bandwidth: {:?}", set_peer_bandwidth);
        self.stream.write_all(&set_peer_bandwidth).await?;

        self.read_message().await?;
        

        info!("==========End Connect msg Handle==========");
        Ok(())
    }

    async fn handle_create_stream(
        &mut self,
        msg: CreateStreamMessage
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a CreateStream message.
        // ...
        info!("CreateStream message: {:?}", msg);
        Ok(())
    }

    async fn handle_play(&mut self, msg: PlayMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Play message.
        // ...
        info!("Play message: {:?}", msg);
        Ok(())
    }

    async fn handle_pause(&mut self, msg: PauseMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Pause message.
        // ...
        info!("Pause message: {:?}", msg);
        Ok(())
    }

    async fn read_message(&mut self) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        // Create a buffer to hold the message data.
        let mut buffer = [0; 4096]; // Adjust the size as needed.

        // Read data from the client into the buffer.
        let size = self.stream.read(&mut buffer).await?;
        // info!("Read {} bytes", size);
        self.marker = 0;

        // Parse the data into an RtmpMessage.
        let message = Self::parse_message(self, &buffer[0..size])?;

        Ok(message)
    }

    pub fn parse_msg_header(&mut self, data: &[u8]) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        if data.len() < 1 {
            error!("No message to read");
            return Err("No message to read".into());
        }

        let basic_header = ChunkBasicHeader::new(&data[self.marker]);
        self.marker += 1;
        info!("fmt: {}, cs: {}", basic_header.fmt, basic_header.cs);
        let msg = Connection::read_header_types(self, &data, basic_header)?;
        Ok(msg)
    }

    pub fn read_header_types(&mut self, data: &[u8], header: ChunkBasicHeader) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        match ChunkFmt::from_u8(header.fmt) 
        {
            Some(ChunkFmt::Type0) => 
            {
                let read_to = self.marker + 11;
                let chunk_message_header = ChunkMessageHeader::type0(&data[self.marker..read_to]);
                self.marker = read_to;
                let msg = Connection::read_msg_type(self, &data, chunk_message_header);
                return msg;
            }
            Some(ChunkFmt::Type1) => 
            {
                let read_to = self.marker + 7;
                let chunk_message_header = ChunkMessageHeader::type1(&data[self.marker..read_to]);
                self.marker = read_to;
                let msg = Connection::read_msg_type(self, &data, chunk_message_header);
                return msg;
            }
            Some(ChunkFmt::Type2) => 
            {
               info!("Message type: Type2");
            }
            Some(ChunkFmt::Type3) => 
            {
                info!("Message type: Type3");
            }
            _ => 
            {
                info!("Unknown chunk format");
            }
        }
        error!("Unknown chunk format");
        Err("Unknown chunk format".into())
    }

    pub fn read_msg_type(&mut self, data: &[u8], msg_header: ChunkMessageHeader) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        match msg_header.message_type_id {
            Some(msg_type_id::SET_CHUNK_SIZE) => {
                info!("Message type: Set Chunk Size");
                let read_to = self.marker + 4;
                let chunk_size = Self::read_set_chunk(&data[self.marker..read_to])?;
                info!("chunk_size: {}", chunk_size);
                self.marker = read_to;
                let set_chunk_size = SetChunkSizeMessage::new(chunk_size);
                return Ok(RtmpMessage::SetChunkSize(set_chunk_size));
            }
            Some(msg_type_id::ABORT) => {
                info!("Message type: Abort");
            }
            Some(msg_type_id::ACKNOWLEDGEMENT) => {
                info!("Message type: Acknowledgement");
                let read_to = self.marker + 4;
                let ack_sequence_number = Self::read_ack(&data[self.marker..read_to])?;
                self.marker = read_to;
                let ack = AcknowledgementMessage::new(ack_sequence_number);
                info!("ack: {:?}", ack);
                return Ok(RtmpMessage::Acknowledgement(ack));
            }
            Some(msg_type_id::USER_CONTROL_EVENT) => {
                info!("Message type: User Control");
            }
            Some(msg_type_id::WIN_ACKNOWLEDGEMENT_SIZE) => {
                info!("Message type: Window Acknowledgement Size");
            }
            Some(msg_type_id::SET_PEER_BANDWIDTH) => {
                info!("Message type: Set Peer Bandwidth");
            }
            Some(msg_type_id::AUDIO) => {
                info!("Message type: Audio");
            }
            Some(msg_type_id::VIDEO) => {
                info!("Message type: Video");
            }
            Some(msg_type_id::COMMAND_AMF3) => {
                info!("Message type: Command AMF3");
            }
            Some(msg_type_id::DATA_AMF3) => {
                info!("Message type: Data AMF3");
            }
            Some(msg_type_id::SHARED_OBJ_AMF3) => {
                info!("Message type: Shared Object AMF3");
            }
            Some(msg_type_id::DATA_AMF0) => {
                info!("Message type: Data AMF0");
            }
            Some(msg_type_id::SHARED_OBJ_AMF0) => {
                info!("Message type: Shared Object AMF0");
            }
            Some(msg_type_id::AGGREGATE) => {
                info!("Message type: Aggregate");
            }
            Some(msg_type_id::COMMAND_AMF0) => {
                info!("Message type: Command AMF0");
                if let Some(msg_len) = msg_header.message_length {
                    let read_to = self.marker + msg_len as usize;
                    let command_name = BasicCommand::parse(&data[self.marker..read_to])?.command_name;
                    info!("command_name: {:?}", command_name);
                    match command_name.as_str() {
                        "connect" => {
                            let message = ConnectMessage::parse(&data[self.marker..read_to])?;
                            self.marker = read_to;
                            return Ok(RtmpMessage::Connect(message));
                        }
                        "releaseStream" => {
                            let message = ReleaseStream::parse(&data[self.marker..read_to])?;
                            self.marker = read_to;
                            return Ok(RtmpMessage::ReleaseStream(message));
                        }
                        _ => {
                            error!("Unknown command: {:?}", command_name);
                            return Err("Unknown command".into())
                        }
                    };
                }
            }
            _ => {
                error!("Message type: Unknown");
                return Err("Unknown message type".into())
            }
        }
        error!("Unknown message type");
        Err("Unknown message type".into())
    }

    fn parse_message(&mut self, data: &[u8]) -> Result<RtmpMessage, Box<dyn std::error::Error>> {
        info!("Parse Message data: {:?}", data);
        
        let msg = Connection::parse_msg_header(self, &data)?;

        // check for more msg types
        if self.marker < data.len() &&  &data[self.marker] != &0 {
            warn!("more msg types");
            let message = Self::parse_message(self, &data)?;
            return Ok(message);
        }
    
        Ok(msg)
    }

    fn read_set_chunk(data: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
        let tmp_data = (data[0] << 1) >> 1;
        let chunk_size = u32::from_be_bytes([tmp_data, data[1], data[2], data[3]]);
        Ok(chunk_size)
    }

    fn read_ack(data: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
        let tmp_data = (data[0] << 1) >> 1;
        let ack = u32::from_be_bytes([tmp_data, data[1], data[2], data[3]]);
        Ok(ack)
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
pub struct ChunkBasicHeader {
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

pub struct ChunkMessageHeader {
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

        info!("timestamp: {}", timestamp);
        info!("message_length: {}", message_length);
        info!("message_type_id: {}", message_type_id);
        info!("message_stream_id: {}", message_stream_id);

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

        info!("timestamp_delta: {}", timestamp_delta);
        info!("message_length: {}", message_length);
        info!("message_type_id: {}", message_type_id);

        chunk_message_header
    }

    fn _type2(bytes: &[u8]) -> ChunkMessageHeader {
        let mut chunk_message_header = ChunkMessageHeader::default();

        chunk_message_header.timestamp_delta = Some(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]));

        chunk_message_header
    }

    
}
