use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

type RoomID = u32;
type UserID = u32;

struct UserMeta {
    user: UserID,
    room: RoomID,
}

enum Packet {
    CreateRoom,
    JoinRoom,
    Unknown(()),
}

type Users = Arc<Mutex<HashMap<UserID, UserMeta>>>;

// rust moment
const fn packet_from_id(id: u8) -> Packet {
    match id {
        0 => Packet::CreateRoom,
        1 => Packet::JoinRoom,
        _ => Packet::Unknown(()),
    }
}

struct Room {
    name: String,
    users: Vec<UserID>,
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
    if bytes.is_empty() {
        return false;
    }

    match packet_from_id(bytes[0]) {
        Packet::CreateRoom => {
            if *room > 0 {
                return false;
            }

            if bytes.len() == 1 {
                return false;
            }

            let slice = &bytes[1..];
            let name = String::from_utf8_lossy(slice).into_owned();

            println!("[{id}] Created room: {name}");
            let mut rooms_lock = rooms.lock().await;
            rooms_lock.push(Room {
                name,
                users: vec![id],
            });
            *room = rooms_lock.len();
            return true;
        }
        Packet::JoinRoom => {
            if *room > 0 {
                return false;
            }

            if bytes.len() == 1 {
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

            return false;
        }
        Packet::Unknown(_) => return false,
    }
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

            println!("[{id}] New connection from {addr}");

            let (mut write, mut read) = ws_stream.split();

            let room_names = concat_room_names(&rooms).await;
            write.send(Message::text(room_names)).await.unwrap();

            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    if !packet_receive(id, &mut room_nr, msg.into_data(), &rooms).await {
                        write.send(Message::text("Invalid packet")).await.unwrap();
                        println!("[{id}] O_o Invalid packet");
                    }
                }
            }

            if room_nr > 0 {
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
