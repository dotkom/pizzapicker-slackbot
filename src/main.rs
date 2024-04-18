use crate::slack::start_websocket_client;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod roulette;
mod slack;
mod slack_message;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(std::env::var("TERM").is_ok()))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    start_websocket_client().await;
}
