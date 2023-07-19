// This file will define the different types of RTMP messages. Each message type should have its own struct, and there should be an enum that represents any possible message.

// Path: src/server/connection/message.rs

/*
    Handshake
    connect
    createStream
    publish
    play
*/

mod amf_util;
use amf_util::decode_amf;

use std::io::Read;

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

    fn parse(cursor: &mut std::io::Cursor<&[u8]>) -> Result<ConnectObject, Box<dyn std::error::Error>> {
        // Create an empty ConnectObject to hold the parsed fields.
        let mut connect_object = ConnectObject::default();
    
        // Get the total length of the data.
        let data_len = cursor.get_ref().len() as u64;
    
        // Loop over the data, reading one field at a time.
        while cursor.position() < data_len {
            // Create a new cursor for this iteration.
            let mut iter_cursor = cursor.clone();
    
            // Read the next field name from the data.
            let (field_name, bytes_read) = read_string(&mut iter_cursor)?;
            println!("Field name: {}", field_name);
    
            // Read the next field value from the data.
            let amf_value = Amf0Value::read_from(&mut iter_cursor)?;
            println!("Field value: {:?}", amf_value);
    
            // Match on the field name to determine which property to assign the value to.
            match field_name.as_str() {
                // testing some fields to see if they are being read correctly
                "app" => {
                    if let Amf0Value::String(s) = amf_value {
                        connect_object.app = s;
                    } else {
                        return Err("Expected 'app' to be a string".into());
                    }
                }
                "flashVer" => {
                    if let Amf0Value::String(s) = amf_value {
                        connect_object.flash_ver = s;
                    } else {
                        return Err("Expected 'flashVer' to be a string".into());
                    }
                }
                "swfUrl" => {
                    if let Amf0Value::String(s) = amf_value {
                        connect_object.swf_url = s;
                    } else {
                        return Err("Expected 'swfUrl' to be a string".into());
                    }
                }
                "tcUrl" => {
                    if let Amf0Value::String(s) = amf_value {
                        connect_object.tc_url = s;
                    } else {
                        return Err("Expected 'tcUrl' to be a string".into());
                    }
                }
                "fpad" => {
                    if let Amf0Value::Boolean(b) = amf_value {
                        connect_object.fpad = b;
                    } else {
                        return Err("Expected 'fpad' to be a boolean".into());
                    }
                }
                "audioCodecs" => {
                    if let Amf0Value::Number(n) = amf_value {
                        connect_object.audio_codecs = n as u16;
                    } else {
                        return Err("Expected 'audioCodecs' to be a number".into());
                    }
                }
                "videoCodecs" => {
                    if let Amf0Value::Number(n) = amf_value {
                        connect_object.video_codecs = n as u8;
                    } else {
                        return Err("Expected 'videoCodecs' to be a number".into());
                    }
                }
                "videoFunction" => {
                    if let Amf0Value::Number(n) = amf_value {
                        connect_object.video_function = n as u8;
                    } else {
                        return Err("Expected 'videoFunction' to be a number".into());
                    }
                }
                "pageUrl" => {
                    if let Amf0Value::String(s) = amf_value {
                        connect_object.page_url = s;
                    } else {
                        return Err("Expected 'pageUrl' to be a string".into());
                    }
                }
                "objectEncoding" => {
                    if let Amf0Value::Number(n) = amf_value {
                        connect_object.object_encoding = n as u8;
                    } else {
                        return Err("Expected 'objectEncoding' to be a number".into());
                    }
                }
                // Handle other fields similarly...
                _ => {}
            }
    
            // Update the original cursor's position to match the iter_cursor's position.
            cursor.set_position(iter_cursor.position());
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
        let mut cursor = std::io::Cursor::new(data);
        // Read the command name (8 bytes) and convert it to a string
        let mut command_name_bytes = [0; 7];
        cursor.read_exact(&mut command_name_bytes)?;
        let command_name = String::from_utf8_lossy(&command_name_bytes).to_string();
        println!("Command Name: {}", command_name);

        // Set the transaction ID to 1
        let mut transaction_id_bytes = [0; 1];
        let transaction_id = cursor.read(&mut transaction_id_bytes)?;
        println!("Transaction ID: {}", transaction_id);

        // Read the command object
        let connect_object = ConnectObject::parse(&mut cursor)?;
        
        Ok(ConnectMessage {
            id: transaction_id,
            connect_object,
        })
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

use amf::amf0::Value as Amf0Value;
use byteorder::{ReadBytesExt, LittleEndian};

fn read_string(cursor: &mut std::io::Cursor<&[u8]>) -> Result<(String, usize), Box<dyn std::error::Error>> {
    let length = cursor.read_u16::<LittleEndian>()? as usize;
    let bytes_left = cursor.get_ref().len() - cursor.position() as usize;
    println!("Length: {}, bytes left: {}", length, bytes_left);
    let mut buffer = vec![0; length];
    cursor.read_exact(&mut buffer)?;
    let s = String::from_utf8(buffer)?;
    Ok((s, length + 2))  // Return the string and the number of bytes read
}


fn read_boolean(cursor: &mut std::io::Cursor<&[u8]>) -> Result<bool, Box<dyn std::error::Error>> {
    // Decode the AMF data.
    let amf_value = Amf0Value::read_from(cursor)?;
    

    // Try to convert the AMF value to a boolean.
    match amf_value {
        Amf0Value::Boolean(b) => Ok(b),
        _ => Err("Expected AMF boolean".into()),
    }
}

fn read_number(cursor: &mut std::io::Cursor<&[u8]>) -> Result<f64, Box<dyn std::error::Error>> {
    // Decode the AMF data.
    let amf_value = Amf0Value::read_from(cursor)?;

    // Try to convert the AMF value to a number.
    match amf_value {
        Amf0Value::Number(n) => Ok(n),
        _ => Err("Expected AMF number".into()),
    }
}

fn read_u8(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u8, Box<dyn std::error::Error>> {
    // Decode the AMF data.
    let amf_value = Amf0Value::read_from(cursor)?;

    // Try to convert the AMF value to a u8.
    match amf_value {
        Amf0Value::Number(n) => Ok(n as u8),
        _ => Err("Expected AMF number".into()),
    }
}

fn read_u16(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u16, Box<dyn std::error::Error>> {
    // Decode the AMF data.
    let amf_value = Amf0Value::read_from(cursor)?;

    // Try to convert the AMF value to a u16.
    match amf_value {
        Amf0Value::Number(n) => Ok(n as u16),
        _ => Err("Expected AMF number".into()),
    }
}
