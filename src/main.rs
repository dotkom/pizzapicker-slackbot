use crate::slack::bootstrap_application;

mod roulette;
mod slack;
mod slack_message;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .init();
    bootstrap_application().await;
}
