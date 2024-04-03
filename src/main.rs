use crate::healthcheck::start_http_server;
use crate::slack::start_websocket_client;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod healthcheck;
mod roulette;
mod slack;
mod slack_message;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(std::env::var("TERM").is_ok()))
        .with(tracing_subscriber::EnvFilter::from_env("PIZZAPICKER_LOG"))
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
