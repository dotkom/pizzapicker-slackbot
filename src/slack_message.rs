pub mod incoming {
    use serde::Deserialize;
    use uuid::Uuid;

    /// Base type for any incoming message that can be received from Slack
    ///
    /// Please refer to [Slack's documentation](https://api.slack.com/apis/connections/socket#events)
    #[derive(Deserialize, Debug)]
    pub struct Incoming<Payload> {
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

    /// An incoming message from Slack that indicates a successful connection
    #[derive(Deserialize, Debug)]
    pub struct SlackHelloIncomingMessage {
        pub num_connections: u32,
    }

    /// An incoming Slash Command message
    #[derive(Deserialize, Debug)]
    pub struct SlashCommandIncomingMessage {
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
    pub enum SlackIncomingMessage {
        #[serde(alias = "slash_commands")]
        SlashCommands(Box<Incoming<SlashCommandIncomingMessage>>),
        #[serde(alias = "disconnect")]
        Disconnect(Box<SlackDisconnectIncomingMessage>),
        #[serde(alias = "hello")]
        Hello(Box<SlackHelloIncomingMessage>),
    }
}

pub mod outgoing {
    use serde::Serialize;
    use uuid::Uuid;

    /// A single text segment in a Slash Command message response
    #[derive(Serialize, Debug)]
    pub struct SlackCommandBlockText {
        pub r#type: String,
        pub text: String,
    }

    /// A block in a Slash Command message response
    #[derive(Serialize, Debug)]
    pub struct SlackCommandBlock {
        pub r#type: String,
        pub text: SlackCommandBlockText,
    }

    /// Outgoing message for a Slash Command, according to https://api.slack.com/messaging/composing
    #[derive(Serialize, Debug)]
    pub struct SlashCommandOutgoingMessage {
        pub blocks: Vec<SlackCommandBlock>,
        pub response_type: String,
    }

    /// Base type for any outgoing message that can be sent to Slack
    #[derive(Serialize, Debug)]
    #[serde(untagged)]
    pub enum SlackOutgoingMessage {
        SlashCommand(Outgoing<SlashCommandOutgoingMessage>),
    }

    /// Acknowledgement message that must be sent to Slack after receiving a message
    #[derive(Serialize, Debug)]
    pub struct Outgoing<Payload> {
        pub envelope_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub payload: Option<Payload>,
    }

    impl<T> Outgoing<T> {
        pub fn new(envelope_id: Uuid, payload: Option<T>) -> Self {
            Self {
                envelope_id,
                payload,
            }
        }
    }
}
