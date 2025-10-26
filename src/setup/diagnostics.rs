use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, FrameCountPlugin, SystemInformationDiagnosticsPlugin};
use bevy::time::{Timer, TimerMode};
use bevy::ecs::entity::Entity;
use bevy::ecs::component::Component;
use bevy::prelude::Children;

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

#[derive(Component)]
pub struct GameEventsText;

// --- FPS Graph ---
#[derive(Resource)]
pub struct FpsGraphConfig {
    pub enabled: bool,
    pub min_fps: f32,
    pub target_fps: f32,
    pub max_samples: usize,
    pub bar_width_px: f32,
    pub height_px: f32,
    pub refresh_seconds: f32,
    pub background: Color,
}

impl Default for FpsGraphConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_fps: 5.0,
            target_fps: 120.0,
            max_samples: 120,
            bar_width_px: 3.0,
            height_px: 120.0,
            refresh_seconds: 0.1,
            background: Color::srgba(0.0, 0.0, 0.0, 0.6),
        }
    }
}

#[derive(Resource, Default)]
pub struct FpsHistory {
    pub samples: Vec<f32>,
}

#[derive(Resource)]
pub struct FpsGraphState {
    pub timer: Timer,
}

impl Default for FpsGraphState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct FpsGraphContainer;

#[derive(Component)]
pub struct FpsGraphBar;

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

/// Resource um GameEvents zu tracken
#[derive(Resource, Default)]
pub struct GameEventStats {
    pub hand_count_changed: u32,
    pub hand_gesture: u32,
    pub hand_pinch_distance: u32,
    pub object_detected: u32,
    pub last_event: Option<String>,
}

impl GameEventStats {
    pub fn total(&self) -> u32 {
        self.hand_count_changed + self.hand_gesture + self.hand_pinch_distance + self.object_detected
    }

    pub fn _reset(&mut self) {
        self.hand_count_changed = 0;
        self.hand_gesture = 0;
        self.hand_pinch_distance = 0;
        self.object_detected = 0;
    }
}

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

/// Toggle nur den FPS-Graphen (Digit5)
pub fn toggle_fps_graph(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<FpsGraphConfig>,
) {
    if keyboard.just_pressed(KeyCode::Digit5) {
        config.enabled = !config.enabled;
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

        // Game Events Text
        parent.spawn((
            Text::new("Events: --"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            GameEventsText,
        ));

        // FPS Graph Container + Bars
        parent.spawn((
            Node {
                margin: UiRect { top: Val::Px(16.0), ..default() },
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd, // Balken am unteren Rand ausrichten
                justify_content: JustifyContent::FlexStart,
                width: Val::Px(120.0 * 3.0), // placeholder, wird durch Bars gefüllt
                height: Val::Px(120.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            FpsGraphContainer,
        )).with_children(|graph| {
            let max_samples = 120usize;
            let bar_width = 3.0f32;
            for _ in 0..max_samples {
                graph.spawn((
                    Node {
                        width: Val::Px(bar_width),
                        height: Val::Px(0.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                    FpsGraphBar,
                ));
            }
        });
    });
}

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

pub fn update_state_text(
    current_state: Res<State<crate::setup::appstate::AppState>>,
    mut query: Query<&mut Text, With<StateText>>,
) {
    for mut text in &mut query {
        **text = format!("State: {:?}", current_state.get());
    }
}

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
        info.push_str(&format_light_count(env_count, env_query.iter().map(|(e, l, v)| (e, l as &dyn std::any::Any, v))));

        // DirectionalLight
        let dir_count = dir_query.iter().count();
        info.push_str(&format!("  Directional: {} ", dir_count));
        info.push_str(&format_light_count(dir_count, dir_query.iter().map(|(e, l, v)| (e, l as &dyn std::any::Any, v))));

        // PointLight
        let point_count = point_query.iter().count();
        info.push_str(&format!("  Point: {} ", point_count));
        info.push_str(&format_light_count(point_count, point_query.iter().map(|(e, l, v)| (e, l as &dyn std::any::Any, v))));

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

fn format_light_count<'a, I>(count: usize, iter: I) -> String
where
    I: Iterator<Item = (Entity, &'a dyn std::any::Any, Option<&'a Visibility>)>,
{
    if count > 0 {
        let active = iter.filter(|(_, _, vis)| {
            vis.map_or(true, |v| *v != Visibility::Hidden)
        }).count();
        format!("({} active)\n", active)
    } else {
        "\n".to_string()
    }
}

pub fn update_game_events_text(
    game_event_stats: Res<GameEventStats>,
    mut query: Query<&mut Text, With<GameEventsText>>,
) {
    for mut text in &mut query {
        if game_event_stats.total() > 0 {
            **text = format!("Events: HandCount: {} | Gesture: {} | Pinch: {} | Object: {}",
                            game_event_stats.hand_count_changed,
                            game_event_stats.hand_gesture,
                            game_event_stats.hand_pinch_distance,
                            game_event_stats.object_detected);
        } else {
            **text = "Events: --".to_string();
        }
    }
}

pub fn update_fps_graph(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut state: ResMut<FpsGraphState>,
    mut history: ResMut<FpsHistory>,
    config: Res<FpsGraphConfig>,
    mut bar_query: Query<&mut Node, With<FpsGraphBar>>,
    mut color_query: Query<&mut BackgroundColor, With<FpsGraphBar>>,
    container_query: Query<&Children, With<FpsGraphContainer>>,
) {
    if !config.enabled {
        if let Ok(children) = container_query.single() {
            for child in children.iter() {
                if let Ok(mut n) = bar_query.get_mut(child) {
                    n.height = Val::Px(0.0);
                }
            }
        }
        return;
    }

    state.timer.tick(time.delta());
    if !state.timer.just_finished() {
        return;
    }

    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps_diag.smoothed() {
            let fps = value as f32;
            if history.samples.len() >= config.max_samples {
                let overflow = history.samples.len() + 1 - config.max_samples;
                if overflow > 0 {
                    history.samples.drain(0..overflow);
                }
            }
            history.samples.push(fps);

            if let Ok(children) = container_query.single() {
                let needed = children.len();
                let mut values = vec![0.0f32; needed.saturating_sub(history.samples.len())];
                values.extend_from_slice(&history.samples);
                if values.len() > needed { values = values[values.len()-needed..].to_vec(); }

                for (i, child) in children.iter().enumerate() {
                    let v = values.get(i).copied().unwrap_or(0.0);
                    if let Ok(mut n) = bar_query.get_mut(child) {
                        let ratio = (v / config.target_fps).clamp(0.0, 1.0);
                        let height = ratio * config.height_px;
                        n.height = Val::Px(height);
                        n.width = Val::Px(config.bar_width_px);
                    }
                    if let Ok(mut c) = color_query.get_mut(child) {
                        let t = ((v - config.min_fps) / (config.target_fps - config.min_fps)).clamp(0.0, 1.0);
                        let r = 1.0 - t;
                        let g = t;
                        *c = BackgroundColor(Color::srgb(r, g, 0.0));
                    }
                }
            }
        }
    }
}
