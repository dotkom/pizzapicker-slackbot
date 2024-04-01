use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[tracing::instrument]
pub async fn start_http_server() -> tokio::io::Result<()> {
    tracing::info!("Starting HTTP server");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match socket.peer_addr() {
                Ok(peer_addr) => tracing::info!("Handling connection from {}", peer_addr),
                Err(err) => tracing::info!("Handling connection from unknown peer {}", err),
            }
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHealthcheck OK";
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });
    }
}
