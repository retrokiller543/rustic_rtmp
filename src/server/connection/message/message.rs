// This file will define the different types of RTMP messages. Each message type should have its own struct, and there should be an enum that represents any possible message.

// Path: src/server/connection/message.rs

/*
    Handshake
    connect
    createStream
    publish
    play
*/

use bytes::BytesMut;
use bytesio::bytes_reader::BytesReader;
use indexmap::IndexMap;

use crate::server::connection::message::amf0::{amf0_reader::Amf0Reader, define::Amf0ValueType, amf0_writer::Amf0Writer};

use super::amf0::errors::{Amf0ReadError, Amf0WriteError};
use log::{info, error};

#[derive(Debug)]
pub enum RtmpMessage {
    BasicCommand(BasicCommand),
    Connect(ConnectMessage),
    CreateStream(CreateStream),
    _Play(PlayMessage),
    _Pause(PauseMessage),
    ResultObject(ResultObject),
    SetChunkSize(SetChunkSizeMessage),
    Acknowledgement(AcknowledgementMessage),
    ReleaseStream(ReleaseStream),
    FCPublish(FCPublish),
    Publish(Publish),
    Event(Event),
    OnStatus(OnStatus),
    SetDataFrame(SetDataFrame),
    VideoData(VideoData),
    AudioData(AudioData),
    // Add other message types as needed
}

#[derive(Debug)]
pub struct AudioData {
    pub stream_id: u32,
    pub data: Vec<u8>
}

impl AudioData {
    pub fn new(stream_id: u32, data: Vec<u8>) -> AudioData {
        AudioData {
            stream_id: stream_id,
            data: data
        }
    }
}

#[derive(Debug)]
pub struct VideoData {
    pub stream_id: u32,
    pub data: Vec<u8>
}

impl VideoData {
    pub fn new(stream_id: u32, data: Vec<u8>) -> VideoData {
        VideoData {
            stream_id: stream_id,
            data: data
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub event_type: u16,
    pub stream_id: u32
}

impl Event {
    pub fn new(event_type: u16, stream_id: u32) -> Event {
        Event {
            event_type: event_type,
            stream_id: stream_id
        }
    }

    pub fn parse(&self) -> [u8; 6] {
        let mut buffer: [u8; 6] = [0; 6];

        let event_type_as_bytes = self.event_type.to_be_bytes();
        let stream_id_as_bytes  = self.stream_id.to_be_bytes();

        buffer[0..2].copy_from_slice(&event_type_as_bytes);
        buffer[2..].copy_from_slice(&stream_id_as_bytes);

        buffer
    }
}

#[derive(Debug)]
pub struct Publish {
    pub command_name: String,
    pub transaction_id: usize,
    pub amf0_null: Amf0ValueType,
    pub stream_key: String,
    pub stream_type: String
}

impl Publish {
    pub fn new(command_name: String, transaction_id: usize, amf0_null: Amf0ValueType, stream_key: String, stream_type: String) -> Publish {
        Publish {
            command_name: command_name,
            transaction_id: transaction_id,
            amf0_null: amf0_null,
            stream_key: stream_key,
            stream_type: stream_type
        }
    }
    
    pub fn parse(data: &[u8]) -> Result<Publish, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let decoded_msg = reader.read_all().unwrap();
        let command_name = match decoded_msg.get(0) {
            Some(Amf0ValueType::UTF8String(command_name)) => command_name.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command name")))
        };
        let transaction_id = match decoded_msg.get(1) {
            Some(Amf0ValueType::Number(transaction_id)) => *transaction_id,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction id")))
        };
        let amf0_null = match decoded_msg.get(2) {
            Some(Amf0ValueType::Null) => Amf0ValueType::Null,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected null")))
        };
        let stream_key = match decoded_msg.get(3) {
            Some(Amf0ValueType::UTF8String(stream_key)) => stream_key.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected stream key")))
        };
        let stream_type = match decoded_msg.get(4) {
            Some(Amf0ValueType::UTF8String(stream_type)) => stream_type.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected stream type")))
        };
        Ok(Publish::new(command_name, transaction_id as usize, amf0_null, stream_key, stream_type))
    }
}

#[derive(Debug)]
pub struct FCPublish {
    pub command_name: String,
    pub transaction_id: usize,
    pub amf0_null: Amf0ValueType,
    pub stream_key: String,
    pub stream_id: Option<u8>
}

impl FCPublish {
    pub fn new(command_name: String, transaction_id: usize, amf0_null: Amf0ValueType, stream_key: String) -> FCPublish {
        FCPublish {
            command_name: command_name,
            transaction_id: transaction_id,
            amf0_null: amf0_null,
            stream_key: stream_key,
            stream_id: None
        }
    }
    
    pub fn parse(data: &[u8]) -> Result<FCPublish, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let decoded_msg = reader.read_all().unwrap();
        let command_name = match decoded_msg.get(0) {
            Some(Amf0ValueType::UTF8String(command_name)) => command_name.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command name")))
        };
        let transaction_id = match decoded_msg.get(1) {
            Some(Amf0ValueType::Number(transaction_id)) => *transaction_id as usize,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction id")))
        };
        let amf0_null = match decoded_msg.get(2) {
            Some(Amf0ValueType::Null) => Amf0ValueType::Null,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected null")))
        };
        let stream_key = match decoded_msg.get(3) {
            Some(Amf0ValueType::UTF8String(stream_key)) => stream_key.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected stream key")))
        };
        Ok(FCPublish::new(command_name, transaction_id, amf0_null, stream_key))
    }
}

#[derive(Debug)]
pub struct ReleaseStream {
    pub command_name: String,
    pub transaction_id: usize,
    pub amf0_null: Amf0ValueType,
    pub stream_key: String,
}

impl ReleaseStream {
    pub fn new(command_name: String, transaction_id: usize, amf0_null: Amf0ValueType, stream_key: String) -> ReleaseStream {
        ReleaseStream {
            command_name: command_name,
            transaction_id: transaction_id,
            amf0_null: amf0_null,
            stream_key: stream_key,
        }
    }

    pub fn parse(data: &[u8]) -> Result<ReleaseStream, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let decoded_msg = reader.read_all()?;
        let command_name = match decoded_msg.get(0) {
            Some(Amf0ValueType::UTF8String(command_name)) => command_name.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command name"))),
        };
        let transaction_id = match decoded_msg.get(1) {
            Some(Amf0ValueType::Number(transaction_id)) => *transaction_id as usize,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction id"))),
        };
        let amf0_null = match decoded_msg.get(2) {
            Some(Amf0ValueType::Null) => Amf0ValueType::Null,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected null"))),
        };
        
        let stream_name = match decoded_msg.get(3) {
            Some(Amf0ValueType::UTF8String(stream_name)) => stream_name.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected stream name"))),
        };

        Ok(ReleaseStream::new(command_name, transaction_id, amf0_null, stream_name))
    }
}

#[derive(Debug)]
pub struct OnStatusObject {
    pub level: String,
    pub code: String,
    pub description: String,
}

impl OnStatusObject {
    pub fn default() -> OnStatusObject {
        OnStatusObject {
            level: "status".to_owned(),
            code: "NetStream.Publish.Start".to_owned(),
            description: "[/] Publishing stream . . .".to_owned(),
        }
    }

    pub fn parse(&self) -> IndexMap<String, Amf0ValueType> {
        let mut writer = Amf0Writer::new(bytesio::bytes_writer::BytesWriter::new());
        let mut obj_map = IndexMap::new();
        obj_map.insert("level".to_owned(), Amf0ValueType::UTF8String(self.level.to_owned()));
        obj_map.insert("code".to_owned(), Amf0ValueType::UTF8String(self.code.to_owned()));
        obj_map.insert("description".to_owned(), Amf0ValueType::UTF8String(self.description.to_owned()));
        obj_map
    }
}

#[derive(Debug)]
pub struct OnStatus
{
    command_name: String,
    transaction_id: usize,
}

impl OnStatus 
{
    pub fn new(transaction_id: usize) -> OnStatus
    {
        OnStatus {
            command_name: "onStatus".to_owned(),
            transaction_id: transaction_id,

        }
    }

    pub fn parse(&self) -> Result<BytesMut, Box<dyn std::error::Error>>
    {
        let mut writer = Amf0Writer::new(bytesio::bytes_writer::BytesWriter::new());
        writer.write_string(&self.command_name)?;
        let tmp = self.transaction_id as f64;
        writer.write_number(&tmp)?;
        writer.write_null()?;

        let on_status_object = OnStatusObject::default();
        let on_status_data = on_status_object.parse();

        writer.write_object(&on_status_data)?;
        let data = writer.extract_current_bytes();
        Ok(data)
    }
}

#[derive(Debug)]
pub struct SetDataFrame {
    data_name: String,
    metadata: String,
    data: SetDataFrameData
}

impl SetDataFrame {
    pub fn new(data_name: String, metadata: String, data: SetDataFrameData) -> SetDataFrame {
        SetDataFrame {
            data_name: data_name,
            metadata: metadata,
            data: data,
        }
    }

    pub fn parse(data: &[u8]) -> Result<SetDataFrame, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let decoded_msg = reader.read_all()?;
        let data_name = match decoded_msg.get(0) {
            Some(Amf0ValueType::UTF8String(data_name)) => data_name.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected data name"))),
        };
        let metadata = match decoded_msg.get(1) {
            Some(Amf0ValueType::UTF8String(metadata)) => metadata.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected metadata"))),
        };
        let data_obj = match decoded_msg.get(2) {
            Some(Amf0ValueType::Object(data_obj)) => data_obj.to_owned(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected data"))),
        };

        Ok(SetDataFrame::new(data_name, metadata, SetDataFrameData::parse(data_obj).unwrap()))
    } 
}

#[derive(Debug)]
pub struct SetDataFrameData {
    pub duration: f64,
    pub file_size: f64,
    pub width: f64,
    pub height: f64,
    pub video_codec_id: f64,
    pub video_data_rate: f64,
    pub frame_rate: f64,
    pub audio_codec_id: f64,
    pub audio_data_rate: f64,
    pub audio_sample_rate: f64,
    pub audio_sample_size: f64,
    pub audio_channels: f64,
    pub stereo: bool,
    pub two_point_one: bool,
    pub three_point_one: bool,
    pub four_point_zero: bool,
    pub four_point_one: bool,
    pub five_point_one: bool,
    pub seven_point_one: bool,
    pub encoder: String,
}

impl SetDataFrameData {
    pub fn default() -> SetDataFrameData {
        SetDataFrameData {
            duration: 0.0,
            file_size: 0.0,
            width: 0.0,
            height: 0.0,
            video_codec_id: 0.0,
            video_data_rate: 0.0,
            frame_rate: 0.0,
            audio_codec_id: 0.0,
            audio_data_rate: 0.0,
            audio_sample_rate: 0.0,
            audio_sample_size: 0.0,
            audio_channels: 0.0,
            stereo: false,
            two_point_one: false,
            three_point_one: false,
            four_point_zero: false,
            four_point_one: false,
            five_point_one: false,
            seven_point_one: false,
            encoder: "".to_owned(),
        }
    }
    pub fn parse(data: IndexMap<String, Amf0ValueType>) -> Result<SetDataFrameData, Box<dyn std::error::Error>> {
        let mut set_data_frame_data = SetDataFrameData::default();

        for (key, value) in data {
            match key.as_str() {
                "duration" => {
                    match value {
                        Amf0ValueType::Number(duration) => set_data_frame_data.duration = duration,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "fileSize" => {
                    match value {
                        Amf0ValueType::Number(file_size) => set_data_frame_data.file_size = file_size,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "width" => {
                    match value {
                        Amf0ValueType::Number(width) => set_data_frame_data.width = width,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "height" => {
                    match value {
                        Amf0ValueType::Number(height) => set_data_frame_data.height = height,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "videocodecid" => {
                    match value {
                        Amf0ValueType::Number(video_codec_id) => set_data_frame_data.video_codec_id = video_codec_id,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "videodatarate" => {
                    match value {
                        Amf0ValueType::Number(video_data_rate) => set_data_frame_data.video_data_rate = video_data_rate,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "framerate" => {
                    match value {
                        Amf0ValueType::Number(frame_rate) => set_data_frame_data.frame_rate = frame_rate,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "audiocodecid" => {
                    match value {
                        Amf0ValueType::Number(audio_codec_id) => set_data_frame_data.audio_codec_id = audio_codec_id,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "audiodatarate" => {
                    match value {
                        Amf0ValueType::Number(audio_data_rate) => set_data_frame_data.audio_data_rate = audio_data_rate,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "audiosamplerate" => {
                    match value {
                        Amf0ValueType::Number(audio_sample_rate) => set_data_frame_data.audio_sample_rate = audio_sample_rate,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "audiosamplesize" => {
                    match value {
                        Amf0ValueType::Number(audio_sample_size) => set_data_frame_data.audio_sample_size = audio_sample_size,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "audiochannels" => {
                    match value {
                        Amf0ValueType::Number(audio_channels) => set_data_frame_data.audio_channels = audio_channels,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected number"))),
                    }
                },
                "stereo" => {
                    match value {
                        Amf0ValueType::Boolean(stereo) => set_data_frame_data.stereo = stereo,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "2.1" => {
                    match value {
                        Amf0ValueType::Boolean(two_point_one) => set_data_frame_data.two_point_one = two_point_one,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "3.1" => {
                    match value {
                        Amf0ValueType::Boolean(three_point_one) => set_data_frame_data.three_point_one = three_point_one,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "4.0" => {
                    match value {
                        Amf0ValueType::Boolean(four_point_zero) => set_data_frame_data.four_point_zero = four_point_zero,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "4.1" => {
                    match value {
                        Amf0ValueType::Boolean(four_point_one) => set_data_frame_data.four_point_one = four_point_one,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "5.1" => {
                    match value {
                        Amf0ValueType::Boolean(five_point_one) => set_data_frame_data.five_point_one = five_point_one,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "7.1" => {
                    match value {
                        Amf0ValueType::Boolean(seven_point_one) => set_data_frame_data.seven_point_one = seven_point_one,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected boolean"))),
                    }
                },
                "encoder" => {
                    match value {
                        Amf0ValueType::UTF8String(encoder) => set_data_frame_data.encoder = encoder,
                        _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected string"))),
                    }
                },
                _ => {
                    error!("Unexpected key {:?}", key);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unexpected key")))
                },
            }
        }
        Ok(set_data_frame_data)
    }
}

#[derive(Debug)]
pub struct AcknowledgementMessage {
    pub sequence_number: u32,
}

impl AcknowledgementMessage {
    pub fn new(sequence_number: u32) -> AcknowledgementMessage {
        AcknowledgementMessage {
            sequence_number: sequence_number,
        }
    }
}

#[derive(Debug)]
pub struct SetChunkSizeMessage {
    pub chunk_size: u32,
}

impl SetChunkSizeMessage {
    pub fn new(chunk_size: u32) -> SetChunkSizeMessage {
        SetChunkSizeMessage { chunk_size: chunk_size }
    }
}

#[derive(Debug)]
pub struct ResultObject {
    pub command_name: String,
    pub transaction_id: usize,
    pub command_object: Option<CommandObject>,
    pub stream_id: usize,
}

impl ResultObject {
    pub fn new(command_name: String, transaction_id: usize, stream_id: usize) -> ResultObject {
        ResultObject {
            command_name: command_name,
            transaction_id: transaction_id,
            command_object: None,
            stream_id: stream_id,
        }
    }

    pub fn set_command_object(&mut self, command_object: CommandObject) {
        self.command_object = Some(command_object);
    }

    pub fn parse(&self) -> Result<BytesMut, Amf0WriteError> {
        let mut writer = Amf0Writer::new(bytesio::bytes_writer::BytesWriter::new());
        writer.write_any(&Amf0ValueType::UTF8String(self.command_name.clone())).unwrap();
        writer.write_any(&Amf0ValueType::Number(self.transaction_id as f64)).unwrap();
        if Option::is_some(&self.command_object) {
            let mut command_obj_map = IndexMap::new();
            
            command_obj_map.insert("fmsVer".to_string(), Amf0ValueType::UTF8String(self.command_object.as_ref().unwrap().fms_ver.clone()));
            command_obj_map.insert("capabilities".to_string(), Amf0ValueType::Number(self.command_object.as_ref().unwrap().capabilities as f64));
            
            writer.write_any(&Amf0ValueType::Object(command_obj_map)).unwrap();
        } else {
            writer.write_any(&Amf0ValueType::Null).unwrap();
        }
        writer.write_any(&Amf0ValueType::Number(self.stream_id as f64)).unwrap();
        let tmp = writer.extract_current_bytes();
        Ok(tmp)
    }
}

#[derive(Debug)]
pub struct CommandObject {
    fms_ver: String,
    capabilities: usize,
}

impl CommandObject {
    pub fn new(fms_ver: String, capabilities: usize) -> CommandObject {
        let command_object = CommandObject {
            fms_ver: fms_ver,
            capabilities: capabilities,
        };
        command_object
    }
}

#[derive(Debug)]
pub struct ConnectObject {
    pub app: String,
    pub flash_ver: String,
    pub swf_url: String,
    pub tc_url: String,
    pub stream_type: String,
}

impl ConnectObject {
    pub fn _new(app: String, tc_url: String, flash_ver: String, sw_url: String, stream_type: String) -> ConnectObject {
        ConnectObject {
            app: app,
            flash_ver: flash_ver,
            swf_url: sw_url,
            tc_url: tc_url,
            stream_type: stream_type,
        }
    }

    fn default() -> ConnectObject {
        ConnectObject {
            app: "".to_string(),
            flash_ver: "".to_string(),
            swf_url: "".to_string(),
            tc_url: "".to_string(),
            stream_type: "".to_string(),
        }
    }

    pub fn parse(data: IndexMap<String, Amf0ValueType>) -> Result<ConnectObject, Box<dyn std::error::Error>> {
        let mut connect_object = ConnectObject::default();

        // Read the command object
        for (key, value) in data {
            match key.as_str() {
                "app" => {
                    if let Amf0ValueType::UTF8String(s) = value {
                        connect_object.app = s;
                    }
                }
                "flashVer" => {
                    if let Amf0ValueType::UTF8String(s) = value {
                        connect_object.flash_ver = s;
                    }
                }
                "tcUrl" => {
                    if let Amf0ValueType::UTF8String(s) = value {
                        connect_object.tc_url = s;
                    }
                }
                "swfUrl" => {
                    if let Amf0ValueType::UTF8String(s) = value {
                        connect_object.swf_url = s;
                    }
                }
                "type" => {
                    if let Amf0ValueType::UTF8String(s) = value {
                        connect_object.stream_type = s;
                    }
                }
                _ => {}
            }
        }
            


        Ok(connect_object)
    }
}

#[derive(Debug)]
pub struct BasicCommand {
    pub command_name: String
}

impl BasicCommand {
    pub fn new(command_name: String) -> BasicCommand {
        BasicCommand {
            command_name: command_name,
        }
    }

    pub fn parse(data: &[u8]) -> Result<BasicCommand, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));

        let decoded_msg = reader.read_all()?;

        let command_name = match decoded_msg.get(0) {
            Some(&Amf0ValueType::UTF8String(ref s)) => s.clone(),
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid command name"))),
        };

        Ok(BasicCommand::new(command_name))
    }
}

#[derive(Debug)]
pub struct ConnectMessage {
    pub connect_object: ConnectObject,
    pub id: usize,
}

impl ConnectMessage {
    pub fn new(id: usize, connect_object: ConnectObject) -> ConnectMessage {
        ConnectMessage {
            connect_object: connect_object,
            id: id,
        }
    }

    pub fn parse(data: &[u8]) -> Result<ConnectMessage, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let mut connect_message = ConnectMessage::new(0, ConnectObject::default());

        let decoded_msg = reader.read_all()?;

        connect_message.id = match decoded_msg.get(1) {
            Some(&Amf0ValueType::Number(n)) => n as usize,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction ID")))
        };

        let decoded_obj = match decoded_msg.get(2) {
        Some(&Amf0ValueType::Object(ref obj)) => obj,
        _ => {
                error!("Failed to get command object from decoded message: {:?}", decoded_msg);
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command object")))
            }
        };
        connect_message.connect_object = ConnectObject::parse(decoded_obj.clone())?;
        Ok(connect_message)
    }
}

#[derive(Debug)]
pub struct CreateStream {
    pub command_name: String,
    pub transaction_id: usize,
    pub amf0_null: Amf0ValueType,
}

impl CreateStream {
    pub fn new(transaction_id: usize) -> CreateStream {
        CreateStream {
            command_name: "createStream".to_string(),
            transaction_id: transaction_id,
            amf0_null: Amf0ValueType::Null,
        }
    }

    pub fn parse(data: &[u8]) -> Result<CreateStream, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));

        let decoded_msg = reader.read_all()?;

        let transaction_id = match decoded_msg.get(1) {
            Some(&Amf0ValueType::Number(n)) => n as usize,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction ID")))
        };

        Ok(CreateStream::new(transaction_id))
    }
}

#[derive(Debug)]
pub struct PlayMessage {
    // Define the fields for a Play message
    // For example:
    pub stream_name: String,
    // Add other fields as needed
}

#[derive(Debug)]
pub struct PauseMessage {
    // Define the fields for a Pause message
    // For example:
    pub is_paused: bool,
    // Add other fields as needed
}
