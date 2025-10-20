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

#[derive(Component)]
pub struct LightInfoText;

#[derive(Resource)]
pub struct DiagnosticsOverlayVisible {
    pub visible: bool,
}

impl Default for DiagnosticsOverlayVisible {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Component)]
pub struct DiagnosticsOverlayRoot;

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

pub fn toggle_diagnostics_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visibility: ResMut<DiagnosticsOverlayVisible>,
    mut query: Query<&mut Visibility, With<DiagnosticsOverlayRoot>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        visibility.visible = !visibility.visible;

        for mut vis in &mut query {
            *vis = if visibility.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}


pub fn setup_fps_overlay(mut commands: Commands) {
    // Root UI Container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        DiagnosticsOverlayRoot,
    )).with_children(|parent| {
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

        // Average FPS Text
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

        // Light Info Text
        parent.spawn((
            Text::new("Lichter: --"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            LightInfoText,
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

/// Update Light Info Text
pub fn update_light_info_text(
    ambient: Option<Res<AmbientLight>>,
    env_query: Query<(Entity, &EnvironmentMapLight, Option<&Visibility>)>,
    dir_query: Query<(Entity, &DirectionalLight, Option<&Visibility>)>,
    point_query: Query<(Entity, &PointLight, Option<&Visibility>)>,
    spot_query: Query<(Entity, &SpotLight, Option<&Visibility>)>,
    mut text_query: Query<&mut Text, With<LightInfoText>>,
) {
    for mut text in &mut text_query {
        let mut info = String::from("💡 Lights:\n");

        // AmbientLight (Resource)
        if let Some(ref amb) = ambient {
            info.push_str(&format!("  Ambient: {:.1}\n", amb.brightness));
        } else {
            info.push_str("  Ambient: None\n");
        }

        // EnvironmentMapLight
        let env_count = env_query.iter().count();
        info.push_str(&format!("  EnvMap: {} ", env_count));
        if env_count > 0 {
            let active = env_query.iter().filter(|(_, _, vis)| {
                vis.map_or(true, |v| *v != Visibility::Hidden)
            }).count();
            info.push_str(&format!("({} active)\n", active));
        } else {
            info.push('\n');
        }

        // DirectionalLight
        let dir_count = dir_query.iter().count();
        info.push_str(&format!("  Directional: {} ", dir_count));
        if dir_count > 0 {
            let active = dir_query.iter().filter(|(_, _, vis)| {
                vis.map_or(true, |v| *v != Visibility::Hidden)
            }).count();
            info.push_str(&format!("({} active)\n", active));
        } else {
            info.push('\n');
        }

        // PointLight
        let point_count = point_query.iter().count();
        info.push_str(&format!("  Point: {} ", point_count));
        if point_count > 0 {
            let active = point_query.iter().filter(|(_, _, vis)| {
                vis.map_or(true, |v| *v != Visibility::Hidden)
            }).count();
            info.push_str(&format!("({} active)\n", active));
        } else {
            info.push('\n');
        }

        // SpotLight
        let spot_count = spot_query.iter().count();
        info.push_str(&format!("  Spot: {} ", spot_count));
        if spot_count > 0 {
            let active = spot_query.iter().filter(|(_, _, vis)| {
                vis.map_or(true, |v| *v != Visibility::Hidden)
            }).count();
            info.push_str(&format!("({} active)", active));
        }

        **text = info;
    }
}
