use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Result};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::task::JoinHandle;

use crate::slack_message::{AcknowledgeMessage, SlackMessage};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

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
/// This function returns a Tokio oneshot channel that will be used to send parsed JSON messages
/// from the Slack channel.
///
/// Additionally, this function will automatically reconnect to the websocket if the server sends
/// a disconnect message.
///
/// TODO: Consider whether we should type out the messages that can be sent over the channel and
///  received over the websocket
#[tracing::instrument]
pub async fn create_subscriber() -> (Receiver<serde_json::Value>, JoinHandle<()>) {
    let client = create_slack_client();
    let (_tx, rx) = oneshot::channel::<serde_json::Value>();
    // Perform the initial websocket connection
    let wss_endpoint = get_websocket_endpoint(&client)
        .await
        .expect("Failed to get websocket endpoint");
    let socket = establish_websocket_connection(&wss_endpoint);

    // Spawn a worker thread to read messages from the channel and translate them into messages
    // to be sent across the oneshot channel
    let future = tokio::spawn(async move {
        let mut socket = socket;
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
            let json = serde_json::from_str::<SlackMessage>(&msg);
            let slack_message = match json {
                Ok(json) => json,
                Err(e) => {
                    tracing::warn!("Failed to parse JSON message from Slack: {}", e);
                    tracing::info!("Received message from Slack: {}", msg);
                    continue;
                }
            };
            match slack_message {
                SlackMessage::Disconnect(message) => {
                    socket.close(None).expect("Failed to close websocket");
                    // If the Socket Mode was disabled, stop trying
                    if message.reason == "link_disabled" {
                        tracing::info!("Link disabled, stopping bot");
                        break;
                    }
                    // Otherwise, reconnect to the websocket with a new endpoint
                    tracing::info!("Reconnecting to Slack websocket");
                    let new_wss_endpoint = get_websocket_endpoint(&client)
                        .await
                        .expect("Failed to get websocket endpoint");
                    socket = establish_websocket_connection(&new_wss_endpoint);
                    continue;
                }
                SlackMessage::SlashCommands(message) => {
                    tracing::info!("Received slash command: {:?}", message);
                    let ack = AcknowledgeMessage::new(message.envelope_id);
                    socket
                        .send(Message::Text(serde_json::to_string(&ack).unwrap()))
                        .expect("Failed to send acknowledge message");
                }
                SlackMessage::Hello(_) => tracing::info!("Received hello message from Slack"),
            };
        }
    });

    (rx, future)
}
