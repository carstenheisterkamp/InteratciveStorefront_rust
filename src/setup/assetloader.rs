use bevy::asset::*;
use bevy::audio::AudioSource;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
pub struct AssetSettings {
    pub assets: AssetsConfig,
}

#[derive(Deserialize)]
pub struct AssetsConfig {
    pub audio: AudioConfig,
    pub textures: HashMap<String, String>, // <- hinzufügen
    pub models: ModelsConfig,
    pub environment: EnvironmentConfig,
}

#[derive(Deserialize, Clone)]
pub struct AudioEntry {
    pub path: String,
    #[serde(default)]
    pub looped: bool,
    #[serde(default)]
    pub volume: Option<f32>,
    #[serde(default)]
    pub streaming: bool,
}

#[derive(Deserialize)]
pub struct AudioConfig {
    pub sounds: HashMap<String, AudioEntry>,
    #[serde(default)]
    pub music: HashMap<String, AudioEntry>, // <- optional machen
}

#[derive(Resource, Clone)]
pub struct AmbienceAudio(pub Option<Handle<AudioSource>>);

#[derive(Deserialize)]
pub struct ModelsConfig {
    #[serde(flatten)]
    pub models: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct EnvironmentConfig {
    pub map: String,
}

#[derive(Resource, Default)]
pub struct AssetHandles(pub Vec<UntypedHandle>);

#[derive(Resource, Default)]
pub struct LoadedTextures {
    pub grid_texture: Option<Handle<Image>>,
    pub dust_particle: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct LoadedModels {
    pub tasse: Option<Handle<Gltf>>,
    pub tasse_collider: Option<Handle<Gltf>>,
    pub plant: Option<Handle<Gltf>>,
    pub plant_collider: Option<Handle<Gltf>>,
}

#[derive(Resource, Clone)]
pub struct LoadedAssetSettings {
    pub environment_map_path: String,
}

pub fn load_assets(settings: &AssetSettings, asset_server: &AssetServer) -> Vec<UntypedHandle> {
    let mut handles = Vec::new();

    for entry in settings.assets.audio.sounds.values() {
        info!("Loading audio asset: {}", entry.path);
        let handle: Handle<AudioSource> = asset_server.load(entry.path.clone());
        handles.push(handle.untyped());
    }

    for entry in settings.assets.audio.music.values() {
        info!("Loading music asset: {}", entry.path);
        let handle: Handle<AudioSource> = asset_server.load(entry.path.clone());
        handles.push(handle.untyped());
    }

    for (name, path) in &settings.assets.textures {
        info!("Loading texture '{}': {}", name, path);
        let handle: Handle<Image> = asset_server.load(path.clone());
        handles.push(handle.untyped());
    }

    for (name, path) in &settings.assets.models.models {
        info!("Loading model '{}': {}", name, path);
        let handle: Handle<Gltf> = asset_server.load(path.clone());
        handles.push(handle.untyped());
    }

    let env_map_path = &settings.assets.environment.map;
    info!("Loading environment map: {}", env_map_path);
    let handle: Handle<Image> = asset_server.load(env_map_path.clone());
    handles.push(handle.untyped());

    handles
}

pub fn load_assets_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_path = "assets/config/settings.json";
    match fs::read_to_string(config_path) {
        Ok(contents) => match serde_json::from_str::<AssetSettings>(&contents) {
            Ok(settings) => {
                let handles = load_assets(&settings, &asset_server);
                info!("Requested {} assets to load", handles.len());
                commands.insert_resource(AssetHandles(handles));

                let mut loaded_textures = LoadedTextures::default();
                if let Some(path) = settings.assets.textures.get("grid_texture") {
                    loaded_textures.grid_texture = Some(asset_server.load(path.clone()));
                }
                if let Some(path) = settings.assets.textures.get("dust_particle") {
                    loaded_textures.dust_particle = Some(asset_server.load(path.clone()));
                }
                commands.insert_resource(loaded_textures);

                // Load and store typed model handles for easy access
                let mut loaded_models = LoadedModels::default();

                if let Some(tasse_path) = settings.assets.models.models.get("tasse") {
                    loaded_models.tasse = Some(asset_server.load(tasse_path.clone()));
                }
                if let Some(tasse_collider_path) = settings.assets.models.models.get("tasse_collider") {
                    loaded_models.tasse_collider = Some(asset_server.load(tasse_collider_path.clone()));
                }
                if let Some(plant_path) = settings.assets.models.models.get("plant") {
                    loaded_models.plant= Some(asset_server.load(plant_path.clone()));
                }
                if let Some(plant_path) = settings.assets.models.models.get("plant") {
                    loaded_models.plant = Some(asset_server.load(plant_path.clone()));
                }

                commands.insert_resource(loaded_models);

                let ambience_path = settings
                    .assets
                    .audio
                    .sounds
                    .get("ambience")
                    .map(|e| e.path.clone())
                    .or_else(|| settings.assets.audio.music.get("ambience").map(|e| e.path.clone()));

                // Lade optionalen Ambience-Handle und speichere als Ressource
                let ambience_handle = ambience_path
                    .as_ref()
                    .map(|p| asset_server.load::<AudioSource>(p.clone()));
                commands.insert_resource(AmbienceAudio(ambience_handle));

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
