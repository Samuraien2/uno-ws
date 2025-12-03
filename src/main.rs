use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

// 0 = no room, 1.. = room id + 1
type RoomID = usize;
type UserID = u32;
type Write = Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>;
type Users = Arc<Mutex<HashMap<UserID, User>>>;
type Rooms = Arc<Mutex<Vec<Room>>>;

struct User {
    id: UserID,
    room: RoomID,
}

struct Connection {
    user: User,
    write: Write,
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

struct Room {
    name: String,
    users: Vec<Connection>,
}

// rust moment
const fn packet_from_id(id: u8) -> Packet {
    match id {
        0 => Packet::CreateRoom,
        1 => Packet::JoinRoom,
        _ => Packet::Invalid,
    }
}

async fn get_rooms_msg(rooms: &Rooms) -> Message {
    let names: Vec<String> = {
        let guard = rooms.lock().await;
        guard.iter().map(|r| r.name.clone()).collect()
    };

    let joined = names.join("\n");

    let mut buf = Vec::with_capacity(1 + joined.len());
    buf.push(SPacket::Rooms as u8);
    buf.extend_from_slice(joined.as_bytes());

    Message::Binary(buf.into())
}

async fn on_packet(uid: UserID, bytes: Bytes, users: &Users, rooms: &Rooms) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let mut users_lock = users.lock().await;
    let conn = users_lock.get_mut(&uid).unwrap();

    match packet_from_id(bytes[0]) {
        Packet::CreateRoom => {
            if conn.room > 0 || bytes.len() == 1 {
                return false;
            }

            let slice = &bytes[1..];
            let name = String::from_utf8_lossy(slice).into_owned();

            println!("[{uid}] Created room: {name}");

            let mut rooms_lock = rooms.lock().await;
            rooms_lock.push(Room {
                name,
                users: vec![uid],
            });
            conn.room = rooms_lock.len();
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
                    println!("[{uid}] Joined room: {name}");
                    room.users.push(conn);
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
            let (write, mut read) = ws_stream.split();
            let conn = &mut Connection {
                user: User { id, room: room_nr },
                write: write,
            };

            println!("[{id}] New connection from {addr}");

            conn.write.send(get_rooms_msg(&rooms).await).await.unwrap();

            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_binary() {
                    if !on_packet(conn, msg.into_data(), &rooms).await {
                        conn.write.send(Message::text("Error")).await.unwrap();
                        println!("[{id}] O_o Invalid packet");
                    }
                }
            }

            if room_nr > 0 {
                let mut rooms_lock = rooms.lock().await;
                let room_idx = room_nr - 1;
                let room = rooms_lock.get_mut(room_idx).unwrap();
                if room.users[0].user.id == id {
                    println!("[{id}] Is owner :O");
                    rooms_lock.remove(room_idx);
                } else {
                    println!("[{id}] Is member :)");
                    if let Some(pos) = room.users.iter().position(|&x| x.user.id == id) {
                        room.users.remove(pos);
                    }
                }
            }
            println!("[{id}] Connection closed");
        });
    }
}
