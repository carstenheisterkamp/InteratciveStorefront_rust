mod setup;

use bevy::prelude::*;
use avian3d::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        bevy::diagnostic::LogDiagnosticsPlugin::default(),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
    ));

    setup::register_startup_systems(&mut app);
    setup::register_state_systems(&mut app);
    app.run();
}