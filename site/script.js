const createRoomInput = document.getElementById("create-room-name");
const joinRoomInput = document.getElementById("join-room-name");
const output = document.getElementById("output");

function log(msg) {
  output.innerText += msg + "\n";
}

const Packet = {
  CREATE_ROOM: 0,
  JOIN_ROOM: 1,
};

const ws = new WebSocket("ws://localhost:9001");

let receivedRoomNames = false;

ws.addEventListener("message", (ev) => {
  if (!receivedRoomNames) {
    receivedRoomNames = true;
    if (!ev.data) {
      log("No rooms");
      return;
    }
    const lines = ev.data.split("\n");
    log("Rooms:");
    for (let line of lines) {
      log("- " + line);
    }
    return;
  }

  log("Msg: " + ev.data);
});

ws.addEventListener("close", (ev) => {
  log("Close: " + ev.reason);
});

ws.addEventListener("error", (err) => {
  log("Err:" + err);
});

function createRoom() {
  const name = createRoomInput.value;
  if (!name) return;

  const encoder = new TextEncoder();
  const stringBytes = encoder.encode(name);
  const bytes = new Uint8Array(1 + stringBytes.length);
  bytes[0] = Packet.CREATE_ROOM;
  bytes.set(stringBytes, 1);
  ws.send(bytes);
}

function joinRoom() {
  const name = joinRoomInput.value;
  if (!name) return;

  const encoder = new TextEncoder();
  const stringBytes = encoder.encode(name);
  const bytes = new Uint8Array(1 + stringBytes.length);
  bytes[0] = Packet.JOIN_ROOM;
  bytes.set(stringBytes, 1);
  ws.send(bytes);
}
