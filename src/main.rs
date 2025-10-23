mod setup;
mod network;
mod gamelogic;

use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_hanabi::HanabiPlugin;
use crate::setup::appstate::AppState;
use crate::network::{OscReceiverPlugin, WebSocketReceiverPlugin};
use crate::gamelogic::GamelogicPlugin;


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
                    power_preference: bevy::render::settings::PowerPreference::HighPerformance,
                    ..default()
                }
            ),
            ..default()
        })
        .set(bevy::log::LogPlugin {
            filter: "wgpu=error,bevy_render=info,bevy_ecs=info,winit=error".to_string(),
            level: bevy::log::Level::INFO,
            ..default()
        })
    );
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());
    app.add_plugins(OscReceiverPlugin { listen_address: "0.0.0.0:9001".to_string(),});
    app.add_plugins(WebSocketReceiverPlugin { listen_address: "0.0.0.0:9002".to_string(),});
    app.add_plugins(GamelogicPlugin);
    app.add_plugins(HanabiPlugin);
    app.insert_resource(Gravity(Vec3::ZERO));
    app.insert_resource(ClearColor(Color::srgb(0.6, 0.6, 0.6)));
    app.init_state::<AppState>();
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
