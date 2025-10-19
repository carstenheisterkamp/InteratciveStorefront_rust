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
        Mesh3d(meshes.add(Cylinder::new(40.0, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 100))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
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

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Restitution::new(1.0),
        Friction::new(0.75),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 255))),
        Transform::from_xyz(0.0, 4.0, 0.0),
        RadialGravity,
    ));

    // GLTF Modelle spawnen mit VEREINFACHTEM COLLIDER
    if let Some(tasse_handle) = &loaded_models.tasse {
        if let Some(gltf) = gltf_assets.get(tasse_handle) {
            // Versuche zuerst einen vereinfachten Collider-Mesh zu finden
            // Namenskonvention: "*_collider" oder "*_collision"
            let collider = find_collider_in_gltf(gltf, &gltf_mesh_assets, &meshes)
                .unwrap_or_else(|| {
                    info!("⚠️ Kein Collider-Mesh gefunden, verwende einfachen Cylinder");
                    // VEREINFACHTER Collider - viel performanter als ConvexHull!
                    Collider::cylinder(0.15, 0.5)
                });

            commands.spawn((
                SceneRoot(gltf.scenes[0].clone()),
                Transform::from_xyz(2.0, 2.0, 2.0),
                RigidBody::Dynamic,
                collider,
                Restitution::new(0.5),
                Friction::new(0.7),
                RadialGravity,
            ));
            info!("☕ Tasse spawned with optimized collider!");
        }
    }

    info!("✅ All initial objects spawned!");
}

/// Sucht nach einem Collider-Mesh im GLTF (by name convention)
fn find_collider_in_gltf(
    gltf: &Gltf,
    gltf_mesh_assets: &Assets<bevy::gltf::GltfMesh>,
    mesh_assets: &Assets<Mesh>,
) -> Option<Collider> {
    // Durchsuche alle Named Nodes nach Collider-Meshes
    for (node_name, _node_handle) in &gltf.named_nodes {
        let name_lower = node_name.to_lowercase();

        // Check für Collider-Naming Convention
        if name_lower.contains("collider") || name_lower.contains("collision") || name_lower.contains("col_") {
            info!("🎯 Found collider mesh: {}", node_name);

            // Versuche den Mesh aus diesem Node zu holen
            if let Some(mesh_handle) = gltf.named_meshes.get(node_name) {
                if let Some(gltf_mesh) = gltf_mesh_assets.get(mesh_handle) {
                    if let Some(primitive) = gltf_mesh.primitives.first() {
                        if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                            // Erstelle ConvexHull aus dem vereinfachten Mesh
                            if let Some(collider) = Collider::convex_hull_from_mesh(mesh) {
                                info!("✅ Created collider from: {}", node_name);
                                return Some(collider);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
