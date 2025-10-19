use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::assetloader::LoadedModels;

/// Spawnt statische und dynamische Beispiel-Entitäten (Ground + Box)
pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Static physics object with a collision shape
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(4.0, 0.1),
        Mesh3d(meshes.add(Cylinder::new(4.0, 0.1))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
}

/// Spawnt die geladenen GLTF-Modelle in die Welt (läuft nur einmal, wenn Assets bereit sind)
pub fn spawn_loaded_models(
    mut commands: Commands,
    loaded_models: Res<LoadedModels>,
    gltf_assets: Res<Assets<Gltf>>,
    mut spawned: Local<bool>,
) {
    // Nur einmal spawnen
    if *spawned {
        return;
    }

    let mut all_ready = true;

    // Spawn die Tasse
    if let Some(tasse_handle) = &loaded_models.tasse {
        if let Some(gltf) = gltf_assets.get(tasse_handle) {
            commands.spawn((
                SceneRoot(gltf.scenes[0].clone()),
                Transform::from_xyz(2.0, 2.0, 0.0),
                RigidBody::Dynamic,
                Collider::sphere(0.5), // Temporärer Collider - anpassen!
            ));
            info!("Tasse spawned!");
        } else {
            all_ready = false;
        }
    }


    // Markiere als gespawnt, wenn alle Assets bereit waren
    if all_ready {
        *spawned = true;
    }
}
