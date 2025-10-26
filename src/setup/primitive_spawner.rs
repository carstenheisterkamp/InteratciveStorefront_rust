use avian3d::prelude::{AngularVelocity, Collider, Friction, LinearVelocity, Mass, Restitution, RigidBody};
use bevy::asset::Handle;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, Component, Entity, Transform};

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