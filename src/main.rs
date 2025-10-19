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
                resolution: (1920, 1080).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        })
        .set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings {
                    // HighPerformance für dedizierte GPU (wichtig für Windows-Laptops)
                    power_preference: bevy::render::settings::PowerPreference::HighPerformance,
                    ..default()
                }
            ),
            ..default()
        })
        .set(bevy::log::LogPlugin {
            // Filtere winit warnings auf macOS
            filter: "wgpu=error,bevy_render=info,bevy_ecs=info,winit=error".to_string(),
            level: bevy::log::Level::INFO,
            ..default()
        })
    );

    // Physik mit benutzerdefinierter Gravitation
    app.add_plugins(PhysicsPlugins::default());

    // Deaktiviere Standard-Gravitation und nutze Custom Gravity System
    app.insert_resource(Gravity(Vec3::ZERO));  // Standard-Gravitation aus

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