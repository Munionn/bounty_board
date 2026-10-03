use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::{verify_token, Claims};
use crate::dto::{
    ClientWsEvent, ListMessagesQuery, ServerWsEvent, WsAuthQuery, WsConnectedPayload,
    WsHistoryPayload, WsMessageAckPayload, WsMessageCreatedPayload,
};
use crate::error::ServiceError;
use crate::services::{bounty as bounty_service, chat as chat_service};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
    Query(auth): Query<WsAuthQuery>,
) -> Response {
    let claims = match verify_token(&auth.token, &state.jwt_secret) {
        Ok(claims) => claims,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let bounty = match bounty_service::get_by_id(&state.db, bounty_id).await {
        Ok(bounty) => bounty,
        Err(err) => return StatusCode::from(err).into_response(),
    };

    if let Err(err) = chat_service::ensure_participant(&bounty, &claims.wallet) {
        return StatusCode::from(err).into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, bounty_id, claims))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, bounty_id: Uuid, claims: Claims) {
    let chat = match chat_service::ensure_chat_for_bounty(&state.db, bounty_id).await {
        Ok(chat) => chat,
        Err(err) => {
            send_error(&mut socket, "chat_init_failed", &err.to_string(), None).await;
            return;
        }
    };

    let mut room_rx = state.chat_hub.subscribe(chat.id).await;

    if !send_event(
        &mut socket,
        &ServerWsEvent::Connected(WsConnectedPayload {
            chat_id: chat.id,
            bounty_id,
        }),
    )
    .await
    {
        return;
    }

    let history = match chat_service::list_messages(
        &state.db,
        chat.id,
        bounty_id,
        &ListMessagesQuery {
            limit: 50,
            before: None,
        },
    )
    .await
    {
        Ok(messages) => messages,
        Err(err) => {
            send_error(&mut socket, "history_failed", &err.to_string(), None).await;
            return;
        }
    };

    if !history.is_empty()
        && !send_event(
            &mut socket,
            &ServerWsEvent::History(WsHistoryPayload { messages: history }),
        )
        .await
    {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if !process_text_message(
                            &mut socket,
                            &state,
                            chat.id,
                            bounty_id,
                            &claims.wallet,
                            &text,
                        ).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        tracing::warn!("websocket receive error: {err}");
                        break;
                    }
                }
            }
            broadcast = room_rx.recv() => {
                match broadcast {
                    Ok(json) => {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("chat client lagged, skipped {skipped} messages");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn process_text_message(
    socket: &mut WebSocket,
    state: &AppState,
    chat_id: Uuid,
    bounty_id: Uuid,
    sender_wallet: &str,
    text: &str,
) -> bool {
    let event: ClientWsEvent = match serde_json::from_str(text) {
        Ok(event) => event,
        Err(err) => {
            send_error(socket, "invalid_payload", &err.to_string(), None).await;
            return true;
        }
    };

    match event {
        ClientWsEvent::Ping => send_event(socket, &ServerWsEvent::Pong).await,
        ClientWsEvent::SendMessage { payload } => {
            let client_message_id = payload.client_message_id.clone();

            let bounty = match bounty_service::get_by_id(&state.db, bounty_id).await {
                Ok(bounty) => bounty,
                Err(err) => {
                    send_error(
                        socket,
                        service_error_code(&err),
                        &err.to_string(),
                        client_message_id,
                    )
                    .await;
                    return true;
                }
            };

            if let Err(err) = chat_service::ensure_participant(&bounty, sender_wallet) {
                send_error(
                    socket,
                    service_error_code(&err),
                    &err.to_string(),
                    client_message_id,
                )
                .await;
                return true;
            }

            match chat_service::send_message(&state.db, chat_id, bounty_id, sender_wallet, &payload)
                .await
            {
                Ok(message) => {
                    let ack = ServerWsEvent::MessageAck(WsMessageAckPayload {
                        client_message_id,
                        message: message.clone(),
                    });
                    let created =
                        ServerWsEvent::MessageCreated(WsMessageCreatedPayload { message });

                    if !send_event(socket, &ack).await {
                        return false;
                    }

                    if let Ok(json) = serde_json::to_string(&created) {
                        state.chat_hub.publish(chat_id, json).await;
                    }

                    true
                }
                Err(err) => {
                    send_error(
                        socket,
                        service_error_code(&err),
                        &err.to_string(),
                        client_message_id,
                    )
                    .await;
                    true
                }
            }
        }
    }
}

async fn send_event(socket: &mut WebSocket, event: &ServerWsEvent) -> bool {
    match serde_json::to_string(event) {
        Ok(json) => socket.send(Message::Text(json.into())).await.is_ok(),
        Err(err) => {
            tracing::error!("failed to serialize websocket event: {err}");
            false
        }
    }
}

async fn send_error(
    socket: &mut WebSocket,
    code: &str,
    message: &str,
    client_message_id: Option<String>,
) {
    let _ = send_event(
        socket,
        &ServerWsEvent::Error(crate::dto::WsErrorPayload {
            code: code.to_string(),
            message: message.to_string(),
            client_message_id,
        }),
    )
    .await;
}

fn service_error_code(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::BadRequest(_) => "bad_request",
        ServiceError::Forbidden => "forbidden",
        ServiceError::Unauthorized => "unauthorized",
        ServiceError::Conflict => "conflict",
        ServiceError::NotFound => "not_found",
        ServiceError::NotImplemented => "not_implemented",
        ServiceError::Internal | ServiceError::Database(_) => "internal_error",
    }
}
