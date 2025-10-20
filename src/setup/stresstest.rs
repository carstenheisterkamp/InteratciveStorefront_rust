use bevy::prelude::*;
use crate::setup::assetloader::LoadedModels;
use crate::setup::gltf_spawner::{GltfSpawnConfig, spawn_gltf_with_physics};
use crate::setup::world::RadialGravity;
use rand::Rng;

#[derive(Component)]
pub struct StressTestObject;

#[derive(Resource)]
pub struct StressTestConfig {
    pub enabled: bool,
    pub spawn_rate: f32,
    pub max_objects: usize,
    pub spawn_timer: Timer,
    pub current_count: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            spawn_rate: 10.0,
            max_objects: 500,
            spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_count: 0,
        }
    }
}

pub fn spawn_stress_test_objects(
    mut commands: Commands,
    loaded_models: Option<Res<LoadedModels>>, // Optional machen
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
    mesh_assets: Res<Assets<Mesh>>,
    time: Res<Time>,
    mut config: ResMut<StressTestConfig>,
) {
    if !config.enabled || config.current_count >= config.max_objects {
        return;
    }

    // Früher Return wenn LoadedModels fehlt
    let Some(loaded_models) = loaded_models else {
        return;
    };

    config.spawn_timer.tick(time.delta());

    if config.spawn_timer.just_finished() {
        let mut rng = rand::rng();

        let objects_to_spawn = (config.spawn_rate * config.spawn_timer.duration().as_secs_f32()) as usize;

        for _ in 0..objects_to_spawn {
            if config.current_count >= config.max_objects {
                break;
            }

            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let radius = rng.random_range(0.0..30.0);
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = rng.random_range(5.0..20.0);

            let pos = Vec3::new(x, y, z);
            let mut direction = -pos.normalize();

            direction += Vec3::new(
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
            );
            let speed = rng.random_range(2.0..4.0);
            let linear_vel = direction.normalize() * speed;

            let angular_vel = Vec3::new(
                rng.random_range(-3.0..3.0),
                rng.random_range(-3.0..3.0),
                rng.random_range(-3.0..3.0),
            );

            // Zufällige Größe mit größerer Variation: 20% bis 200%
            let scale = rng.random_range(0.2..2.0);

            // BEST PRACTICE: Nutze generische spawn_gltf_with_physics Funktion
            if let Some(tasse_handle) = &loaded_models.tasse {
                let spawn_config = GltfSpawnConfig::new(tasse_handle.clone())
                    .with_collider_gltf(loaded_models.tasse_collider.clone().unwrap_or(tasse_handle.clone()))
                    .with_transform(Transform::from_xyz(x, y, z))
                    .with_scale(scale)
                    .with_mass(0.2)
                    .with_physics(0.0, 0.0)
                    .with_velocity(linear_vel, angular_vel)
                    .with_radial_gravity(true);

                if let Some(entity) = spawn_gltf_with_physics(
                    &mut commands,
                    &gltf_assets,
                    &gltf_mesh_assets,
                    &mesh_assets,
                    spawn_config,
                    scale,
                    Some(RadialGravity),
                ) {
                    // Füge noch den StressTestObject Marker hinzu
                    commands.entity(entity).insert(StressTestObject);
                }
            }

            config.current_count += 1;
        }
    }
}

/// Input-System um Stresstest zu steuern
pub fn stress_test_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<StressTestConfig>,
    query: Query<Entity, With<StressTestObject>>,
    mut commands: Commands,
) {
    // T - Toggle Stresstest an/aus
    if keyboard.just_pressed(KeyCode::KeyT) {
        config.enabled = !config.enabled;
        if config.enabled {
            info!("🔥 Stresstest gestartet! (Max: {} Objekte, {} Obj/s)",
                  config.max_objects, config.spawn_rate);
        } else {
            info!("⏸️  Stresstest pausiert");
        }
    }

    // C - Clear alle Stresstest-Objekte
    if keyboard.just_pressed(KeyCode::KeyC) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        config.current_count = 0;
        info!("🧹 Alle Stresstest-Objekte gelöscht");
    }

    // + - Spawn-Rate erhöhen
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        config.spawn_rate = (config.spawn_rate * 1.5).min(1000.0);
        info!("⬆️  Spawn-Rate: {:.1} Obj/s", config.spawn_rate);
    }

    // - - Spawn-Rate verringern
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        config.spawn_rate = (config.spawn_rate / 1.5).max(1.0);
        info!("⬇️  Spawn-Rate: {:.1} Obj/s", config.spawn_rate);
    }

    // M - Max-Objekte erhöhen
    if keyboard.just_pressed(KeyCode::KeyM) {
        config.max_objects = (config.max_objects + 500).min(10000);
        info!("📈 Max Objekte: {}", config.max_objects);
    }

    // N - Max-Objekte verringern
    if keyboard.just_pressed(KeyCode::KeyN) {
        config.max_objects = (config.max_objects.saturating_sub(500)).max(100);
        info!("📉 Max Objekte: {}", config.max_objects);
    }
}

/// Zeigt Stresstest-Info im Diagnostics-Overlay
pub fn update_stress_test_info(
    config: Res<StressTestConfig>,
    query: Query<&StressTestObject>,
) {
    // Wird nur alle paar Sekunden geloggt
    if config.spawn_timer.just_finished() && config.current_count % 50 == 0 {
        let actual_count = query.iter().count();
        if config.enabled {
            info!("📊 Stresstest: {}/{} Objekte aktiv",
                  actual_count, config.max_objects);
        }
    }
}
