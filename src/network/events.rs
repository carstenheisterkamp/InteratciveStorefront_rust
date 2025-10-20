
use bevy::prelude::*;

#[derive(Event, Debug, Clone, Message)]
pub enum GameEvent {
    HandCountChanged(i32),
    HandGesture { hand_id: i32, gesture: String },
    HandPinchDistance { hand_id: i32, distance: f32 },
    ObjectDetected { name: String, x: f32, y: f32 },
}