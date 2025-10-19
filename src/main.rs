// main.rs

mod setup;
use bevy::prelude::*;
use avian3d::prelude::*;
use crate::setup::appstate::AppState;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Interactive Storefront".to_string(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                mode: bevy::window::WindowMode::Windowed,
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        })
        .set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings {
                    power_preference: bevy::render::settings::PowerPreference::HighPerformance,
                    ..default()
                }
            ),
            ..default()
        })
    );

    app.add_plugins(PhysicsPlugins::default());

    // Initialize state system
    app.init_state::<AppState>();

    // Physics läuft nur im Running state
    app.configure_sets(
        Update,
        PhysicsSystems::StepSimulation.run_if(in_state(AppState::Running))
    );

    // Diagnostics Plugins
    app.add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
        bevy::diagnostic::SystemInformationDiagnosticsPlugin::default(),
    ));

    setup::register_startup_systems(&mut app);
    setup::register_update_systems(&mut app);
    app.run();
}