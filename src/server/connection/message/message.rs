// This file will define the different types of RTMP messages. Each message type should have its own struct, and there should be an enum that represents any possible message.

// Path: src/server/connection/message.rs

/*
    Handshake
    connect
    createStream
    publish
    play
*/


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
        let mut connect_object = ConnectObject::default();
    
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
        let connect_message = ConnectMessage::new(0, ConnectObject::default());
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


