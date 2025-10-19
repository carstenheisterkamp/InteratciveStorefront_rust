use bevy::prelude::*;

pub fn spawn_default_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 2.0, 15.0).looking_at(Vec3::ZERO, Dir3::Y),
        SpatialListener::new(50.0),
    ));
}
