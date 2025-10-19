use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::assetloader::LoadedModels;
use rand::Rng;

/// Marker-Component für Stresstest-Objekte
#[derive(Component)]
pub struct StressTestObject;

/// Resource zur Konfiguration des Stresstests
#[derive(Resource)]
pub struct StressTestConfig {
    pub enabled: bool,
    pub spawn_rate: f32,  // Objekte pro Sekunde
    pub max_objects: usize,
    pub spawn_timer: Timer,
    pub current_count: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            spawn_rate: 10.0,  // 10 Objekte pro Sekunde
            max_objects: 1000,
            spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_count: 0,
        }
    }
}

/// Spawnt kontinuierlich neue Objekte für den Stresstest
pub fn spawn_stress_test_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loaded_models: Res<LoadedModels>,
    gltf_assets: Res<Assets<Gltf>>,
    _gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
    time: Res<Time>,
    mut config: ResMut<StressTestConfig>,
) {
    if !config.enabled || config.current_count >= config.max_objects {
        return;
    }

    config.spawn_timer.tick(time.delta());

    if config.spawn_timer.just_finished() {
        let mut rng = rand::rng();

        // Wie viele Objekte spawnen wir diesen Frame?
        let objects_to_spawn = (config.spawn_rate * config.spawn_timer.duration().as_secs_f32()) as usize;

        for _ in 0..objects_to_spawn {
            if config.current_count >= config.max_objects {
                break;
            }

            // Zufällige Position im Kreis über dem Boden
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let radius = rng.random_range(0.0..30.0);
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = rng.random_range(5.0..20.0);

            // Zufällige Geschwindigkeit
            let linear_vel = Vec3::new(
                rng.random_range(-2.0..2.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(-2.0..2.0),
            );

            // 50/50 Chance für Würfel oder Tasse
            if rng.random_bool(0.5) {
                // Würfel spawnen
                let size = rng.random_range(0.5..1.5);
                let color = Color::srgb_u8(
                    rng.random_range(100..255),
                    rng.random_range(100..255),
                    rng.random_range(100..255),
                );

                commands.spawn((
                    RigidBody::Dynamic,
                    Collider::cuboid(size, size, size),
                    Restitution::new(0.7),
                    Friction::new(0.5),
                    LinearVelocity(linear_vel),
                    AngularVelocity(Vec3::new(
                        rng.random_range(-5.0..5.0),
                        rng.random_range(-5.0..5.0),
                        rng.random_range(-5.0..5.0),
                    )),
                    Mesh3d(meshes.add(Cuboid::from_length(size))),
                    MeshMaterial3d(materials.add(color)),
                    Transform::from_xyz(x, y, z),
                    StressTestObject,
                    crate::setup::world::RadialGravity,
                ));
            } else {
                // Tasse spawnen mit VEREINFACHTEM COLLIDER (Cylinder statt Mesh!)
                if let Some(tasse_handle) = &loaded_models.tasse {
                    if let Some(gltf) = gltf_assets.get(tasse_handle) {
                        // Nutze einfachen Cylinder-Collider - DEUTLICH performanter!
                        let collider = Collider::cylinder(0.15, 0.5);

                        commands.spawn((
                            SceneRoot(gltf.scenes[0].clone()),
                            Transform::from_xyz(x, y, z),
                            RigidBody::Dynamic,
                            collider,
                            Restitution::new(0.5),
                            Friction::new(0.7),
                            LinearVelocity(linear_vel),
                            StressTestObject,
                            crate::setup::world::RadialGravity,
                        ));
                    }
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
