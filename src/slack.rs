use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Result};

use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

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
pub fn establish_websocket_connection(url: &str) -> WebSocket<MaybeTlsStream<std::net::TcpStream>> {
    let (socket, response) = tungstenite::connect(url).expect("Failed to connect to websocket");
    tracing::info!(
        "Request for websocket connection returned {}",
        response.status()
    );
    socket
}

pub fn create_slack_client() -> Client {
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
