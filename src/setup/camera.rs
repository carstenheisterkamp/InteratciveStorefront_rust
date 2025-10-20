use bevy::render::view::Hdr;
use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub radius: f32,
    pub angle_x: f32,
    pub angle_y: f32,
}

pub fn spawn_static_default_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Transform::from_xyz(-2.5, 2.0, 15.0).looking_at(Vec3::ZERO, Dir3::Y),
        SpatialListener::new(50.0),
    ));
}

pub fn spawn_static_orbit_camera(mut commands: Commands) {
    let position = Vec3::new(-2.5, 2.0, 15.0);
    let target = Vec3::ZERO;
    let radius = position.distance(target);
    
    let direction = (position - target).normalize();
    let angle_y = direction.y.asin();
    let angle_x = direction.z.atan2(direction.x);

    commands.spawn((
        Camera3d::default(),
        Hdr,
        Transform::from_translation(position).looking_at(target, Vec3::Y),
        SpatialListener::new(50.0),
        OrbitCamera {
            target,
            radius,
            angle_x,
            angle_y,
        },
    ));
}

pub fn spawn_dynamic_default_camera(commands: &mut Commands, position: Vec3, target: Vec3) -> Entity {
    commands
        .spawn((
        Camera3d::default(),
        Hdr,
            Transform::from_translation(position).looking_at(target, Vec3::Y),
            SpatialListener::new(50.0),
    ))
        .id()
}

pub fn spawn_dynamic_orbit_camera(commands: &mut Commands, position: Vec3, target: Vec3, radius: f32) -> Entity {
    let direction = (position - target).normalize();
    let angle_y = direction.y.asin();
    let angle_x = direction.z.atan2(direction.x);

    commands
        .spawn((
            Camera3d::default(),
            Hdr,
            Transform::from_translation(position).looking_at(target, Vec3::Y),
            SpatialListener::new(50.0),
            OrbitCamera {
                target,
                radius,
                angle_x,
                angle_y,
            },
        ))
        .id()
}

pub fn orbit_camera_controls(
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    for (mut orbit, mut transform) in query.iter_mut() {
        // Maus-Rotation (rechte Maustaste gedrückt halten)
        if mouse_button.pressed(MouseButton::Right) {
            for motion in mouse_motion.read() {
                orbit.angle_x -= motion.delta.x * 0.005;
                orbit.angle_y = (orbit.angle_y - motion.delta.y * 0.005)
                    .clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
            }
        } else {
            // Events verwerfen wenn nicht gedrückt
            mouse_motion.clear();
        }

        // Zoom (Mausrad)
        for wheel in mouse_wheel.read() {
            orbit.radius = (orbit.radius - wheel.y * 0.5).max(1.0).min(50.0);
        }

        // Berechne neue Kamera-Position basierend auf Winkeln
        let x = orbit.radius * orbit.angle_y.cos() * orbit.angle_x.cos();
        let y = orbit.radius * orbit.angle_y.sin();
        let z = orbit.radius * orbit.angle_y.cos() * orbit.angle_x.sin();

        let new_position = orbit.target + Vec3::new(x, y, z);
        transform.translation = new_position;
        transform.look_at(orbit.target, Vec3::Y);
    }
}
