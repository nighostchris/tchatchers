//! The websocket is used to communicate between users within rooms.
//!
//! Websockets are isolated to each others, with one existing for each room.

// Copyright ⓒ 2022 LABEYE Loïc
// This tool is distributed under the MIT License, check out [here](https://github.com/nag763/tchatchers/blob/main/LICENSE.MD).

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::extractor::JwtUserExtractor;
use crate::AppState;
use axum::{
    extract::{ws::Message, ws::WebSocket, Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tchatchers_core::{
    room::Room,
    ws_message::{WsMessage, WsMessageContent},
};
use tokio::sync::broadcast;

#[derive(Default, Debug)]
pub struct WsRooms(HashMap<String, broadcast::Sender<String>>);

impl Deref for WsRooms {
    type Target = HashMap<String, broadcast::Sender<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WsRooms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The HTTP entry point.
///
/// # Arguments
///
/// - ws : The 'Upgrade' header, mandatory.
/// - state : The data shared across threads, used to retrieve existing rooms.
/// - room : the room name.
/// - jwt : The authenticated user infos.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(room): Path<String>,
    JwtUserExtractor(_jwt): JwtUserExtractor,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, room))
}

/// The socket handler
///
/// # Arguments
///
/// - socket : The struct used to communicate between the client and the server.
/// - state : The data shared across threads.
/// - room : The room name.
/// - user : The connected user's infos.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, room: String) {
    let (mut sender, mut receiver) = socket.split();
    let tx = {
        let mut rooms = state.txs.lock().unwrap();
        match rooms.get(&room) {
            Some(v) => v.clone(),
            None => {
                let (tx, _rx) = broadcast::channel(1000);
                rooms.insert(room.clone(), tx.clone());
                tx
            }
        }
    };
    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // This task will receive messages from client and send them to broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(msg) = serde_json::from_str(text.as_str()) {
                match msg {
                    WsMessage::Close => break,
                    WsMessage::Ping => {
                        let _ = tx.send(serde_json::to_string(&WsMessage::Pong).unwrap());
                    }
                    WsMessage::Pong | WsMessage::ClientKeepAlive => continue,
                    WsMessage::Send(ws_message) => {
                        let _ = tx.send(
                            serde_json::to_string(&WsMessage::Receive(ws_message.clone())).unwrap(),
                        );

                        Room::publish_message_in_room(
                            &mut state.redis_pool.get().unwrap(),
                            &room,
                            ws_message,
                        );
                    }
                    WsMessage::RetrieveMessages(session_id) => {
                        let messages: Vec<WsMessageContent> = Room::find_messages_in_room(
                            &mut state.redis_pool.get().unwrap(),
                            &room,
                        );
                        let _ = tx.send(
                            serde_json::to_string(&WsMessage::MessagesRetrieved {
                                messages,
                                session_id,
                            })
                            .unwrap(),
                        );
                    }
                    _ => {}
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
