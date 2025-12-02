#include <libwebsockets.h>
#include <stdio.h>

#define PING_INTERVAL 5 // seconds

typedef struct PerSessionData {
    time_t last_pong;
} PerSessionData;

static int callback_echo(
    struct lws *wsi,
    enum lws_callback_reasons reason,
    void *user,
    void *in,
    size_t len
) {
    PerSessionData *pss = (PerSessionData*)user;

    switch (reason) {
        case LWS_CALLBACK_LOCK_POLL:
            //printf("LWS_CALLBACK_LOCK_POLL\n");
            break;
        case LWS_CALLBACK_ADD_POLL_FD:
            //printf("LWS_CALLBACK_ADD_POLL_FD\n");
            break;
        case LWS_CALLBACK_UNLOCK_POLL:
            //printf("LWS_CALLBACK_UNLOCK_POLL\n");
            break;
        case LWS_CALLBACK_GET_THREAD_ID:
            //printf("LWS_CALLBACK_GET_THREAD_ID\n");
            break;
        case LWS_CALLBACK_PROTOCOL_INIT:
            //printf("LWS_CALLBACK_PROTOCOL_INIT\n");
            break;
        case LWS_CALLBACK_EVENT_WAIT_CANCELLED:
            //printf("LWS_CALLBACK_EVENT_WAIT_CANCELLED\n");
            break;
        case LWS_CALLBACK_CHANGE_MODE_POLL_FD:
            //printf("LWS_CALLBACK_CHANGE_MODE_POLL_FD\n");
            break;
        case LWS_CALLBACK_ESTABLISHED:
            printf("New client connected!\n");
            lws_callback_on_writable(wsi);
            break;

        case LWS_CALLBACK_RECEIVE:
            printf("RECEIVED DATA!\n");
            break;
        case LWS_CALLBACK_RECEIVE_PONG:
            printf("PONG\n");
            break;
        case LWS_CALLBACK_SERVER_WRITEABLE: {
            time_t now = time(NULL);

            // Send a ping every PING_INTERVAL seconds
            if (now - pss->last_pong >= PING_INTERVAL) {
                printf("Sending ping to client...\n");
                unsigned char buf[LWS_PRE + 0]; // empty payload, still need LWS_PRE
                unsigned char *p = &buf[LWS_PRE];
                lws_write(wsi, p, 0, LWS_WRITE_PING);
                pss->last_pong = now;
            }

            // schedule next check
            lws_callback_on_writable(wsi);
            break;
        }
        case LWS_CALLBACK_CLOSED:
        case LWS_CALLBACK_CLOSED_CLIENT_HTTP:
        case LWS_CALLBACK_CLIENT_CLOSED:
            printf("Client disconnected.\n");
            break;
        default:
            printf("BREH %p %lu %p %p %d\n", wsi, len, user, in, reason);
            break;
    }
    return 0;
}

static struct lws_protocols protocols[] = {
    {
        .name = "echo-protocol",
        .callback = callback_echo,
        .per_session_data_size = sizeof(PerSessionData),
    },
    { NULL, NULL, 0, 0 }
};

int main(void)
{
    struct lws_context_creation_info info = {
        .port = 9001,
        .protocols = protocols,
        .options = LWS_SERVER_OPTION_DISABLE_IPV6
    };

    struct lws_context *context = lws_create_context(&info);
    if (!context) {
        fprintf(stderr, "failed to create context\n");
        return 1;
    }

    while (1) {
        lws_service(context, 100);
    }

    lws_context_destroy(context);
    return 0;
}
