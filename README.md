# Rustic RTMP

A Rust implementation of the RTMP protocol to handle streaming.

## What needs to be done
RTMP (Real-Time Messaging Protocol) is a protocol developed by Macromedia (now owned by Adobe) for the transmission of audio, video, and other data over the Internet. It's primarily used for live streaming and is based on TCP, which means it prioritizes the reliability of the stream over latency.

RTMP consists of several parts:

    Handshake: This is the initial part of the connection where the client and server exchange information and agree to communicate.

    Chunk Stream: RTMP messages are divided into smaller chunks which are then sent over the network. This allows for the interleaving of data and helps to prevent large messages from blocking the delivery of other messages.

    Messages: These are the actual data being sent. They can be audio, video, or other types of data.

    AMF (Action Message Format): This is a binary format used to encode the messages. It's similar to JSON, but binary.

    Create a TCP Listener: This will allow your server to accept incoming TCP connections.

    Handle the Handshake: When a client connects, you'll need to perform the RTMP handshake. This involves exchanging a series of messages with the client.

    Process Chunk Streams: Once the handshake is complete, you'll start receiving chunk streams from the client. You'll need to reassemble these into complete RTMP messages.

    Handle Messages: Depending on the type of message, you might need to do different things. For example, audio and video messages will need to be sent to whoever is watching the stream.

    Send Messages: You'll also need to be able to send messages to the client, such as control messages or responses to commands.

    Encode/Decode AMF: You'll need to be able to encode and decode AMF data. There might be existing Rust libraries that can do this, or you might need to implement it yourself.

