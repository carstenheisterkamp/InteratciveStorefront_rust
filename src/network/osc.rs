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
    // TODO: 'start_osc_listener_thread' hier einfügen
}

fn process_osc_messages_system(
    receiver: Res<OscMessageReceiver>,
    mut event_writer: EventWriter<GameEvent>,
) {
    // TODO: 'process_osc_messages_system
}