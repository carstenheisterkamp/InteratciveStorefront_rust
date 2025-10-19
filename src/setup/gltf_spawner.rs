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
            mass: 0.2,
            restitution: 0.5,
            friction: 0.7,
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

/// Generische Funktion zum Spawnen von GLTF-Modellen mit Physik
///
/// # Best Practice
/// Verwendet separate Visual- und Collider-GLTFs:
/// - `visual_gltf`: Das sichtbare 3D-Modell
/// - `collider_gltf`: Separates GLTF mit vereinfachter Geometrie für Kollisionen
///
/// # Beispiel
/// ```rust
/// let config = GltfSpawnConfig::new(visual_handle)
///     .with_collider_gltf(collider_handle)
///     .with_transform(Transform::from_xyz(0.0, 5.0, 0.0))
///     .with_scale(1.5)
///     .with_mass(0.2)
///     .with_physics(0.5, 0.7)
///     .with_velocity(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO)
///     .with_radial_gravity(true);
///
/// spawn_gltf_with_physics(&mut commands, &gltf_assets, &gltf_mesh_assets, &meshes, config);
/// ```
pub fn spawn_gltf_with_physics(
    commands: &mut Commands,
    gltf_assets: &Assets<Gltf>,
    gltf_mesh_assets: &Assets<bevy::gltf::GltfMesh>,
    mesh_assets: &Assets<Mesh>,
    config: GltfSpawnConfig,
    uniform_scale: f32,
    radial_gravity_marker: Option<impl Component>,
) -> Option<Entity> {
    // Lade visuelles GLTF
    let visual_gltf = gltf_assets.get(&config.visual_gltf)?;

    // Versuche Collider aus separatem GLTF zu laden, sonst Fallback
    let collider = if let Some(collider_handle) = &config.collider_gltf {
        if let Some(collider_gltf) = gltf_assets.get(collider_handle) {
            find_collider_in_gltf(collider_gltf, gltf_mesh_assets, mesh_assets, uniform_scale)
                .unwrap_or_else(|| config.fallback_collider.clone())
        } else {
            config.fallback_collider
        }
    } else {
        config.fallback_collider.clone()
    };

    // Spawn Entity mit allen Components
    // HINWEIS: Collider muss bereits in der richtigen Größe übergeben werden.
    // Falls uniform_scale != 1.0, muss der fallback_collider bereits passend dimensioniert sein,
    // oder der GLTF-Collider muss in der GLTF-Datei die richtige Größe haben.
    // Für ConvexHull-Collider aus GLTF: Diese werden in der Originalgröße der GLTF-Mesh erstellt.
    let mut entity = commands.spawn((
        SceneRoot(visual_gltf.scenes[0].clone()),
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

/// Sucht nach einem Collider-Mesh im GLTF (by name convention)
///
/// Sucht nach Nodes/Meshes mit "collider", "collision" oder "col_" im Namen
/// und erstellt daraus einen ConvexHull-Collider.
fn find_collider_in_gltf(
    gltf: &Gltf,
    gltf_mesh_assets: &Assets<bevy::gltf::GltfMesh>,
    mesh_assets: &Assets<Mesh>,
    scale: f32,
) -> Option<Collider> {
    // 1) Suche über named_nodes (erwartet Node-Namen wie "tasse_collision")
    for (node_name, _node_handle) in &gltf.named_nodes {
        let name_lower = node_name.to_lowercase();
        if name_lower.contains("collider") || name_lower.contains("collision") || name_lower.contains("col_") {
            info!("🎯 Found collider candidate node: {}", node_name);

            if let Some(mesh_handle) = gltf.named_meshes.get(node_name) {
                info!("  ↳ Found named_mesh handle for node: {}", node_name);
                if let Some(gltf_mesh) = gltf_mesh_assets.get(mesh_handle) {
                    info!("  ↳ GltfMesh found, primitives: {}", gltf_mesh.primitives.len());
                    if let Some(primitive) = gltf_mesh.primitives.first() {
                        if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                            info!("  ↳ Underlying Mesh asset present, trying ConvexHull...");
                            // Skaliere Mesh vor ConvexHull-Erstellung
                            let scaled_mesh = scale_mesh_vertices(mesh, scale);
                            if let Some(collider) = Collider::convex_hull_from_mesh(&scaled_mesh) {
                                info!("✅ Created collider from node: {}", node_name);
                                return Some(collider);
                            } else {
                                info!("⚠️ ConvexHull creation failed for node: {}", node_name);
                            }
                        } else {
                            info!("⚠️ Mesh asset not yet available for primitive of node: {}", node_name);
                        }
                    } else {
                        info!("⚠️ No primitives in GltfMesh for node: {}", node_name);
                    }
                } else {
                    info!("⚠️ GltfMesh handle not yet resolved for node: {}", node_name);
                }
            } else {
                info!("⚠️ Node '{}' has no entry in named_meshes", node_name);
            }
        }
    }

    // 2) Fallback: scanne alle named_meshes direkt
    for (mesh_name, mesh_handle) in &gltf.named_meshes {
        let name_lower = mesh_name.to_lowercase();
        if name_lower.contains("collider") || name_lower.contains("collision") || name_lower.contains("col_") {
            info!("🔎 Fallback: found named_mesh: {}", mesh_name);
            if let Some(gltf_mesh) = gltf_mesh_assets.get(mesh_handle) {
                info!("  ↳ GltfMesh found, primitives: {}", gltf_mesh.primitives.len());
                if let Some(primitive) = gltf_mesh.primitives.first() {
                    if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                        info!("  ↳ Underlying Mesh asset present, trying ConvexHull...");
                        // Skaliere Mesh vor ConvexHull-Erstellung
                        let scaled_mesh = scale_mesh_vertices(mesh, scale);
                        if let Some(collider) = Collider::convex_hull_from_mesh(&scaled_mesh) {
                            info!("✅ Created collider from named_mesh: {}", mesh_name);
                            return Some(collider);
                        } else {
                            info!("⚠️ ConvexHull creation failed for named_mesh: {}", mesh_name);
                        }
                    } else {
                        info!("⚠️ Mesh asset not yet available for named_mesh: {}", mesh_name);
                    }
                } else {
                    info!("⚠️ No primitives in GltfMesh for named_mesh: {}", mesh_name);
                }
            } else {
                info!("⚠️ GltfMesh handle not yet resolved for named_mesh: {}", mesh_name);
            }
        }
    }

    info!("🔚 No collider found in GLTF (checked named_nodes + named_meshes).");
    None
}

/// Hilfsfunktion zum Skalieren der Vertex-Positionen eines Meshes
fn scale_mesh_vertices(mesh: &Mesh, scale: f32) -> Mesh {
    let mut scaled_mesh = mesh.clone();

    // Hole die Position-Attribute als lesbare Referenz
    if let Some(positions_attr) = scaled_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        // Erstelle einen neuen Vec mit skalierten Positionen
        let scaled_positions: Vec<[f32; 3]> = positions_attr
            .as_float3()
            .expect("Position attribute should be Float32x3")
            .iter()
            .map(|&[x, y, z]| [x * scale, y * scale, z * scale])
            .collect();

        // Setze die skalierten Positionen zurück ins Mesh
        scaled_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, scaled_positions);
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
    // HINWEIS: Collider muss bereits in der richtigen Größe übergeben werden.
    // uniform_scale Parameter wird derzeit nicht verwendet - Collider direkt passend dimensionieren.
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
