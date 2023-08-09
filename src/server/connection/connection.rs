// Path: src/server/connection.rs
use crate::server::connection::message::message::{
    ConnectMessage,
    CreateStream,
    PauseMessage,
    PlayMessage,
    RtmpMessage,
    CommandObject,
    ResultObject,
    SetChunkSizeMessage,
    AcknowledgementMessage,
    BasicCommand,
    ReleaseStream,
    FCPublish,
    Publish,
    Event,
    OnStatus,
    SetDataFrame,
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
    marker: usize,
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
                RtmpMessage::CreateStream(create_stream_message) => {
                    self.handle_create_stream(create_stream_message).await?;
                }
                RtmpMessage::_Play(play_message) => {
                    self.handle_play(play_message).await?;
                }
                RtmpMessage::_Pause(pause_message) => {
                    self.handle_pause(pause_message).await?;
                }
                RtmpMessage::Publish(publish_message) => {
                    self.handle_publish(publish_message).await?;
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
        let mut result_obj = ResultObject::new("_result".to_string(), 1, 0);
        result_obj.set_command_object(e);
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
        msg: CreateStream
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a CreateStream message.
        // ...
        let result_obj = ResultObject::new("_result".to_string(), msg.transaction_id, 1);
        let command = result_obj.parse()?;
        let command_vec: Vec<u8> = command.freeze().to_vec();
        let result_header = self.write_header(20, command_vec.len() as u32, 0, 0, 3);
        let mut result_msg = Vec::new();
        result_msg.extend_from_slice(&result_header);
        result_msg.extend_from_slice(&command_vec);
        info!("result msg: {:?}", result_msg);

        self.stream.write_all(&result_msg).await?;
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

    async fn handle_publish(&mut self, msg: Publish) -> Result<(), Box<dyn std::error::Error>> {
        // Handle a Publish message.
        // ...
        let stream_begin_header = self.write_header(4, 6, 0, 0, 2);
        let stream_begin = Event::new(0, 1).parse();
        let mut stream_begin_msg = Vec::new();
        stream_begin_msg.extend_from_slice(&stream_begin_header);
        stream_begin_msg.extend_from_slice(&stream_begin);
        info!("stream begin msg: {:?}", stream_begin_msg);
        self.stream.write_all(&stream_begin_msg).await?;

        let on_status = OnStatus::new(msg.transaction_id).parse();
        let mut on_status_vec: Vec<u8> = Vec::new();
        on_status_vec.extend_from_slice(&on_status.unwrap());
        let on_status_header = self.write_header(20, on_status_vec.len() as u32, 0, 1, 3);
        let mut on_status_msg = Vec::new();
        on_status_msg.extend_from_slice(&on_status_header);
        on_status_msg.extend_from_slice(&on_status_vec);
        info!("on status msg: {:?}", on_status_msg);
        self.stream.write_all(&on_status_msg).await?;
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

        error!("Marker Before CBH: {}", self.marker);
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
                let read_to = self.marker + 3;
                let chunk_message_header = ChunkMessageHeader::type2(&data[self.marker..read_to]);
                self.marker = read_to;
                let msg = Connection::read_msg_type(self, &data, chunk_message_header);
                return msg;
            }
            Some(ChunkFmt::Type3) => 
            {
                let chunk_message_header = ChunkMessageHeader::type3();
                let msg = Connection::read_msg_type(self, &data, chunk_message_header);
                return msg;
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
                if let Some(msg_len) = msg_header.message_length {
                    let read_to = self.marker + msg_len as usize;
                    let msg_name = BasicCommand::parse(&data[self.marker..read_to])?.command_name;
                    info!("msg_name: {:?}", msg_name);
                    match msg_name.as_str() {
                        "@setDataFrame" => {
                            let message = SetDataFrame::parse(&data[self.marker..read_to])?;
                            self.marker = read_to;
                            info!("message: {:?}", message);
                            return Ok(RtmpMessage::SetDataFrame(message));
                        }
                        _ => {
                            error!("Unknown Data: {:?}", msg_name);
                            return Err("Unknown Data".into())
                        }
                    }
                }
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
                            info!("releaseStream: {:?}", message);
                            self.marker = read_to;
                            return Ok(RtmpMessage::ReleaseStream(message));
                        }
                        "FCPublish" => 
                        {
                            let message = FCPublish::parse(&data[self.marker..read_to])?;
                            info!("FCPublish: {:?}", message);
                            self.marker = read_to;
                            return Ok(RtmpMessage::FCPublish(message));
                        }
                        "createStream" => 
                        {
                            let message = CreateStream::parse(&data[self.marker..read_to])?;
                            info!("createStream: {:?}", message);
                            self.marker = read_to;
                            return Ok(RtmpMessage::CreateStream(message));
                        }
                        "publish" => 
                        {
                            let message = Publish::parse(&data[self.marker..read_to])?;
                            info!("publish: {:?}", message);
                            self.marker = read_to;
                            return Ok(RtmpMessage::Publish(message));
                        }
                        _ => {
                            error!("Unknown command: {:?}", command_name);
                            return Err("Unknown command".into())
                        }
                    };
                } else {
                    let command_name = BasicCommand::parse(&data[self.marker..])?.command_name;
                    info!("command_name: {:?}", command_name);
                    match command_name.as_str() {
                        "connect" => {
                            let message = ConnectMessage::parse(&data[self.marker..])?;
                            return Ok(RtmpMessage::Connect(message));
                        }
                        "releaseStream" => {
                            let message = ReleaseStream::parse(&data[self.marker..])?;
                            info!("releaseStream: {:?}", message);
                            return Ok(RtmpMessage::ReleaseStream(message));
                        }
                        "FCPublish" => 
                        {
                            let message = FCPublish::parse(&data[self.marker..])?;
                            info!("FCPublish: {:?}", message);
                            return Ok(RtmpMessage::FCPublish(message));
                        }
                        "createStream" => 
                        {
                            let message = CreateStream::parse(&data[self.marker..])?;
                            info!("createStream: {:?}", message);
                            return Ok(RtmpMessage::CreateStream(message));
                        }
                        "publish" => 
                        {
                            let message = Publish::parse(&data[self.marker..])?;
                            info!("publish: {:?}", message);
                            return Ok(RtmpMessage::Publish(message));
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

    fn type2(bytes: &[u8]) -> ChunkMessageHeader {
        let mut chunk_message_header = ChunkMessageHeader::default();

        chunk_message_header.timestamp_delta = Some(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]));

        chunk_message_header
    }

    fn type3() -> ChunkMessageHeader {
        ChunkMessageHeader::default()
    }
}


#[allow(unused_mut)]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    async fn setup() -> (Connection, TcpStream) {
        // Start a TcpListener to accept connections (server-side)
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Connect a TcpStream to the TcpListener (client-side)
        let mut client = TcpStream::connect(addr).await.unwrap();

        // Accept the connection on the server-side
        let (server, _) = listener.accept().await.unwrap();
        
        // Create the Connection instance
        let mut conn = Connection::new(server);

        (conn, client)
    }

    #[tokio::test]
    async fn test_read_connect() {
        let (mut conn, mut client) = setup().await;

        // Emulate the client sending data
        let mock_data: &[u8] = &[2, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 16, 0, 3, 0, 0, 0, 0, 0, 179, 20, 0, 0, 0, 0, 2, 0, 7, 99, 111, 110, 110, 101, 99, 116, 0, 63, 240, 0, 0, 0, 0, 0, 0, 3, 0, 3, 97, 112, 112, 2, 0, 4, 108, 105, 118, 101, 0, 4, 116, 121, 112, 101, 2, 0, 10, 110, 111, 110, 112, 114, 105, 118, 97, 116, 101, 0, 8, 102, 108, 97, 115, 104, 86, 101, 114, 2, 0, 31, 70, 77, 76, 69, 47, 51, 46, 48, 32, 40, 99, 111, 109, 112, 97, 116, 105, 98, 108, 101, 59, 32, 70, 77, 83, 99, 47, 49, 46, 48, 41, 0, 6, 115, 119, 102, 85, 114, 108, 2, 0, 30, 114, 116, 109, 112, 58, 47, 47, 49, 57, 50, 46, 49, 54, 56, 46, 49, 46, 49, 49, 50, 58, 49, 57, 51, 53, 47, 108, 105, 118, 101, 0, 5, 116, 99, 85, 114, 108, 2, 0, 30, 114, 116, 109, 112, 58, 47, 47, 49, 57, 50, 46, 49, 54, 56, 46, 49, 46, 49, 49, 50, 58, 49, 57, 51, 53, 47, 108, 105, 118, 101, 0, 0, 9];
        client.write_all(mock_data).await.expect("Failed to write mock data");

        // Read & handle the message in the Connection instance
        let message = conn.read_message().await.expect("Failed to read message");
        let result = match message {
            RtmpMessage::Connect(connect_message) => {
                assert_eq!(connect_message.connect_object.app, "live", "App should be 'live'");
                assert_eq!(connect_message.connect_object.flash_ver, "FMLE/3.0 (compatible; FMSc/1.0)", "Flash version should be 'FMLE/3.0 (compatible; FMSc/1.0)'");
                assert_eq!(connect_message.connect_object.swf_url, "rtmp://192.168.1.112:1935/live", "SWF URL should be 'rtmp://192.168.1.112:1935/live'");
                assert_eq!(connect_message.connect_object.tc_url, "rtmp://192.168.1.112:1935/live", "TC URL should be 'rtmp://192.168.1.112:1935/live'");
                assert_eq!(connect_message.connect_object.stream_type, "nonprivate", "Stream type should be 'nonprivate'");
                assert_eq!(connect_message.id, 1, "ID should be 1");
                Ok(())
            }
            // ... handle other cases or use a default case.
            _ => Err(Box::<dyn std::error::Error>::from("Unknown message type"))
        };

        // Check the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_ack() {
        let (mut conn, mut client) = setup().await;

        // Emulate the client sending data
        let mock_data: &[u8] = &[66, 0, 0, 0, 0, 0, 4, 3, 0, 0, 12, 35];
        client.write_all(mock_data).await.expect("Failed to write mock data");

        // Read & handle the message in the Connection instance
        let message = conn.read_message().await.expect("Failed to read message");
        let result = match message {
            RtmpMessage::Acknowledgement(ack) => {
                assert_eq!(ack.sequence_number, 3107, "Sequence number should be 3107");
                Ok(())
            }
            // ... handle other cases or use a default case.
            _ => Err(Box::<dyn std::error::Error>::from("Unknown message type"))
        };

        // Check the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_create() {
        let (mut conn, mut client) = setup().await;

        // Emulate the client sending data
        let mock_data: &[u8] = &[67, 0, 0, 0, 0, 0, 38, 20, 2, 0, 13, 114, 101, 108, 101, 97, 115, 101, 83, 116, 114, 101, 97, 109, 0, 64, 0, 0, 0, 0, 0, 0, 0, 5, 2, 0, 9, 115, 116, 114, 101, 97, 109, 107, 101, 121, 67, 0, 0, 0, 0, 0, 34, 20, 2, 0, 9, 70, 67, 80, 117, 98, 108, 105, 115, 104, 0, 64, 8, 0, 0, 0, 0, 0, 0, 5, 2, 0, 9, 115, 116, 114, 101, 97, 109, 107, 101, 121, 67, 0, 0, 0, 0, 0, 25, 20, 2, 0, 12, 99, 114, 101, 97, 116, 101, 83, 116, 114, 101, 97, 109, 0, 64, 16, 0, 0, 0, 0, 0, 0, 5];
        client.write_all(mock_data).await.expect("Failed to write mock data");

        // Read & handle the message in the Connection instance
        let message = conn.read_message().await.expect("Failed to read message");
        let result = match message {
            RtmpMessage::CreateStream(create_stream) => {
                assert_eq!(create_stream.command_name, "createStream", "Command name should be 'createStream'");
                assert_eq!(create_stream.transaction_id, 4, "Transaction ID should be 4");
                Ok(())
            }
            // ... handle other cases or use a default case.
            _ => Err(Box::<dyn std::error::Error>::from("Unknown message type"))
        };

        // Check the result
        assert!(result.is_ok());
    }
}