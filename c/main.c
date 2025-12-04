#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdatomic.h>
#include "ws/ws.h"

#define MAX_CONNECTIONS 20
#define MAX_ROOMS 20
#define MAX_USERS_IN_ROOM 8

typedef enum {
    PACKET_CREATE_ROOM,
    PACKET_JOIN_ROOM,

    PACKET_SEND_ROOMS = 50,
} Packet;

typedef struct {
    const char *name;
    int room;
} User;

typedef struct {
    const char *name;
    int userIDs[MAX_USERS_IN_ROOM];
} Room;

// ONLY ACCESSED VIA LOCK!
User *users[MAX_CONNECTIONS] = {};
Room *rooms[MAX_ROOMS] = {};
pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;

// returns id
void rooms_add(Room *room, int owner_user_id) {
    room->userIDs[0] = owner_user_id;
    pthread_mutex_lock(&lock);
    int free_room = 0;
    for (int i = 0; i < MAX_ROOMS; i++) {
        if (rooms[i] == NULL) {
            free_room = i;
            break;
        }
    }
    rooms[free_room] = room;
    users[owner_user_id]->room = free_room;

    pthread_mutex_unlock(&lock);
}

void on_event(WsEvent *event) {
    WsEvent e = *event;
    if (e.type == WS_EVENT_BINARY_MESSAGE) {
        uint8_t *bytes = e.msg.bytes;
        uint64_t len = e.msg.len;

        printf("[");
        for (int i = 0; i < len; i++) {
            printf("%d,", bytes[i]);
        }
        printf("]\n");

        if (bytes[0] == PACKET_CREATE_ROOM) {
            int name_len = len - 1;

            char *name = malloc(len);
            if (!name) exit(67);
            for (int i = 0; i < name_len; i++) {
                name[i] = bytes[i+1];
            }
            name[name_len] = '\0';

            printf("Creating room: %s\n", name);

            Room *room = malloc(sizeof(Room));
            if (!room) exit(67);
            room->name = name;
            rooms_add(room, e.msg.user_id);
        }
    }
    else if (e.type == WS_EVENT_CONNECT) {
        printf("%d connected\n", e.conn.user_id);
    }
    else if (e.type == WS_EVENT_DISCONNECT) {
        printf("%d disconnected\n", e.conn.user_id);
    }
}

int main() {
    WsState *ws = ws_listen(9001);
    if (!ws) {
        printf("Failed lol\n");
        return 1;
    }
    printf("Server listening on port 9001...\n");

    ws_loop(ws, on_event);

    ws_close(ws);
    return 0;
}