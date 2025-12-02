const WebSocket = require("ws");

const url = "ws://localhost:9001";
const ws = new WebSocket(url);

ws.on("open", () => {
  console.log("Connected to server!");

  ws.send("Hello from Node.js client!");
});

ws.on("message", (data) => {
  console.log("Received from server:", data.toString());
});

ws.on("ping", () => {
  console.log("Received ping from server");
  // Node.js ws automatically replies with pong
});

ws.on("pong", () => {
  console.log("Received pong from server");
});

ws.on("close", () => {
  console.log("Disconnected from server");
});

ws.on("error", (err) => {
  console.error("WebSocket error:", err);
});
