use bevy::prelude::*;
use avian3d::prelude::*;

/// Konfiguration für das Spawnen von GLTF-Modellen mit Physik (Builder Pattern)
#[derive(Clone)]
pub struct GltfSpawnConfig {
    pub visual_gltf: Handle<Gltf>,
    pub collider_gltf: Option<Handle<Gltf>>,
    pub transform: Transform,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub fallback_collider: Collider,
    pub apply_radial_gravity: bool,
}

impl GltfSpawnConfig {
    pub fn new(visual_gltf: Handle<Gltf>) -> Self {
        Self {
            visual_gltf,
            collider_gltf: None,
            transform: Transform::default(),
            mass: 100.0,
            restitution: 0.0,
            friction: 0.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            fallback_collider: Collider::cylinder(0.15, 0.5),
            apply_radial_gravity: false,
        }
    }

    /// Builder-Methoden für einfache Konfiguration
    pub fn with_collider_gltf(mut self, collider: Handle<Gltf>) -> Self {
        self.collider_gltf = Some(collider);
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.transform.scale = Vec3::splat(scale);
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_physics(mut self, restitution: f32, friction: f32) -> Self {
        self.restitution = restitution;
        self.friction = friction;
        self
    }

    pub fn with_velocity(mut self, linear: Vec3, angular: Vec3) -> Self {
        self.linear_velocity = linear;
        self.angular_velocity = angular;
        self
    }

    pub fn with_radial_gravity(mut self, enabled: bool) -> Self {
        self.apply_radial_gravity = enabled;
        self
    }

    pub fn with_fallback_collider(mut self, collider: Collider) -> Self {
        self.fallback_collider = collider;
        self
    }
}

pub fn spawn_gltf_with_physics(
    commands: &mut Commands,
    gltf_assets: &Assets<Gltf>,
    gltf_mesh_assets: &Assets<bevy::gltf::GltfMesh>,
    mesh_assets: &Assets<Mesh>,
    config: GltfSpawnConfig,
    _uniform_scale: f32, // Nicht mehr verwendet - Transform.scale übernimmt die Skalierung
    radial_gravity_marker: Option<impl Component>,
) -> Option<Entity> {
    let visual_gltf = gltf_assets.get(&config.visual_gltf)?;

    let collider = if let Some(collider_handle) = &config.collider_gltf {
        if let Some(collider_gltf) = gltf_assets.get(collider_handle) {
            find_collider_in_gltf(collider_gltf, gltf_mesh_assets, mesh_assets)
                .unwrap_or_else(|| config.fallback_collider.clone())
        } else {
            config.fallback_collider
        }
    } else {
        find_collider_in_gltf(visual_gltf, gltf_mesh_assets, mesh_assets)
            .unwrap_or_else(|| config.fallback_collider.clone())
    };

    // Szene sicher ermitteln (kein Index 0 ohne Check)
    let Some(scene_handle) = visual_gltf.scenes.first().cloned() else {
        warn!("GLTF has no scenes; skipping spawn");
        return None;
    };

    let mut entity = commands.spawn((
        SceneRoot(scene_handle),
        config.transform,
        RigidBody::Dynamic,
        collider,
        Mass(config.mass),
        Restitution::new(config.restitution),
        Friction::new(config.friction),
    ));

    // Optional: Velocity
    if config.linear_velocity != Vec3::ZERO {
        entity.insert(LinearVelocity(config.linear_velocity));
    }
    if config.angular_velocity != Vec3::ZERO {
        entity.insert(AngularVelocity(config.angular_velocity));
    }

    // Optional: Radiale Gravitation Marker (falls bereitgestellt)
    if config.apply_radial_gravity {
        if let Some(marker) = radial_gravity_marker {
            entity.insert(marker);
        }
    }
    Some(entity.id())
}

fn find_collider_in_gltf(
    gltf: &Gltf,
    gltf_mesh_assets: &Assets<bevy::gltf::GltfMesh>,
    mesh_assets: &Assets<Mesh>,
) -> Option<Collider> {
    for (node_name, _node_handle) in &gltf.named_nodes {
        let name_lower = node_name.to_lowercase();
        if name_lower.contains("collider") || name_lower.contains("collision") || name_lower.contains("col_") {
            if let Some(mesh_handle) = gltf.named_meshes.get(node_name) {
                if let Some(gltf_mesh) = gltf_mesh_assets.get(mesh_handle) {
                    if let Some(primitive) = gltf_mesh.primitives.first() {
                        if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                            let scaled_mesh = scale_mesh_vertices(mesh, 1.0);
                            if let Some(collider) = Collider::convex_hull_from_mesh(&scaled_mesh) {
                                return Some(collider);
                            }
                        }
                    }
                }
            }
        }
    }

    for (mesh_name, mesh_handle) in &gltf.named_meshes {
        let name_lower = mesh_name.to_lowercase();
        if name_lower.contains("collider") || name_lower.contains("collision") || name_lower.contains("col_") {
            if let Some(gltf_mesh) = gltf_mesh_assets.get(mesh_handle) {
                if let Some(primitive) = gltf_mesh.primitives.first() {
                    if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                        let scaled_mesh = scale_mesh_vertices(mesh, 1.0);
                        if let Some(collider) = Collider::convex_hull_from_mesh(&scaled_mesh) {
                            return Some(collider);
                        }
                    }
                }
            }
        }
    }
    None
}

fn scale_mesh_vertices(mesh: &Mesh, scale: f32) -> Mesh {
    let mut scaled_mesh = mesh.clone();

    if let Some(positions_attr) = scaled_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(original_positions) = positions_attr.as_float3() {
            let scaled_positions: Vec<[f32; 3]> = original_positions
                .iter()
                .map(|&[x, y, z]| [x * scale, y * scale, z * scale])
                .collect();
            scaled_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, scaled_positions);
        } else {
            // Unerwartetes Format: nicht skalieren, nur warnen
            warn!("Mesh position attribute is not Float32x3; skipping scale");
        }
    }

    scaled_mesh
}

/// Spawnt ein einfaches Mesh (Primitive) mit Physik-Komponenten nach dem gleichen Muster wie GLTF
pub fn spawn_primitive_with_physics(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    collider: Collider,
    mass: f32,
    restitution: f32,
    friction: f32,
    linear_velocity: Vec3,
    angular_velocity: Vec3,
    _uniform_scale: f32,
    radial_gravity_marker: Option<impl Component>,
) -> Entity {
    let mut entity = commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        RigidBody::Dynamic,
        collider,
        Mass(mass),
        Restitution::new(restitution),
        Friction::new(friction),
    ));
    if linear_velocity != Vec3::ZERO {
        entity.insert(LinearVelocity(linear_velocity));
    }
    if angular_velocity != Vec3::ZERO {
        entity.insert(AngularVelocity(angular_velocity));
    }
    if let Some(marker) = radial_gravity_marker {
        entity.insert(marker);
    }
    entity.id()
}

/// Deaktiviert die Collider-Visualisierung beim Start
pub fn disable_physics_gizmos_on_startup(
    mut gizmos_config: ResMut<GizmoConfigStore>,
) {
    let (config, _) = gizmos_config.config_mut::<PhysicsGizmos>();
    config.enabled = false;
}

/// Toggle Collider-Visualisierung mit Taste 'V'
pub fn toggle_physics_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmos_config: ResMut<GizmoConfigStore>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        let (config, _) = gizmos_config.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
        if config.enabled {
            info!("👁️  Collider-Visualisierung AN");
        } else {
            info!("🙈 Collider-Visualisierung AUS");
        }
    }
}
