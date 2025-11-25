use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

// 0 = no room, 1.. = room id + 1
type RoomID = usize;
type UserID = u32;

struct User {
    id: UserID,
    room: RoomID,
}

enum Packet {
    CreateRoom,
    JoinRoom,
    Invalid,
}

#[repr(u8)]
enum SPacket {
    Rooms,
}

type Users = Arc<Mutex<HashMap<UserID, User>>>;
type Rooms = Arc<Mutex<Vec<Room>>>;

// rust moment
const fn packet_from_id(id: u8) -> Packet {
    match id {
        0 => Packet::CreateRoom,
        1 => Packet::JoinRoom,
        _ => Packet::Invalid,
    }
}

struct Room {
    name: String,
    users: Vec<UserID>,
}

async fn concat_room_names(rooms: &Rooms) -> String {
    let rooms_lock = rooms.lock().await;

    let mut result = String::new();
    for room in rooms_lock.iter() {
        result.push_str(&room.name);
        result.push('\n');
    }
    result.trim_end().to_string()
}

async fn send_rooms()

async fn packet_receive(user: &mut User, bytes: Bytes, rooms: &Rooms) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let id = user.id;

    match packet_from_id(bytes[0]) {
        Packet::CreateRoom => {
            if user.room > 0 || bytes.len() == 1 {
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
            user.room = rooms_lock.len();
            return true;
        }
        Packet::JoinRoom => {
            if user.room > 0 || bytes.len() == 1 {
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
        Packet::Invalid => return false,
    }
}

#[tokio::main]
async fn main() {
    let addr = "localhost:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    let mut total_users_ever: UserID = 0;
    println!("Listening on ws://{addr}");

    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let rooms: Rooms = Arc::new(Mutex::new(Vec::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        total_users_ever += 1;

        let rooms = Arc::clone(&rooms);
        let users = Arc::clone(&users);

        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            let room_nr: RoomID = 0;
            let id: UserID = total_users_ever;
            let user = &mut User { id, room: room_nr };

            println!("[{id}] New connection from {addr}");

            let (mut write, mut read) = ws_stream.split();

            let room_names = concat_room_names(&rooms).await;

            let mut buf = Vec::with_capacity(1 + room_names.len());
            buf.push(SPacket::Rooms as u8);
            buf.extend_from_slice(room_names.as_bytes());
            write.send(Message::Binary(buf.into())).await.unwrap();

            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    if !packet_receive(user, msg.into_data(), &rooms).await {
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
