use crate::roulette::{get_random_fortune_phrase, get_random_pizza, SpinMode};
use crate::slack_message::incoming;
use crate::slack_message::outgoing;
use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Result};
use std::collections::HashMap;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

#[derive(Debug)]
struct Entry {
    count: u32,
    last: chrono::DateTime<Utc>,
}

#[derive(Debug)]
struct Stats(HashMap<String, Entry>);

impl Stats {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn entry(&self, user: &str) -> Option<&Entry> {
        let entry = self.0.get(user)?;
        if Utc::now().signed_duration_since(entry.last) <= Duration::try_days(2)? {
            Some(entry)
        } else {
            None
        }
    }

    pub fn spin_counts(&self, user: &str) -> u32 {
        if let Some(entry) = self.entry(user) {
            entry.count
        } else {
            0
        }
    }

    pub fn add_spin(&mut self, user: &str) {
        let count = if let Some(current) = self.entry(user) {
            current.count + 1
        } else {
            1
        };

        self.0.insert(
            user.to_string(),
            Entry {
                count,
                last: Utc::now(),
            },
        );
    }
}

/// Get the websocket endpoint for the slack bot
///
/// This function makes a request to the apps.connections.open endpoint to get the websocket
/// endpoint for the slack bot. It requires the Slack integration to have Socket Mode enabled.
#[tracing::instrument]
pub async fn get_websocket_endpoint(client: &Client) -> Result<String> {
    tracing::info!("Requesting websocket endpoint from Slack");
    let response = client
        .post("https://slack.com/api/apps.connections.open")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let url = response["url"].as_str().expect("url not found in response");
    tracing::info!("Received websocket endpoint: {}", url);
    Ok(url.to_string())
}

/// Connect to a given websocket endpoint
#[tracing::instrument]
fn establish_websocket_connection(url: &str) -> SlackWebSocket {
    let (socket, response) = tungstenite::connect(url).expect("Failed to connect to websocket");
    tracing::info!(
        "Request for websocket connection returned {}",
        response.status()
    );
    socket
}

type SlackWebSocket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

fn create_slack_client() -> Client {
    let app_token = std::env::var("SLACK_APP_TOKEN").expect("SLACK_APP_TOKEN must be set");
    let mut default_headers = HeaderMap::new();
    let bearer_token = format!("Bearer {}", app_token);
    let mut authorization_header = HeaderValue::from_str(&bearer_token).unwrap();
    authorization_header.set_sensitive(true);
    default_headers.insert("Authorization", authorization_header);

    Client::builder()
        .default_headers(default_headers)
        .build()
        .expect("Failed to create reqwest client")
}

/// Create a channel that emits messages from the websocket
///
///
/// This function will automatically reconnect to the websocket if the server sends
/// a disconnect message.
#[tracing::instrument]
pub async fn start_websocket_client() -> () {
    let client = create_slack_client();
    // Perform the initial websocket connection
    let wss_endpoint = get_websocket_endpoint(&client)
        .await
        .expect("Failed to get websocket endpoint");
    let mut socket = establish_websocket_connection(&wss_endpoint);

    let mut stats = Stats::new();

    // Spawn a worker thread to read messages from the channel and translate them into messages
    // to be sent across the oneshot channel
    loop {
        let msg = socket.read().expect("Failed to read from websocket");
        // The Slack API promises that messages are sent as JSON, so we can safely assume that
        // the message is a JSON string
        let msg = match msg {
            Message::Text(msg) => msg,
            Message::Ping(_) => {
                tracing::debug!("Received ping from Slack websocket");
                socket
                    .send(Message::Pong(vec![]))
                    .expect("Failed to send pong");
                continue;
            }
            _ => {
                tracing::warn!("Received non-Text message from Slack websocket");
                continue;
            }
        };

        // If the server requests a regular disconnect, we should reconnect, but if the server
        // sends a link_disallowed message, we should stop the bot
        let json = serde_json::from_str::<incoming::SlackIncomingMessage>(&msg);
        if let Err(e) = json {
            tracing::warn!(
                "Failed to parse JSON message from Slack: {} from JSON {}",
                e,
                msg
            );
            continue;
        }
        match json.unwrap() {
            incoming::SlackIncomingMessage::Disconnect(message) => {
                // We always want to close the connection right away. Then we can decide if we want
                // to reconnect or not.
                socket.close(None).expect("Failed to close websocket");
                if let Some(new_socket) = handle_disconnect_message(*message, &client).await {
                    socket = new_socket;
                    continue;
                }
                break;
            }
            incoming::SlackIncomingMessage::SlashCommands(message) => {
                tracing::info!("Received slash command: {:?}", message);
                let response = handle_slash_command(*message, &mut stats).await;
                if let Some(response) = response {
                    let json =
                        serde_json::to_string(&response).expect("Failed to serialize response");
                    tracing::info!("Sending response to Slack: {}", json);
                    socket
                        .send(Message::Text(json))
                        .expect("Failed to send response to Slack");
                }
            }
            incoming::SlackIncomingMessage::Hello(_) => {
                tracing::info!("Received hello message from Slack")
            }
        };
    }
}

async fn handle_disconnect_message(
    message: incoming::SlackDisconnectIncomingMessage,
    client: &Client,
) -> Option<SlackWebSocket> {
    tracing::info!("Received disconnect message: {:?}", message.reason.as_str());

    match message.reason.as_str() {
        "link_disabled" => {
            tracing::info!("Link disabled, stopping bot");
            None
        }
        _ => {
            tracing::info!("Reconnecting to Slack websocket");
            let new_wss_endpoint = get_websocket_endpoint(client)
                .await
                .expect("Failed to get websocket endpoint");
            Some(establish_websocket_connection(&new_wss_endpoint))
        }
    }
}

fn mention_user(user_id: &str) -> String {
    format!("<@{}>", user_id)
}

#[tracing::instrument]
async fn handle_slash_command(
    message: incoming::Incoming<incoming::SlashCommandIncomingMessage>,
    stats: &mut Stats,
) -> Option<outgoing::SlackOutgoingMessage> {
    let spin_mode = SpinMode::from_command(&message.payload.command).or({
        tracing::warn!("Received unknown command: {}", message.payload.command);
        None
    })?;

    let pizza = get_random_pizza(spin_mode);
    let mention = mention_user(&message.payload.user_id);
    let fortune_phrase = get_random_fortune_phrase();

    let spins = stats.spin_counts(&message.payload.user_id);
    let spin_msg = match spins {
        0 => "Du har *én gratis* respin igjen.".to_string(),
        1 => "Du har *ingen gratis* respins igjen.".to_string(),
        x => format!(
            "Du har spunnet {} ganger. Straffer: {}",
            x + 1,
            "🍺".repeat((x - 1) as usize)
        ),
    };
    stats.add_spin(&message.payload.user_id);

    let outgoing_message = outgoing::SlashCommandOutgoingMessage {
        response_type: "in_channel".to_string(),
        blocks: vec![
            outgoing::SlackCommandBlock {
                r#type: "section".to_string(),
                text: outgoing::SlackCommandBlockText {
                    r#type: "mrkdwn".to_string(),
                    text: format!("{} {} *{}* 🎉", mention, fortune_phrase.phrase, pizza.name),
                },
            },
            outgoing::SlackCommandBlock {
                r#type: "section".to_string(),
                text: outgoing::SlackCommandBlockText {
                    r#type: "mrkdwn".to_string(),
                    text: format!("Pizzaen består av {} ({})", pizza.description, pizza.extra),
                },
            },
            outgoing::SlackCommandBlock {
                r#type: "section".to_string(),
                text: outgoing::SlackCommandBlockText {
                    r#type: "mrkdwn".to_string(),
                    text: spin_msg,
                },
            },
        ],
    };

    Some(outgoing::SlackOutgoingMessage::SlashCommand(
        outgoing::Outgoing::new(message.envelope_id, Some(outgoing_message)),
    ))
}
