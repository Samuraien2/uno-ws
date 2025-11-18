use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

type RoomID = u16;
type UserID = u32;

struct Room {
    name: String,
    users: Vec<UserID>,
}

async fn read_metadata<R>(id: u32, read: &mut R) -> bool
where
    R: futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    if let Some(Ok(metadata)) = read.next().await {
        if !metadata.is_text() {
            return false;
        }
        if let Message::Text(text) = metadata {
            let lines: Vec<&str> = text.lines().collect();

            if lines.len() != 10 {
                return false;
            }

            println!("[{id}] Metadata");
            if false {
                println!("- User agent: {}", lines[0]);
                println!("- CPU Cores: {}", lines[1]);
                println!("- Memory: {}gb", lines[2]);
                println!("- WebGL Vendor: {}", lines[3]);
                println!("- WebGL Renderer: {}", lines[4]);
                println!("- Languages: {}", lines[5]);
                println!("- Connection: {}", lines[6]);
                if lines[8] == "y" {
                    println!("- Battery: {}% (charging)", lines[7]);
                } else {
                    println!("- Battery: {}% (not charging)", lines[7]);
                }
                println!("- Timezone: {}", lines[9]);
            }

            return true;
        }
    }

    false
}

async fn concat_room_names(rooms: &Arc<Mutex<Vec<Room>>>) -> String {
    let rooms_lock = rooms.lock().await;

    let mut result = String::new();
    for room in rooms_lock.iter() {
        result.push_str(&room.name);
        result.push('\n');
    }

    result.trim_end().to_string()
}

async fn packet_receive(
    id: UserID,
    room: &mut usize,
    bytes: Bytes,
    rooms: &Arc<Mutex<Vec<Room>>>,
) -> bool {
    let packet_id = bytes[0];
    let packet_len = bytes.len();
    if packet_id == 1 {
        if *room > 0 {
            println!("[{id}] O_o Tried creating room while in room");
            return false;
        }

        if packet_len == 1 {
            println!("[{id}] O_o Tried creating empty room");
            return false;
        }

        let slice = &bytes[1..];
        let name = String::from_utf8_lossy(slice).into_owned();

        println!("[{id}] Created room: {name}");
        let mut rooms_lock = rooms.lock().await;
        let len = rooms_lock.len() + 1;
        rooms_lock.push(Room {
            name,
            users: vec![id],
        });

        *room = len;
        return true;
    }
    if packet_id == 2 {
        if *room > 0 {
            println!("[{id}] O_o Tried joining room while in room");
            return false;
        }

        if packet_len == 1 {
            println!("[{id}] O_o Tried joining empty room");
            return false;
        }

        let slice = &bytes[1..];
        let name = String::from_utf8_lossy(slice).into_owned();

        let mut rooms_lock = rooms.lock().await;
        for room in rooms_lock.iter_mut() {
            if room.name == name {
                println!("[{id}] Joined room: {name}");
                room.users.push(id);
                return true;
            }
        }
        println!("[{id}] Tried joining invalid room: {name}");
        return false;
    }
    return false;
}

#[tokio::main]
async fn main() {
    let addr = "localhost:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    let mut total_users_ever: UserID = 0;
    println!("Listening on ws://{addr}");

    let rooms: Arc<Mutex<Vec<Room>>> = Arc::new(Mutex::new(Vec::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        total_users_ever += 1;
        let id: UserID = total_users_ever;
        let rooms = Arc::clone(&rooms);

        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();

            let mut room_nr: usize = 0;

            println!("[{id}] New connection [{addr}]");

            let (mut write, mut read) = ws_stream.split();

            if !read_metadata(id, &mut read).await {
                println!("[{id}] Failed reading metadata, closing!");
                return;
            }

            let room_names = concat_room_names(&rooms).await;
            write.send(Message::text(room_names)).await.unwrap();

            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    let bytes = msg.into_data();
                    if bytes.is_empty() {
                        println!("[{id}] O_o Empty packet");
                        break;
                    }

                    if !packet_receive(id, &mut room_nr, bytes, &rooms).await {
                        write.send(Message::text("oopsies")).await.unwrap();
                    }
                }
            }

            if room_nr > 0 {
                println!("[{id}] Leaving while in room");
                let mut rooms_lock = rooms.lock().await;
                let room_idx = room_nr - 1;
                let room = rooms_lock.get_mut(room_idx).unwrap();
                if room.users[0] == id {
                    println!("[{id}] Is owner :O");
                    rooms_lock.remove(room_idx);
                } else {
                    println!("[{id}] Is member :)");
                    if let Some(pos) = room.users.iter().position(|&x| x == id) {
                        room.users.remove(pos);
                    }
                }
            }
            println!("[{id}] Connection closed");
        });
    }
}
