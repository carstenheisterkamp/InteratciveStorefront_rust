use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct LowestFpsText;

#[derive(Component)]
pub struct AverageFpsText;

#[derive(Component)]
pub struct StateText;

#[derive(Component)]
pub struct LoadingProgressText;

#[derive(Component)]
pub struct StressTestInfoText;

/// Resource um den niedrigsten FPS-Wert zu tracken
#[derive(Resource)]
pub struct LowestFps {
    pub value: f64,
}

impl Default for LowestFps {
    fn default() -> Self {
        Self {
            value: f64::MAX, // Starte mit maximalem Wert
        }
    }
}

/// Resource um den durchschnittlichen FPS-Wert zu berechnen
#[derive(Resource)]
pub struct AverageFps {
    pub total_fps: f64,
    pub frame_count: u64,
}

impl Default for AverageFps {
    fn default() -> Self {
        Self {
            total_fps: 0.0,
            frame_count: 0,
        }
    }
}

impl AverageFps {
    pub fn average(&self) -> f64 {
        if self.frame_count > 0 {
            self.total_fps / self.frame_count as f64
        } else {
            0.0
        }
    }
}

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

        // Average FPS Text (NEU)
        parent.spawn((
            Text::new("Avg: --"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.8, 1.0)),
            AverageFpsText,
        ));

        // Lowest FPS Text
        parent.spawn((
            Text::new("Lowest: --"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.5, 0.0)),
            LowestFpsText,
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

        // Stresstest Info Text
        parent.spawn((
            Text::new("Press T to start stresstest"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 1.0, 1.0)),
            StressTestInfoText,
        ));
    });
}

/// Update FPS Text und tracke niedrigsten Wert sowie Average
pub fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut lowest_fps: ResMut<LowestFps>,
    mut average_fps: ResMut<AverageFps>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Tracke niedrigsten FPS-Wert
                if value < lowest_fps.value && value > 0.0 {
                    lowest_fps.value = value;
                }

                // Berechne Average FPS
                if value > 0.0 {
                    average_fps.total_fps += value;
                    average_fps.frame_count += 1;
                }

                **text = format!("FPS: {:.1}", value);
            }
        }
    }
}

/// Update Average FPS Text
pub fn update_average_fps_text(
    average_fps: Res<AverageFps>,
    mut query: Query<&mut Text, With<AverageFpsText>>,
) {
    for mut text in &mut query {
        if average_fps.frame_count > 0 {
            **text = format!("Avg: {:.1} FPS", average_fps.average());
        } else {
            **text = "Avg: --".to_string();
        }
    }
}

/// Update Lowest FPS Text
pub fn update_lowest_fps_text(
    lowest_fps: Res<LowestFps>,
    mut query: Query<&mut Text, With<LowestFpsText>>,
) {
    for mut text in &mut query {
        if lowest_fps.value < f64::MAX {
            **text = format!("Lowest: {:.1} FPS", lowest_fps.value);
        } else {
            **text = "Lowest: --".to_string();
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

/// Update Stresstest Info Text
pub fn update_stress_test_info_text(
    config: Res<crate::setup::stresstest::StressTestConfig>,
    query: Query<&crate::setup::stresstest::StressTestObject>,
    mut text_query: Query<&mut Text, With<StressTestInfoText>>,
) {
    for mut text in &mut text_query {
        let actual_count = query.iter().count();
        if config.enabled {
            **text = format!("🔥 Stresstest: {}/{} | {:.0} obj/s",
                            actual_count, config.max_objects, config.spawn_rate);
        } else {
            **text = format!("⏸️ Stresstest: {} objects | Press T to start", actual_count);
        }
    }
}
