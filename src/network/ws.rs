use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures_util::StreamExt;
use serde::Deserialize;
use std::thread;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use super::events::GameEvent;

#[derive(Deserialize, Debug, Clone)]
struct WsHandData {
    id: i32,
    pinch_distance: f32,
    gesture: String,
}

#[derive(Deserialize, Debug, Clone)]
struct WsObjectData {
    // z.B. name: String, x: f32, ...
}

#[derive(Deserialize, Debug, Clone)]
struct WsFrameData {
    hand_count: Option<i32>,
    hands: Vec<WsHandData>,
    objects: Vec<WsObjectData>,
}

pub struct WebSocketReceiverPlugin {
    pub listen_address: String,
}

impl Plugin for WebSocketReceiverPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = unbounded::<WsFrameData>();

        app.insert_resource(WsMessageReceiver(rx))
            .add_message::<GameEvent>()
            .add_systems(Update, process_ws_messages_system);

        info!("Starte WebSocket-Listener-Thread auf {}", self.listen_address);
        start_ws_listener_thread(self.listen_address.clone(), tx);
    }
}

#[derive(Resource)]
struct WsMessageReceiver(Receiver<WsFrameData>);

fn start_ws_listener_thread(listen_address: String, sender: Sender<WsFrameData>) {
    thread::spawn(move || {
        let rt = Runtime::new().expect("Konnte Tokio-Runtime nicht erstellen");
        rt.block_on(run_ws_server(listen_address, sender));
    });
}

async fn run_ws_server(listen_address: String, sender: Sender<WsFrameData>) {
    let addr = listen_address;
    let listener = TcpListener::bind(&addr).await.expect("Bind an WS-Port fehlgeschlagen");
    info!("WebSocket-Server lauscht auf {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let sender_clone = sender.clone();
        tokio::spawn(handle_connection(stream, sender_clone));
    }
}

async fn handle_connection(stream: TcpStream, sender: Sender<WsFrameData>) {
    let addr = stream.peer_addr().expect("Client hat keine Adresse");
    info!("Neuer WebSocket-Client verbunden: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket-Handshake-Fehler: {}", e);
            return;
        }
    };

    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsFrameData>(&text) {
                    Ok(frame_data) => {
                        if sender.send(frame_data).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Konnte WS-JSON nicht parsen: {}. Payload: {}", e, text);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket-Client getrennt: {}", addr);
                break;
            }
            Err(e) => {
                error!("WebSocket-Fehler [{}]: {}", addr, e);
                break;
            }
            _ => { /* Ignoriere Binary, Ping, Pong etc. */ }
        }
    }
    info!("Verbindung zu {} geschlossen.", addr);
}

fn process_ws_messages_system(
    receiver: Res<WsMessageReceiver>,
    mut event_writer: MessageWriter<GameEvent>,
) {
    while let Ok(frame) = receiver.0.try_recv() {
        if let Some(count) = frame.hand_count {
            if count >= 0 {
                event_writer.write(GameEvent::HandCountChanged(count));
                trace!("Hand count changed: {}", count);
            } else {
                warn!("Ungültige hand_count empfangen: {}", count);
            }
        }

        for hand in frame.hands {
            if hand.id < 0 {
                warn!("Ungültige hand_id: {}", hand.id);
                continue;
            }

            if hand.pinch_distance.is_finite() {
                event_writer.write(GameEvent::HandPinchDistance {
                    hand_id: hand.id,
                    distance: hand.pinch_distance,
                });
            } else {
                warn!("Ungültige pinch_distance für Hand {}: {}", hand.id, hand.pinch_distance);
            }

            if !hand.gesture.is_empty() {
                event_writer.write(GameEvent::HandGesture {
                    hand_id: hand.id,
                    gesture: hand.gesture,
                });
            } else {
                warn!("Leeres gesture für Hand {}", hand.id);
            }
        }
    }
}