use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct StateText;

#[derive(Component)]
pub struct LoadingProgressText;

/// Spawnt das FPS Overlay UI
pub fn setup_fps_overlay(mut commands: Commands) {
    // Root UI Container
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        left: Val::Px(10.0),
        flex_direction: FlexDirection::Column,
        ..default()
    }).with_children(|parent| {
        // FPS Text
        parent.spawn((
            Text::new("FPS: --"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            FpsText,
        ));

        // State Text
        parent.spawn((
            Text::new("State: Loading"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
            StateText,
        ));

        // Loading Progress Text
        parent.spawn((
            Text::new("Loading..."),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.5, 0.0)),
            LoadingProgressText,
        ));
    });
}

/// Update FPS Text
pub fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Farbe basierend auf FPS
                **text = format!("FPS: {:.1}", value);
            }
        }
    }
}

/// Update State Text
pub fn update_state_text(
    current_state: Res<State<crate::setup::appstate::AppState>>,
    mut query: Query<&mut Text, With<StateText>>,
) {
    for mut text in &mut query {
        **text = format!("State: {:?}", current_state.get());
    }
}

/// Update Loading Progress
pub fn update_loading_progress(
    asset_server: Res<bevy::asset::AssetServer>,
    asset_handles: Option<Res<crate::setup::assetloader::AssetHandles>>,
    mut query: Query<&mut Text, With<LoadingProgressText>>,
    current_state: Res<State<crate::setup::appstate::AppState>>,
) {
    use crate::setup::appstate::AppState;

    for mut text in &mut query {
        if *current_state.get() == AppState::Loading {
            if let Some(handles_res) = &asset_handles {
                let total = handles_res.0.len();
                let loaded = handles_res.0.iter().filter(|handle| {
                    matches!(
                        asset_server.get_load_state(handle.id()),
                        Some(bevy::asset::LoadState::Loaded)
                    )
                }).count();
                **text = format!("Loading: {}/{} assets", loaded, total);
            } else {
                **text = "Initializing...".to_string();
            }
        } else {
            **text = "Ready!".to_string();
        }
    }
}
