use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::thread;

use super::events::GameEvent;
pub struct OscReceiverPlugin {
    pub listen_address: String,
}

impl Plugin for OscReceiverPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = unbounded::<OscMessage>();

        app.insert_resource(OscMessageReceiver(rx))
            // .add_event::<GameEvent>()
            .add_systems(Update, process_osc_messages_system);

        start_osc_listener_thread(self.listen_address.clone(), tx);
    }
}

#[derive(Resource)]
struct OscMessageReceiver(Receiver<OscMessage>);

fn start_osc_listener_thread(listen_address: String, sender: Sender<OscMessage>) {
    thread::spawn(move || {
        let addr = SocketAddrV4::from_str(&listen_address)
            .expect("Failed to parse OSC listen address");
        let socket = UdpSocket::bind(addr).expect("Failed to bind OSC socket");

        info!("OSC receiver listening on {}", listen_address);

        let mut buf = [0u8; 65536];

        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    let packet = rosc::decoder::decode_udp(&buf[..size]);
                    match packet {
                        Ok((_, OscPacket::Message(msg))) => {
                            if sender.send(msg).is_err() {
                                error!("Failed to send OSC message to channel");
                                break;
                            }
                        }
                        Ok((_, OscPacket::Bundle(bundle))) => {
                            for packet in bundle.content {
                                if let OscPacket::Message(msg) = packet {
                                    if sender.send(msg).is_err() {
                                        error!("Failed to send OSC message to channel");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to decode OSC packet: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive OSC packet: {}", e);
                }
            }
        }
    });
}

fn process_osc_messages_system(
    receiver: Res<OscMessageReceiver>,
    mut event_writer: MessageWriter<GameEvent>,
) {
    while let Ok(msg) = receiver.0.try_recv() {
        match msg.addr.as_str() {
            "/hand/count" => {
                if let Some(OscType::Int(count)) = msg.args.first() {
                    event_writer.write(GameEvent::HandCountChanged(*count));
                }
            }
            "/hand/gesture" => {
                if let (Some(OscType::Int(hand_id)), Some(gesture)) =
                    (msg.args.get(0), msg.args.get(1))
                {
                    if let OscType::String(gesture_str) = gesture {
                        event_writer.write(GameEvent::HandGesture {
                            hand_id: *hand_id,
                            gesture: gesture_str.clone(),
                        });
                    }
                }
            }
            "/hand/pinch" => {
                if let (Some(OscType::Int(hand_id)), Some(distance)) =
                    (msg.args.get(0), msg.args.get(1))
                {
                    let dist_f32 = match distance {
                        OscType::Float(f) => *f,
                        OscType::Double(d) => *d as f32,
                        _ => continue,
                    };
                    event_writer.write(GameEvent::HandPinchDistance {
                        hand_id: *hand_id,
                        distance: dist_f32,
                    });
                }
            }
            "/object/detected" => {
                if let (Some(name), Some(x), Some(y)) =
                    (msg.args.get(0), msg.args.get(1), msg.args.get(2))
                {
                    if let OscType::String(name_str) = name {
                        let x_f32 = match x {
                            OscType::Float(f) => *f,
                            OscType::Double(d) => *d as f32,
                            _ => continue,
                        };
                        let y_f32 = match y {
                            OscType::Float(f) => *f,
                            OscType::Double(d) => *d as f32,
                            _ => continue,
                        };
                        event_writer.write(GameEvent::ObjectDetected {
                            name: name_str.clone(),
                            x: x_f32,
                            y: y_f32,
                        });
                    }
                }
            }
            _ => {
                warn!("Unknown OSC address: {}", msg.addr);
            }
        }
    }
}