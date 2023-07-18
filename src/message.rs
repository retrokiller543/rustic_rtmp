// This file will define the different types of RTMP messages. Each message type should have its own struct, and there should be an enum that represents any possible message.

// Path: src/message.rs

pub enum RtmpMessage {
    Connect(ConnectMessage),
    CreateStream(CreateStreamMessage),
    Play(PlayMessage),
    Pause(PauseMessage),
    // Add other message types as needed
}

pub struct ConnectMessage {
    // Define the fields for a Connect message
    // For example:
    pub app: String,
    // Add other fields as needed
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