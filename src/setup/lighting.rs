use bevy::prelude::*;
use crate::setup::assetloader::LoadedAssetSettings;

pub fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 8000.0,
            ..default()
        },
        Transform::from_xyz(-2.0, 8.0, 2.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4, 0.0)),
    ));
}



pub fn spawn_ambient_light(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::srgb(1.0, 0.1, 0.1),
        brightness: 75.0,
        affects_lightmapped_meshes: true,
    });
}

pub fn spawn_environment_map_light(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<LoadedAssetSettings>,
) {
    let environment_map = asset_server.load(&settings.environment_map_path);

    commands.spawn(EnvironmentMapLight {
        diffuse_map: environment_map.clone(),
        specular_map: environment_map,
        intensity: 10000.0,
        ..default()
    });
    info!("🌤️ EnvironmentMapLight spawned: {}", settings.environment_map_path);
}
