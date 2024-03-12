use crate::slack::{create_slack_client, establish_websocket_connection, get_websocket_endpoint};

mod slack;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .init();

    let client = create_slack_client();
    let ws_url = get_websocket_endpoint(&client)
        .await
        .expect("Failed to get websocket endpoint");
    let mut socket = establish_websocket_connection(&ws_url);

    loop {
        let msg = socket.read().expect("Failed to read message");
        println!("Received: {}", msg);
    }
}
