const createRoomInput = document.getElementById("create-room-name");
const joinRoomInput = document.getElementById("join-room-name");
const output = document.getElementById("output");

function log(msg) {
  output.innerText += msg + "\n";
}

const Packet = {
  CreateRoom: 0,
  JoinRoom: 1,
};

const SPacket = {
  Rooms: 0,
};

let rooms = [];

const ws = new WebSocket("ws://localhost:9001");

let receivedRoomNames = false;

ws.onmessage = async (ev) => {
  const arrayBuffer = await ev.data.arrayBuffer();
  const arr = new Uint8Array(arrayBuffer);
  const packet_id = arr[0];
  if (packet_id === SPacket.Rooms) {
    let rooms_data = arr.subarray(1);
    const decoder = new TextDecoder("utf-8");
    const text = decoder.decode(rooms_data);
    const rooms = text.split("\n");
    log("ROOMS:");
    log(rooms);
  }
};

ws.onclose = () => {
  log("Connection closed");
};

ws.onerror = () => {
  log("Connection error");
};

function createRoom() {
  const name = createRoomInput.value;
  if (!name) return;

  const encoder = new TextEncoder();
  const stringBytes = encoder.encode(name);
  const bytes = new Uint8Array(1 + stringBytes.length);
  bytes[0] = Packet.CreateRoom;
  bytes.set(stringBytes, 1);
  ws.send(bytes);
}

function joinRoom() {
  const name = joinRoomInput.value;
  if (!name) return;

  const encoder = new TextEncoder();
  const stringBytes = encoder.encode(name);
  const bytes = new Uint8Array(1 + stringBytes.length);
  bytes[0] = Packet.JoinRoom;
  bytes.set(stringBytes, 1);
  ws.send(bytes);
}
