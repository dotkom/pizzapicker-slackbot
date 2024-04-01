use crate::healthcheck::start_http_server;
use crate::slack::start_websocket_client;

mod healthcheck;
mod roulette;
mod slack;
mod slack_message;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .init();
    let app_handle = tokio::spawn(async move {
        start_websocket_client().await;
    });
    let http_handle = tokio::spawn(async move {
        start_http_server()
            .await
            .expect("Failed to start HTTP server");
    });
    futures::future::join_all(vec![app_handle, http_handle]).await;
}
