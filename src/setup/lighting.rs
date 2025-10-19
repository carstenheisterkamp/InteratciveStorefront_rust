use bevy::prelude::*;
use crate::setup::assetloader::LoadedAssetSettings;

pub fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            ..default()
        },
        Transform::from_xyz(-2.0, 8.0, 2.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4, 0.0)),
    ));
}

pub fn spawn_ambient_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8, // Etwas heller, da Schatten aus sind
        affects_lightmapped_meshes: false,
    });
}

pub fn spawn_environment_map_light(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_settings: Option<Res<LoadedAssetSettings>>,
) {
    if let Some(settings) = loaded_settings {
        let environment_map = asset_server.load(&settings.environment_map_path);

        commands.spawn(EnvironmentMapLight {
            diffuse_map: environment_map.clone(),
            specular_map: environment_map,
            intensity: 5000.0, // Reduziert für bessere Performance
            ..default()
        });
        info!("🌤️ EnvironmentMapLight spawned: {}", settings.environment_map_path);
    } else {
        info!("ℹ️ LoadedAssetSettings not available yet; skipping EnvironmentMapLight this frame.");
    }
}

/// Update-System: spawnt das EnvironmentMapLight genau einmal, sobald Settings verfügbar sind
pub fn ensure_environment_map_light_once(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_settings: Option<Res<LoadedAssetSettings>>,
    existing: Query<(), With<EnvironmentMapLight>>,
) {
    if existing.is_empty() {
        if let Some(settings) = loaded_settings {
            let environment_map = asset_server.load(&settings.environment_map_path);
            commands.spawn(EnvironmentMapLight {
                diffuse_map: environment_map.clone(),
                specular_map: environment_map,
                intensity: 5000.0,
                ..default()
            });
            info!("🌤️ EnvironmentMapLight ensured: {}", settings.environment_map_path);
        }
    }
}
