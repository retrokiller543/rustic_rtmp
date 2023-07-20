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

use crate::server::connection::message::amf0::{amf0_reader::Amf0Reader, define::Amf0ValueType};

#[derive(Debug)]
pub enum RtmpMessage {
    Connect(ConnectMessage),
    CreateStream(CreateStreamMessage),
    Play(PlayMessage),
    Pause(PauseMessage),
    // Add other message types as needed
}

#[derive(Debug)]
pub struct ConnectObject {
    pub app: String,
    pub flash_ver: String,
    pub swf_url: String,
    pub tc_url: String,
    pub fpad: bool,
    pub audio_codecs: u16,
    pub video_codecs: u8,
    pub video_function: u8,
    pub page_url: String,
    pub object_encoding: u8,
}

impl ConnectObject {
    pub fn new(app: String, tc_url: String, fpad: bool, audio_codecs: u16, video_codec: u8, video_function: u8, page_url: String, object_encoding: u8, flash_ver: String, sw_url: String) -> ConnectObject {
        ConnectObject {
            app: app,
            flash_ver: flash_ver,
            swf_url: sw_url,
            tc_url: tc_url,
            fpad: fpad,
            audio_codecs: audio_codecs,
            video_codecs: video_codec,
            video_function: video_function,
            page_url: page_url,
            object_encoding: object_encoding,
        }
    }

    fn default() -> ConnectObject {
        ConnectObject {
            app: "".to_string(),
            flash_ver: "".to_string(),
            swf_url: "".to_string(),
            tc_url: "".to_string(),
            fpad: false,
            audio_codecs: 0,
            video_codecs: 0,
            video_function: 1,
            page_url: "".to_string(),
            object_encoding: 0,
        }
    }

    pub fn parse(data: &mut &[u8]) -> Result<ConnectObject, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let mut connect_object = ConnectObject::default();
        // Read the command object
        println!("Parsing command object...");
        match reader.read_any()? {
            Amf0ValueType::Object(obj) => {
                for (key, value) in obj {
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
                        "fpad" => {
                            if let Amf0ValueType::Boolean(b) = value {
                                connect_object.fpad = b;
                            }
                        }
                        "audioCodecs" => {
                            if let Amf0ValueType::Number(n) = value {
                                connect_object.audio_codecs = n as u16;
                            }
                        }
                        "videoCodecs" => {
                            if let Amf0ValueType::Number(n) = value {
                                connect_object.video_codecs = n as u8;
                            }
                        }
                        "videoFunction" => {
                            if let Amf0ValueType::Number(n) = value {
                                connect_object.video_function = n as u8;
                            }
                        }
                        "pageUrl" => {
                            if let Amf0ValueType::UTF8String(s) = value {
                                connect_object.page_url = s;
                            }
                        }
                        "objectEncoding" => {
                            if let Amf0ValueType::Number(n) = value {
                                connect_object.object_encoding = n as u8;
                            }
                        }
                        // ... handle other fields ...
                        _ => {}
                    }
                }
            }
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command object"))),
        }

        Ok(connect_object)
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

    pub fn parse(mut data: &[u8]) -> Result<ConnectMessage, Box<dyn std::error::Error>> {
        let mut reader = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let mut reader_copy = Amf0Reader::new(BytesReader::new(BytesMut::from(&data[..])));
        let tmp = reader_copy.read_all()?;
        println!("tmp: {:?}", tmp);
        let mut connect_message = ConnectMessage::new(0, ConnectObject::default());

        // Read the command name (should be "connect")
        println!("Parsing command name...");
        match reader.read_any()? {
            Amf0ValueType::UTF8String(s) => {
                
                if s != "connect" {
                    println!("Expected 'connect' command, got '{}'", s);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected 'connect' command")));
                }
            }
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected 'connect' command"))),
        }

        // Read the transaction ID
        println!("Parsing transaction ID...");
        match reader.read_any()? {
            Amf0ValueType::Number(n) => {
                println!("Transaction ID: {}", n);
                connect_message.id = n as usize;
            }
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected transaction ID"))),
        }

        let connect_object = ConnectObject::parse(&mut data)?;

        connect_message.connect_object = connect_object;

        Ok(connect_message)
    }
}

#[derive(Debug)]
pub struct CreateStreamMessage {
    // Define the fields for a CreateStream message
    // For example:
    pub stream_id: u32,
    // Add other fields as needed
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
