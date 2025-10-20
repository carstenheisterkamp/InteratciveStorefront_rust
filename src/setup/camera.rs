use bevy::render::view::Hdr;
use bevy::prelude::*;

pub fn spawn_static_default_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Transform::from_xyz(-2.5, 2.0, 15.0).looking_at(Vec3::ZERO, Dir3::Y),
        SpatialListener::new(50.0),
    ));
}

pub fn spawn_dynamic_default_camera(mut commands: Commands, position: Vec3, target: Vec3) -> Entity {
    commands
        .spawn((
        Camera3d::default(),
        Hdr,
            Transform::from_translation(position).looking_at(target, Vec3::Y),
    ))
        .id()
}