use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tavla_core::Player;
use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};

use super::actions::handle_client_message;
use super::ai_offer::{
    resend_pending_ai_offer, take_back_seat_from_ai, withdraw_ai_offer_after_return,
};
use super::broadcast::{broadcast_state, notify_presence, notify_round_countdown};
use super::{PING_INTERVAL, PONG_TIMEOUT};
use crate::{
    service::game::{sessions::ManagedSession, ws_protocol::ServerMessage},
    state::app_state::AppState,
};

#[derive(Debug)]
enum SocketExit {
    ClientClosed,
    ReadError,
    SendFailed,
    PongTimeout,
}

pub(super) async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    session: Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    session.set_sender(player, tx).await;
    take_back_seat_from_ai(&state, &session, game_id, player).await;
    withdraw_ai_offer_after_return(&state, &session, game_id, player).await;

    tracing::info!(game_id, ?player, "ws_connected");
    notify_presence(&session, player, ServerMessage::OpponentConnected).await;
    broadcast_state(&session).await;
    notify_round_countdown(&session, player).await;
    resend_pending_ai_offer(&session, player).await;

    let mut ping_ticker = interval(PING_INTERVAL);
    ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_pong = tokio::time::Instant::now();

    let exit_reason = loop {
        tokio::select! {
            outgoing = rx.recv() => {
                let Some(text) = outgoing else { break SocketExit::ClientClosed };
                tracing::info!(game_id, ?player, message = %text, "ws_send");
                if ws_sender.send(Message::Text(text.into())).await.is_err() {
                    break SocketExit::SendFailed;
                }
            }
            incoming = ws_receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        tracing::info!(game_id, ?player, message = %text, "ws_recv");
                        handle_client_message(&state, &session, game_id, player, &text).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = tokio::time::Instant::now();
                    }
                    Some(Ok(Message::Close(_))) => break SocketExit::ClientClosed,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break SocketExit::ReadError,
                    None => break SocketExit::ClientClosed,
                }
            }
            _ = ping_ticker.tick() => {
                if last_pong.elapsed() > PONG_TIMEOUT {
                    break SocketExit::PongTimeout;
                }
                if ws_sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break SocketExit::SendFailed;
                }
            }
        }
    };

    tracing::warn!(game_id, ?player, ?exit_reason, "ws_disconnected");

    session.clear_sender(player).await;
    notify_presence(&session, player, ServerMessage::OpponentDisconnected).await;
}
