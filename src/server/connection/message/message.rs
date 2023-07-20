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

use crate::server::connection::message::amf0::{amf0_reader::Amf0Reader, define::Amf0ValueType};

#[derive(Debug)]
pub enum RtmpMessage {
    Connect(ConnectMessage),
    _CreateStream(CreateStreamMessage),
    _Play(PlayMessage),
    _Pause(PauseMessage),
    // Add other message types as needed
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
            println!("Failed to get command object from decoded message: {:?}", decoded_msg);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected command object")))
        }
    };
        connect_message.connect_object = ConnectObject::parse(decoded_obj.clone())?;
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
