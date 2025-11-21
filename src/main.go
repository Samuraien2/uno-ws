package main

import (
	"fmt"
	"net/http"
	"strings"
	"sync"
	"sync/atomic"

	"github.com/gorilla/websocket"
)

type Room struct {
	name  string
	users map[uint64]*websocket.Conn
}

type ServerState struct {
	rooms map[string]*Room
	mu    sync.RWMutex
}

var state = ServerState{
	rooms: make(map[string]*Room),
}

var counter uint64

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func parseMetadata(id uint64, data []byte) {
	str := string(data)
	lines := strings.Split(str, "\n")
	if len(lines) != 10 {
		fmt.Println("Failed reading metadata: Too many lines")
		return
	}
	/*
		fmt.Printf("[%d] Metadata\n", id)
		fmt.Printf("- User Agent: %s\n", lines[0])
		fmt.Printf("- CPU Cores: %s\n", lines[1])
		fmt.Printf("- Memory: %sgb\n", lines[2])
		fmt.Printf("- WebGL Vendor: %s\n", lines[3])
		fmt.Printf("- WebGL Renderer: %s\n", lines[4])
		fmt.Printf("- Languages: %s\n", lines[5])
		fmt.Printf("- Connection: %s\n", lines[6])
		if lines[8] == "y" {
			fmt.Printf("- Battery: %s%% (charging)\n", lines[7])
		} else {
			fmt.Printf("- Battery: %s%%\n", lines[7])
		}
		fmt.Printf("- Timezone: %s\n", lines[9])
	*/
}

// IDs for received packets
type PacketID byte

const (
	CREATE_ROOM PacketID = iota
	JOIN_ROOM
	LEAVE_ROOM
	KICK_USER
)

func createRoom(id uint64, name string, conn *websocket.Conn) bool {
	state.mu.Lock()
	defer state.mu.Unlock()

	if _, exists := state.rooms[name]; exists {
		fmt.Printf("[%d] Room already exists: %s\n", id, name)
		return false
	}

	r := &Room{
		name:  name,
		users: map[uint64]*websocket.Conn{id: conn},
	}

	state.rooms[name] = r
	return true
}

func joinRoom(id uint64, roomName string, conn *websocket.Conn) bool {
	state.mu.Lock()
	defer state.mu.Unlock()

	room, ok := state.rooms[roomName]
	if !ok {
		fmt.Printf("[%d] Tried joining unknown room: %s\n", id, roomName)
		return false
	}

	room.users[id] = conn
	return true
}

func leaveRoom(id uint64, roomName string) {
	state.mu.Lock()
	defer state.mu.Unlock()

	r, ok := state.rooms[roomName]
	if !ok {
		return
	}

	delete(r.users, id)

	if len(r.users) == 0 {
		delete(state.rooms, roomName)
		fmt.Printf("Deleted empty room: %s\n", roomName)
	}
}

func onPacket(id uint64, data []byte, packetID PacketID, room *string, conn *websocket.Conn) bool {
	switch packetID {
	case CREATE_ROOM:
		if *room != "" {
			fmt.Printf("[%d] 0_o Already in room\n", id)
			return false
		}

		roomName := string(data)
		if roomName == "" {
			fmt.Printf("[%d] 0_o Empty room name\n", id)
			return false
		}

		if createRoom(id, roomName, conn) {
			*room = roomName
			fmt.Printf("[%d] Created room: %s\n", id, roomName)
		}
	case JOIN_ROOM:
		if *room != "" {
			fmt.Printf("[%d] 0_o Already in room\n", id)
			return false
		}

		roomName := string(data)
		if roomName == "" {
			fmt.Printf("[%d] 0_o Empty room name\n", id)
			return false
		}

		if joinRoom(id, roomName, conn) {
			*room = roomName
			fmt.Printf("[%d] Joined room: %s\n", id, roomName)
		}
	}
	return true
}

func getConcatRooms() string {
	var lines []string
	for _, room := range state.rooms {
		lines = append(lines, room.name)
	}
	return strings.Join(lines, "\n")
}

func onConnect(id uint64, conn *websocket.Conn) {
	var room string

	_, msg, err := conn.ReadMessage()
	if err != nil {
		fmt.Println("Read error:", err)
		return
	}

	parseMetadata(id, msg)

	for {
		mt, msg, err := conn.ReadMessage()
		if err != nil {
			fmt.Println("Read error:", err)
			return
		}

		// if close packet or invalid packet
		if mt != websocket.BinaryMessage {
			return
		}

		if len(msg) == 0 {
			fmt.Printf("[%d] Empty msg\n", id)
			return
		}
		onPacket(id, msg[1:], PacketID(msg[0]), &room, conn)

		// err = conn.WriteMessage(mt, msg)
	}
}

func wsHandler(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}

	id := atomic.AddUint64(&counter, 1) - 1

	fmt.Printf("[%d] Connected\n", id)

	onConnect(id, conn)
	conn.Close()

	fmt.Printf("[%d] Connection closed\n", id)
}

func main() {
	addr := "127.0.0.1:9001"
	http.HandleFunc("/", wsHandler)
	fmt.Println("Listening on ws://" + addr)
	http.ListenAndServe(addr, nil)
}
