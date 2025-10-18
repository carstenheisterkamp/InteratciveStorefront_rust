pub mod world;
pub mod lighting;
pub mod camera;
pub mod assetloader;
pub mod gamestates;

use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::setup::gamestates::GameState;

pub fn register_startup_systems(app: &mut App) {
    app.add_systems(Startup, (
        world::spawn_world,
        lighting::spawn_directional_light,
        camera::spawn_camera,
        assetloader::load_assets_startup,
    ));
}

/// Simple, explicit state-machine example implemented as a Resource.
/// - `register_state_systems` installs the resource and the demo systems.
pub fn register_state_systems(app: &mut App) {
    // initial state: Loading with a short timer
    app.insert_resource(AppState::new(GameState::Loading));

    app.add_systems(Update, (
        loading_timer_system,
        input_transition_system,
        state_debug_system,
        check_assets_loaded_system,
    ));
}

#[derive(Resource)]
pub struct AppState {
    pub state: GameState,
    loading_timer: Timer,
    last_reported: Option<GameState>,
}

impl AppState {
    pub fn new(initial: GameState) -> Self {
        AppState {
            state: initial,
            loading_timer: Timer::from_seconds(2.0, TimerMode::Once),
            last_reported: None,
        }
    }
}

fn loading_timer_system(time: Res<Time>, mut app_state: ResMut<AppState>) {
    if app_state.state == GameState::Loading {
        app_state.loading_timer.tick(time.delta());
        if app_state.loading_timer.is_finished() {
            app_state.state = GameState::Menu;
            // reset timer in case we re-enter Loading later
            app_state.loading_timer = Timer::from_seconds(2.0, TimerMode::Once);
        }
    }
}

fn input_transition_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<AppState>,
) {
    // Space : toggle between Menu and InGame
    if keyboard.just_pressed(KeyCode::Space) {
        match app_state.state {
            GameState::Menu => app_state.state = GameState::InGame,
            GameState::InGame => app_state.state = GameState::Paused,
            GameState::Paused => app_state.state = GameState::InGame,
            _ => {}
        }
    }

    // Esc : go to Menu from any state
    if keyboard.just_pressed(KeyCode::Escape) {
        app_state.state = GameState::Menu;
    }
}

fn state_debug_system(mut app_state: ResMut<AppState>) {
    if app_state.last_reported != Some(app_state.state) {
        info!("AppState changed: {:?}", app_state.state);
        app_state.last_reported = Some(app_state.state);
    }
}

// Check if all assets from AssetHandles are loaded. If yes, transition to Menu.
fn check_assets_loaded_system(
    asset_handles: Option<Res<crate::setup::assetloader::AssetHandles>>,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<AppState>,
) {
    // only act while in Loading
    if app_state.state != GameState::Loading {
        return;
    }

    if let Some(handles_res) = asset_handles {
        let handles = &handles_res.0;
        if handles.is_empty() {
            // nothing to load -> go to Menu
            app_state.state = GameState::Menu;
            return;
        }
        // Check if all assets are loaded
        let all_loaded = handles.iter().all(|handle| {
            matches!(
                asset_server.get_load_state(handle.id()),
                Some(bevy::asset::LoadState::Loaded)
            )
        });
        if all_loaded {
            app_state.state = GameState::Menu;
        }
    } else {
        // No resource present; treat as no assets
        app_state.state = GameState::Menu;
    }
}
