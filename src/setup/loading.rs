// rust
use bevy::prelude::*;
use bevy::asset::LoadState;
use crate::AppState;
use crate::setup::assetloader::{LoadedModels, LoadedAssetSettings, AmbienceAudio};


#[derive(Component)]
pub struct LoadingMarker;

pub fn spawn_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        LoadingMarker,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Loading..."),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    info!("🔄 Ladescreen angezeigt");
}

pub fn despawn_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingMarker>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    info!("✅ Ladescreen entfernt");
}

pub fn check_assets_loaded_transition(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    loaded_models: Option<Res<LoadedModels>>,
    loaded_settings: Option<Res<LoadedAssetSettings>>,
    ambience: Option<Res<AmbienceAudio>>,
) {
    let Some(models) = loaded_models else { return; };
    let Some(_settings) = loaded_settings else { return; };

    let mut all_loaded = true;

    if let Some(h) = models.tasse.as_ref() {
        if !matches!(asset_server.load_state(h.id()), LoadState::Loaded) {
            all_loaded = false;
        }
    } else {
        all_loaded = false;
    }

    if let Some(amb) = ambience {
        if let Some(h) = &amb.0 {
            if !matches!(asset_server.load_state(h.id()), LoadState::Loaded) {
                all_loaded = false;
            }
        }
    }

    if all_loaded {
        next_state.set(AppState::Running);
        info!("🚀 Assets geladen — Wechsel zu AppState::Running");
    }
}
