use bevy::prelude::*;
use crate::network::GameEvent;
use crate::setup::diagnostics::GameEventStats;

fn process_game_events_system(
    mut event_reader: MessageReader<GameEvent>,
    mut game_event_stats: ResMut<GameEventStats>,
) {
    for event in event_reader.read() {
        match event {
            GameEvent::HandCountChanged(count) => {
                info!("Gamelogic: Handanzahl geändert: {}", count);
                game_event_stats.hand_count_changed += 1;
                game_event_stats.last_event = Some(format!("HandCount: {}", count));
            }
            GameEvent::HandGesture { hand_id, gesture } => {
                info!("Gamelogic: Hand {} Geste erkannt: {}", hand_id, gesture);
                game_event_stats.hand_gesture += 1;
                game_event_stats.last_event = Some(format!("Gesture: Hand {} - {}", hand_id, gesture));
                // apply_gesture_to_entity(*hand_id, gesture);
            }
            GameEvent::HandPinchDistance { hand_id, distance } => {
                // trace! wird oft für kontinuierliche, weniger kritische Updates verwendet
                trace!("Gamelogic: Hand {} Pinz-Abstand: {}", hand_id, distance);
                game_event_stats.hand_pinch_distance += 1;
                game_event_stats.last_event = Some(format!("Pinch: Hand {} - {:.2}", hand_id, distance));
                // scale_object_by_pinch_distance(*hand_id, *distance);
            }
            GameEvent::ObjectDetected { name, x, y } => {
                info!("Gamelogic: Objekt '{}' an Position ({}, {}) erkannt.", name, x, y);
                game_event_stats.object_detected += 1;
                game_event_stats.last_event = Some(format!("Object: {} at ({:.1}, {:.1})", name, x, y));
                // spawn_entity_at_position(name, *x, *y);
            }
        }
    }
}

/// Ein Bevy-Plugin, um alle Gamelogic-Systeme zu bündeln und zur App hinzuzufügen.
pub struct GamelogicPlugin;

impl Plugin for GamelogicPlugin {
    fn build(&self, app: &mut App) {
        // Füge das Event-Verarbeitungssystem zum Update-Schedule hinzu
        app.add_systems(Update, process_game_events_system);
    }
}