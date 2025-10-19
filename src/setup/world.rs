use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::assetloader::{LoadedModels, AmbienceAudio};
use crate::setup::gltf_spawner::{GltfSpawnConfig, spawn_gltf_with_physics, spawn_primitive_with_physics};

#[derive(Component)]
pub struct AmbienceAudioMarker;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Hier keine Audio-Initialisierung mehr, um frühe Panics zu verm
}

/// Spawnt Ambience erst, wenn wir im Running-State sind und Assets geladen sind
pub fn spawn_ambience_when_ready(
    mut commands: Commands,
    ambience: Option<Res<AmbienceAudio>>,
    asset_server: Res<AssetServer>,
    existing: Query<(), With<AmbienceAudioMarker>>,
) {
    // Nur spawnen, wenn noch nicht vorhanden
    if !existing.is_empty() {
        return;
    }

    if let Some(amb) = ambience {
        if let Some(handle) = &amb.0 {
            // Prüfen, ob das Audio-Asset vollständig geladen ist
            if !matches!(
                asset_server.get_load_state(handle.id()),
                Some(bevy::asset::LoadState::Loaded)
            ) {
                return; // Asset noch nicht geladen, warten
            }

            commands.spawn((
                bevy::audio::AudioPlayer::new(handle.clone()),
                bevy::audio::PlaybackSettings::LOOP
                    .with_volume(bevy::audio::Volume::Linear(0.5))
                    .with_spatial(false), // Nicht-räumlich für Hintergrundmusik
                AmbienceAudioMarker,
            ));
            info!("🔊 Ambience audio spawned and playing!");
        }
    }
}

#[derive(Component)]
pub struct RadialGravity;

pub fn apply_radial_gravity(
    mut query: Query<(&Transform, &mut LinearVelocity), With<RadialGravity>>,
    time: Res<Time>,
) {
    let center = Vec3::ZERO;
    let strength = 0.5;

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
    loaded_models: Option<Res<LoadedModels>>, // optional machen, um Panics zu vermeiden
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
) {
    info!("🎲 Spawning initial objects!");

    // Wenn die erforderlichen Modell-Handles nicht verfügbar sind, frühzeitig beenden
    let Some(loaded_models) = loaded_models else {
        info!("ℹ️ LoadedModels resource missing; skipping initial object spawn.");
        return;
    };

    // Spawne einen Würfel mit Physik über Helper
    spawn_primitive_with_physics(
        &mut commands,
        meshes.add(Cuboid::from_length(2.0)),
        materials.add(Color::srgb_u8(0, 0, 0)),
        Transform::from_xyz(0.0, 4.0, 0.0),
        Collider::cuboid(2.0, 2.0, 2.0),
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
