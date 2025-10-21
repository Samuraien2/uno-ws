use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};

async fn print_meta_data<R>(id: u32, read: &mut R) -> bool
where
    R: futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    if let Some(Ok(metadata)) = read.next().await {
        if !metadata.is_text() {
            return false;
        }
        let msg = metadata;
        if let Message::Text(text) = msg {
            let lines: Vec<&str> = text.lines().collect();

            if lines.len() != 10 {
                return false;
            }

            println!("[{}] Metadata:", id);
            println!("- User agent: {}", lines[0]);
            println!("- CPU Cores: {}", lines[1]);
            println!("- Memory: {}gb", lines[2]);
            println!("- WebGL Vendor: {}", lines[3]);
            println!("- WebGL Renderer: {}", lines[4]);
            println!("- Languages: {}", lines[5]);
            println!("- Connection: {}", lines[6]);
            if lines[8] == "0" {
                println!("- Battery: {}% (not charging)", lines[7]);
            } else {
                println!("- Battery: {}% (charging)", lines[7]);
            }
            println!("- Timezone: {}", lines[9]);
            
            return true;
        }
    }

    false
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    let mut total_users_ever: u32 = 0;
    println!("WebSocket server listening on ws://{addr}");

    while let Ok((stream, addr)) = listener.accept().await {
        total_users_ever += 1;
        
        tokio::spawn(async move {
            println!("New connection [{} ({})]", addr, total_users_ever);

            let ws_stream = accept_async(stream).await.unwrap();
            let id = total_users_ever;

            let (mut write, mut read) = ws_stream.split();

            if !print_meta_data(id, &mut read).await {
                println!("Failed reading metadata, closing!");
                return;
            }
            
            
            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    let bytes: Vec<u8> = msg.into_data();
                    if bytes.is_empty() {
                        return;
                    }

                    let packet_id = bytes[0];
                    if packet_id == 1 {
                        println!("[{}] CREATE ROOM!", id);
                    }
                    write.send(Message::Text("Creating room".to_string())).await.unwrap();
                }
            }

            println!("Connection closed");
        });
    }
}
