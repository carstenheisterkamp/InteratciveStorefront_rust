use bevy::asset::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct AssetSettings {
    pub model_paths: Vec<String>,
    pub audio_paths: Vec<String>,
}

pub fn load_assets(settings: &AssetSettings, asset_server: &AssetServer) -> Vec<UntypedHandle> {
    let mut handles = Vec::new();
    for path in settings.model_paths.iter().chain(settings.audio_paths.iter()) {
        let handle = asset_server.load::<LoadedUntypedAsset>(path.clone()).untyped();
        handles.push(handle);
    }
    handles
}

/// Resource that stores all loaded asset handles so other systems can check their load state.
#[derive(Resource, Default)]
pub struct AssetHandles(pub Vec<UntypedHandle>);

/// Startup system: read config/settings.json, load assets and insert `AssetHandles` resource.
pub fn load_assets_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_path = "config/settings.json";
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
