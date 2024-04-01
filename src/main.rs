use crate::healthcheck::build_healthcheck_server;
use crate::slack::build_websocket_client;

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
        build_websocket_client().await;
    });
    let http_handle = tokio::spawn(async move {
        build_healthcheck_server()
            .await
            .expect("Failed to start HTTP server");
    });
    futures::future::join_all(vec![app_handle, http_handle]).await;
}
