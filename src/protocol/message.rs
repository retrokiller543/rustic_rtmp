// This file will define the different types of RTMP messages. Each message type should have its own struct, and there should be an enum that represents any possible message.

// Path: src/message.rs

/*
    Handshake
    connect
    createStream
    publish
    play
*/

pub enum RtmpMessage {
    Connect(ConnectMessage),
    CreateStream(CreateStreamMessage),
    Play(PlayMessage),
    Pause(PauseMessage),
    // Add other message types as needed
}

pub struct ConnectObject {
    pub app: String,
    pub tc_url: String,
    pub fpad: bool,
    pub audio_codecs: u16,
    pub video_codecs: u8,
    pub video_function: bool,
    pub page_url: String,
    pub object_encoding: u8,
}

impl ConnectObject {
    pub fn new(app: String, tc_url: String, fpad: bool, audio_codecs: u16, video_codec: u8, video_function: bool, page_url: String, object_encoding: u8) -> ConnectObject {
        ConnectObject {
            app: app,
            tc_url: tc_url,
            fpad: fpad,
            audio_codecs: audio_codecs,
            video_codecs: video_codec,
            video_function: video_function,
            page_url: page_url,
            object_encoding: object_encoding,
        }
    }
}

pub struct ConnectMessage {
    pub connect_object: ConnectObject,
    pub id: u32,
}

impl ConnectMessage {
    pub fn new(id: u32) -> ConnectMessage {
        let connect_object = ConnectObject::new("live".to_string(), "rtmp://localhost/live".to_string(), false, 0x0FFF, 0x00FF, false, "http://localhost:8080/live".to_string(), 3);
        ConnectMessage {
            connect_object: connect_object,
            id: id,
        }
    }
}

pub struct CreateStreamMessage {
    // Define the fields for a CreateStream message
    // For example:
    pub stream_id: u32,
    // Add other fields as needed
}

pub struct PlayMessage {
    // Define the fields for a Play message
    // For example:
    pub stream_name: String,
    // Add other fields as needed
}

pub struct PauseMessage {
    // Define the fields for a Pause message
    // For example:
    pub is_paused: bool,
    // Add other fields as needed
}