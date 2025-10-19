use bevy::asset::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct AssetSettings {
    pub assets: AssetsConfig,
}

#[derive(Deserialize)]
pub struct AssetsConfig {
    pub audio: AudioConfig,
    pub models: ModelsConfig,
    pub environment: EnvironmentConfig,
}

#[derive(Deserialize)]
pub struct AudioConfig {
    pub sounds: HashMap<String, String>,
    pub music: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ModelsConfig {
    #[serde(flatten)]
    pub models: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct EnvironmentConfig {
    pub map: String,
}

pub fn load_assets(settings: &AssetSettings, asset_server: &AssetServer) -> Vec<UntypedHandle> {
    let mut handles = Vec::new();

    for sound in settings.assets.audio.sounds.values() {
        info!("Loading audio asset: {}", sound);
        let handle = asset_server.load::<AudioSource>(sound.clone()).untyped();
        handles.push(handle);
    }
    for music in settings.assets.audio.music.values() {
        info!("Loading music asset: {}", music);
        let handle = asset_server.load::<AudioSource>(music.clone()).untyped();
        handles.push(handle);
    }

    for (name, path) in &settings.assets.models.models {
        info!("Loading model '{}': {}", name, path);
        let handle = asset_server.load::<Gltf>(path.clone()).untyped();
        handles.push(handle);
    }

    // Load environment map
    let env_map_path = &settings.assets.environment.map;
    info!("Loading environment map: {}", env_map_path);
    let handle = asset_server.load::<Image>(env_map_path.clone()).untyped();
    handles.push(handle);

    handles
}

#[derive(Resource, Default)]
pub struct AssetHandles(pub Vec<UntypedHandle>);

/// Resource that stores typed handles for loaded models
#[derive(Resource, Default)]
pub struct LoadedModels {
    pub tasse: Option<Handle<Gltf>>,
    pub test: Option<Handle<Gltf>>,
    pub tasse_collider: Option<Handle<Gltf>>,
}

/// Resource that stores the loaded asset settings for use by other systems
#[derive(Resource, Clone)]
pub struct LoadedAssetSettings {
    pub environment_map_path: String,
}

/// Startup system: read config/settings.json, load assets and insert `AssetHandles` resource.
pub fn load_assets_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_path = "assets/config/settings.json";
    match fs::read_to_string(config_path) {
        Ok(contents) => match serde_json::from_str::<AssetSettings>(&contents) {
            Ok(settings) => {
                let handles = load_assets(&settings, &asset_server);
                info!("Requested {} assets to load", handles.len());
                commands.insert_resource(AssetHandles(handles));
                
                // Load and store typed model handles for easy access
                let mut loaded_models = LoadedModels::default();
                
                if let Some(tasse_path) = settings.assets.models.models.get("tasse") {
                    loaded_models.tasse = Some(asset_server.load(tasse_path.clone()));
                }
                if let Some(tasse_collider_path) = settings.assets.models.models.get("tasse_collider") {
                    loaded_models.tasse_collider = Some(asset_server.load(tasse_collider_path.clone()));
                }
                if let Some(test_path) = settings.assets.models.models.get("test") {
                    loaded_models.test = Some(asset_server.load(test_path.clone()));
                }
                
                commands.insert_resource(loaded_models);

                // Store the settings for use by other systems (e.g., lighting)
                commands.insert_resource(LoadedAssetSettings {
                    environment_map_path: settings.assets.environment.map.clone(),
                });
            }
            Err(e) => {
                warn!("Failed to parse {}: {}. No assets will be loaded.", config_path, e);
                commands.insert_resource(AssetHandles::default());
            }
        },
        Err(e) => {
            warn!("Failed to read {}: {}. No assets will be loaded.", config_path, e);
            commands.insert_resource(AssetHandles::default());
        }
    }
}
