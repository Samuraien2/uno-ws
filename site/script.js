const createRoomInput = document.getElementById("create-room-name");
const joinRoomInput = document.getElementById("join-room-name");
const output = document.getElementById("output");

function log(msg) {
  output.innerText += msg + "\n";
}

const Packet = {
  CREATE_ROOM: 1,
  JOIN_ROOM: 2,
};

const ws = new WebSocket("ws://localhost:9001");

function sendTelemetry(battery, isCharging) {
  let webgl = getWebGLRendererInfo();
  let connection = "?";
  if (navigator.connection) {
    connection = navigator.connection.effectiveType;
  }

  ws.send(`${navigator.userAgent}
${navigator.hardwareConcurrency}
${navigator.deviceMemory || 0}
${webgl.vendor}
${webgl.renderer}
${navigator.languages}
${connection}
${battery}
${isCharging}
${Intl.DateTimeFormat().resolvedOptions().timeZone}`);
}

ws.addEventListener("open", (ev) => {
  if ("getBattery" in navigator) {
    navigator.getBattery().then((battery) => {
      sendTelemetry(Math.round(battery.level * 100), battery.charging + 0);
    });
  } else {
    sendTelemetry("?", "0");
  }
});

const received_room_names = false;

ws.addEventListener("message", (ev) => {
  if (!received_room_names) {
    if (!ev.data) {
      log("No rooms");
      return;
    }
    const lines = ev.data.split("\n");
    log("Rooms:");
    for (let line of lines) {
      log("- " + line);
    }
    received_room_names = true;
    return;
  }

  log("Received: " + ev.data);
});

ws.addEventListener("close", (ev) => {
  log("Closed: " + ev.reason);
});

ws.addEventListener("error", (err) => {
  log("WebSocket error:" + err);
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
  if (!name) {
    return;
  }
  const encoder = new TextEncoder();
  const stringBytes = encoder.encode(name);
  const bytes = new Uint8Array(1 + stringBytes.length);
  bytes[0] = Packet.JOIN_ROOM;
  bytes.set(stringBytes, 1);
  ws.send(bytes);
}

function getWebGLRendererInfo() {
  const canvas = document.createElement("canvas");

  const gl =
    canvas.getContext("webgl") || canvas.getContext("experimental-webgl");

  if (!gl) return { vendor: "?", renderer: "?" };

  const debugInfo = gl.getExtension("WEBGL_debug_renderer_info");

  if (debugInfo) {
    const renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
    const vendor = gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL);
    return { vendor, renderer };
  } else {
    return {
      vendor: gl.getParameter(gl.VENDOR),
      renderer: gl.getParameter(gl.RENDERER),
    };
  }
}
