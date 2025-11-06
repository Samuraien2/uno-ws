use std::{sync::Arc};
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async};
use tokio_tungstenite::tungstenite::{Message};
use futures_util::{SinkExt, StreamExt};

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

#[allow(dead_code)]
struct Room {
    name: String,
    users: Vec<u32>,
}

#[tokio::main]
async fn main() {
    let addr = "localhost:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    let mut total_users_ever = 0;
    println!("Listening on ws://{addr}");

    let rooms: Arc<Mutex<Vec<Room>>> = Arc::new(Mutex::new(Vec::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        total_users_ever += 1;
        let id = total_users_ever;
        let rooms = Arc::clone(&rooms);

        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();

            let mut room = 0;

            println!("[{}] New connection [{}]", id, addr);

            let (mut write, mut read) = ws_stream.split();

            if !print_meta_data(id, &mut read).await {
                println!("Failed reading metadata, closing!");
                return;
            }

            write.send(Message::text(id.to_string())).await.unwrap();

            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    let bytes = msg.into_data();
                    if bytes.is_empty() {
                        break;
                    }

                    let packet_id = bytes[0];
                    let packet_len = bytes.len();
                    if packet_id == 1 {
                        if room > 0 {
                            println!("[{}] O_o Tried creating room while in room", id);
                            continue;
                        }

                        if packet_len == 1 {
                            println!("[{}] O_o No name supplied", id);
                            continue;
                        }
                        
                        let slice = &bytes[1..];
                        let s = String::from_utf8_lossy(slice).into_owned();

                        println!("[{}] Created room: {}", id, s);
                        let mut rooms_lock = rooms.lock().await;
                        let len = rooms_lock.len() + 1;
                        rooms_lock.push(Room {
                            name: s,
                            users: vec![id]
                        });

                        room = len;
                    }    
                }
            }
            println!("[{}] Connection closed", id);
        });
    }
}
