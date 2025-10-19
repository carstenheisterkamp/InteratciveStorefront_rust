use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::assetloader::LoadedModels;
use crate::setup::gltf_spawner::{GltfSpawnConfig, spawn_gltf_with_physics, spawn_primitive_with_physics};

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn((
    //     RigidBody::Static,
    //     Collider::cylinder(40.0, 0.1),
    //     Mesh3d(meshes.add(Cylinder::new(40.0, 0.1))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 100))),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));
}

/// Marker-Component für Objekte mit radialer Gravitation
#[derive(Component)]
pub struct RadialGravity;

/// System das radiale Gravitation zum Zentrum anwendet
pub fn apply_radial_gravity(
    mut query: Query<(&Transform, &mut LinearVelocity), With<RadialGravity>>,
    time: Res<Time>,
) {
    let center = Vec3::ZERO;
    let strength = 9.81;

    for (transform, mut velocity) in query.iter_mut() {
        let position = transform.translation;
        let to_center = center - position;
        let distance = to_center.length();

        if distance > 0.01 {
            // Berechne Beschleunigung zum Zentrum
            let gravity_accel = to_center.normalize() * strength;
            // Addiere zur Geschwindigkeit
            velocity.0 += gravity_accel * time.delta_secs();
        }
    }
}

/// Spawnt alle initialen Objekte, nachdem Assets geladen sind
pub fn spawn_initial_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loaded_models: Res<LoadedModels>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
) {
    info!("🎲 Spawning initial objects!");

    // Spawne einen Würfel mit Physik über Helper
    spawn_primitive_with_physics(
        &mut commands,
        meshes.add(Cuboid::from_length(1.0)),
        materials.add(Color::srgb_u8(0, 0, 0)),
        Transform::from_xyz(0.0, 4.0, 0.0),
        Collider::cuboid(1.0, 1.0, 1.0),
        100.0,
        0.0,
        0.0,
        Vec3::ZERO,
        Vec3::new(0.1, 0.1, 0.1),
        1.0,
        Some(RadialGravity),
    );

    // BEST PRACTICE: Nutze generische spawn_gltf_with_physics Funktion
    if let Some(tasse_handle) = &loaded_models.tasse {
        let scale = 0.5;
        let config = GltfSpawnConfig::new(tasse_handle.clone())
            // Nutze denselben Collider wie Stresstest
            .with_collider_gltf(loaded_models.tasse_collider.clone().unwrap_or(tasse_handle.clone()))
            // Verwende einen SEHR kleinen Fallback-Collider zum Testen
            .with_fallback_collider(Collider::cylinder(0.02, 0.05))
            .with_transform(Transform::from_xyz(2.0, 2.0, 2.0))
            .with_scale(scale)
            .with_mass(1.0)
            .with_physics(0.1, 0.2)  // Gleiche Physik wie Stresstest
            .with_radial_gravity(true);

        if let Some(entity) = spawn_gltf_with_physics(
            &mut commands,
            &gltf_assets,
            &gltf_mesh_assets,
            &meshes,
            config,
            scale,
            Some(RadialGravity),
        ) {
            info!("☕ Tasse spawned with entity ID: {:?}", entity);
        }
    }

    info!("✅ All initial objects spawned!");
}
