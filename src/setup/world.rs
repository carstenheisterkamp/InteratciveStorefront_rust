use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::assetloader::LoadedModels;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(40.0, 0.1),
        Mesh3d(meshes.add(Cylinder::new(4.0, 0.1))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
}

/// Spawnt alle initialen Objekte, nachdem Assets geladen sind
pub fn spawn_initial_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loaded_models: Res<LoadedModels>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    info!("🎲 Spawning initial objects!");

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Restitution::new(1.0),
        Friction::new(0.75),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 255))),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));

    // GLTF Modelle spawnen
    if let Some(tasse_handle) = &loaded_models.tasse {
        if let Some(gltf) = gltf_assets.get(tasse_handle) {
            commands.spawn((
                SceneRoot(gltf.scenes[0].clone()),
                Transform::from_xyz(2.0, 2.0, 2.0),
                RigidBody::Dynamic,
                Collider::cuboid(0.2, 0.5, 0.2),
            ));
            info!("☕ Tasse spawned!");
        }
    }

    info!("✅ All initial objects spawned!");
}
