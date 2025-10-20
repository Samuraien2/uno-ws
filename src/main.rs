use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("WebSocket server listening on ws://{addr}");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            println!("New WebSocket connection");

            let (mut write, mut read) = ws_stream.split();

            // Echo messages back
            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_text() || msg.is_binary() {
                    write.send(msg).await.unwrap();
                }
            }

            println!("Connection closed");
        });
    }
}
