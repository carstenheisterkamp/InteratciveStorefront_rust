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

    handles
}

#[derive(Resource, Default)]
pub struct AssetHandles(pub Vec<UntypedHandle>);

/// Startup system: read config/settings.json, load assets and insert `AssetHandles` resource.
pub fn load_assets_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_path = "assets/config/settings.json";
    match fs::read_to_string(config_path) {
        Ok(contents) => match serde_json::from_str::<AssetSettings>(&contents) {
            Ok(settings) => {
                let handles = load_assets(&settings, &asset_server);
                info!("Requested {} assets to load", handles.len());
                commands.insert_resource(AssetHandles(handles));
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
