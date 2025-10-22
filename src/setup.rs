pub mod world;
pub mod lighting;
pub mod camera;
pub mod assetloader;
pub mod appstate;
pub mod diagnostics;
pub mod stresstest;
pub mod gltf_spawner;
mod loading;

use bevy::prelude::*;
use appstate::AppState;

pub fn register_startup_systems(app: &mut App) {
    app.init_resource::<stresstest::StressTestConfig>();
    app.init_resource::<diagnostics::LowestFps>();
    app.init_resource::<diagnostics::AverageFps>();
    app.init_resource::<diagnostics::DiagnosticsOverlayVisible>();
    app.init_resource::<diagnostics::GameEventStats>();
    app.add_systems(Startup, (
        camera::spawn_static_orbit_camera,
        assetloader::load_assets_startup,
        lighting::spawn_directional_light,
        lighting::spawn_ambient_light,
        diagnostics::setup_fps_overlay,
        gltf_spawner::disable_physics_gizmos_on_startup,
    ).chain());
}

pub fn register_update_systems(app: &mut App) {
    app.add_systems(
        Update, (
        camera::orbit_camera_controls,
        diagnostics::update_fps_text,
        diagnostics::update_average_fps_text,
        diagnostics::update_lowest_fps_text,
        diagnostics::update_state_text,
        diagnostics::update_loading_progress,
        diagnostics::update_stress_test_info_text,
        diagnostics::update_light_info_text,
        diagnostics::update_game_events_text,
        diagnostics::toggle_diagnostics_overlay,
        stresstest::stress_test_input,
        stresstest::update_stress_test_info,
        gltf_spawner::toggle_physics_debug,
        world::spawn_ambience_when_ready,
        world::apply_radial_gravity.run_if(in_state(AppState::Running)),
        check_assets_loaded_transition.run_if(in_state(AppState::Loading)),
        stresstest::spawn_stress_test_objects
            .run_if(in_state(AppState::Running))
            .run_if(resource_exists::<assetloader::LoadedModels>)

    ));

    app.add_systems(
        OnEnter(AppState::Running),
        (
            loading::despawn_loading_screen,
            lighting::spawn_environment_map_light,
            world::spawn_initial_objects.run_if(resource_exists::<assetloader::LoadedModels>),
            setup_complete_log,
        ).chain()
    );
}

fn setup_complete_log() {
    info!("🚀 Setup complete - simulation starting!");
}

fn check_assets_loaded_transition(
    asset_handles: Option<Res<crate::setup::assetloader::AssetHandles>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(handles_res) = asset_handles {
        let handles = &handles_res.0;
        if handles.is_empty() {
            info!("No assets to load, transitioning to Running");
            next_state.set(AppState::Running);
            return;
        }

        let all_loaded = handles.iter().all(|handle| {
            matches!(
                asset_server.get_load_state(handle.id()),
                Some(bevy::asset::LoadState::Loaded)
            )
        });

        if all_loaded {
            info!("✅ All assets loaded successfully! Transitioning to Running state");
            next_state.set(AppState::Running);
        }
    } else {
        info!("No asset handles found, transitioning to Running");
        next_state.set(AppState::Running);
    }
}
