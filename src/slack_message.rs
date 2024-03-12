use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base type for any incoming message that can be received from Slack
///
/// Please refer to [Slack's documentation](https://api.slack.com/apis/connections/socket#events)
#[derive(Deserialize, Debug)]
pub struct SlackIncomingMessage<Payload> {
    pub payload: Payload,
    pub envelope_id: Uuid,
    pub accepts_response_payload: bool,
}

/// An incoming message from Slack that indicates a disconnection
///
/// This message is sent from Slack when the websocket connection is about to be closed.
///
/// This implementation does not account for the debug_info field, as it is not currently used in
/// this application.
#[derive(Deserialize, Debug)]
pub struct SlackDisconnectIncomingMessage {
    pub reason: String,
}

#[derive(Deserialize, Debug)]
pub struct SlackHelloIncomingMessage {
    pub num_connections: u32,
}

/// An incoming Slash Command message
#[derive(Deserialize, Debug)]
pub struct SlashCommandMessagePayload {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
}

/// An incoming message from Slack
///
/// This type is not exhaustive across all possible message types from Slack, and only includes
/// message types that are currently used in this application.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SlackMessage {
    #[serde(alias = "slash_commands")]
    SlashCommands(Box<SlackIncomingMessage<SlashCommandMessagePayload>>),
    #[serde(alias = "disconnect")]
    Disconnect(Box<SlackDisconnectIncomingMessage>),
    #[serde(alias = "hello")]
    Hello(Box<SlackHelloIncomingMessage>),
}

/// Acknowledgement message that must be sent to Slack after receiving a message
#[derive(Serialize, Debug)]
pub struct AcknowledgeMessage {
    pub envelope_id: Uuid,
}

impl AcknowledgeMessage {
    pub fn new(envelope_id: Uuid) -> Self {
        Self { envelope_id }
    }
}
