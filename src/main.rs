use tokio::join;
use crate::slack::create_subscriber;

mod slack;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .init();
    let (_rx, future) = create_subscriber().await;

    let _ = join!(future);
}
