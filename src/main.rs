use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_square)
        .run();
}

fn setup(mut commands: Commands) {
    // 2D-Kamera
    commands.spawn(Camera2d);

    // Rotes Quadrat (Sprite)
    commands.spawn(Sprite {
        color: Color::srgb(0.9, 0.1, 0.1),
        custom_size: Some(Vec2::new(100.0, 100.0)),
        ..Default::default()
    });
}

fn rotate_square(mut query: Query<&mut Transform, With<Sprite>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(1.0 * time.delta_secs());
    }
}