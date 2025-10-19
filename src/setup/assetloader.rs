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
    pub models: ModelsConfig,
    pub environment: EnvironmentConfig,
}

/// Audio-Entry mit Metadaten (pfad, loop-flag, optionaler volume)
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
    pub music: HashMap<String, AudioEntry>,
}

/// Ressource: optionaler Handle zur Ambience-Audio (looped)
#[derive(Resource, Clone)]
pub struct AmbienceAudio(pub Option<Handle<AudioSource>>);

/// Ressource: alle SFX-Handles nach Namen für schnellen Zugriff
#[derive(Resource, Default)]
pub struct SoundFx(pub HashMap<String, Handle<AudioSource>>);

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

/// Resource that stores typed handles for loaded models
#[derive(Resource, Default)]
pub struct LoadedModels {
    pub tasse: Option<Handle<Gltf>>,
    pub tasse_collider: Option<Handle<Gltf>>,
    pub test: Option<Handle<Gltf>>,
}

#[derive(Resource, Clone)]
pub struct LoadedAssetSettings {
    pub environment_map_path: String,
    pub ambience_path: Option<String>,
}

pub fn load_assets(settings: &AssetSettings, asset_server: &AssetServer) -> Vec<UntypedHandle> {
    let mut handles = Vec::new();

    for entry in settings.assets.audio.sounds.values() {
        info!("Loading audio asset: {}", entry.path);
        let handle = asset_server.load::<AudioSource>(entry.path.clone()).untyped();
        handles.push(handle);
    }
    for entry in settings.assets.audio.music.values() {
        info!("Loading music asset: {}", entry.path);
        let handle = asset_server.load::<AudioSource>(entry.path.clone()).untyped();
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
                    ambience_path,
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

pub fn load_audio_resources(
    audio_cfg: &AudioConfig,
    asset_server: &AssetServer,
) -> (AmbienceAudio, SoundFx) {
    let mut sfx_map = HashMap::new();

    // Alle SFX laden
    for (name, entry) in &audio_cfg.sounds {
        let handle = asset_server.load::<AudioSource>(entry.path.clone());
        sfx_map.insert(name.clone(), handle);
    }

    // Ambience suchen (z.B. unter "ambience" in music oder sounds)
    let ambience_path = audio_cfg
        .music
        .get("ambience")
        .or_else(|| audio_cfg.sounds.get("ambience"))
        .map(|e| e.path.clone());

    let ambience_handle = ambience_path.map(|p| asset_server.load::<AudioSource>(p));

    (AmbienceAudio(ambience_handle), SoundFx(sfx_map))
}
