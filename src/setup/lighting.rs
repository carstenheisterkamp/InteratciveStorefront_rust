use bevy::prelude::*;
use avian3d::prelude::*;

pub fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..default()
        },
        Transform::from_xyz(-2.0, 8.0, 2.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4, 0.0)),
    ));
}

