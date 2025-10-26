# Rendering- und Kamera-Leitfaden (InteractiveStorefront)

Stand: 2025-10-26

Dieser Leitfaden fasst Ursachen und Lösungen zu Render-Performance (Fog), dem pinken Aufblitzen beim Start und Kamera-Strategien (eine vs. mehrere) zusammen, inklusive Beispiel-Code.

## TL;DR
- DistanceFog kostet Fill-Rate/Bandbreite (Fullscreen-Pass). In Kombination mit HDR, Bloom, DOF und Debug-Builds fällt die FPS stark ab. Nutze Release-Build, reduziere Post-Processing, senke Auflösung/MSAA, beschränke Fog-Range.
- Pinkes Aufblitzen: Surface wird vor dem ersten Camera-Clear präsentiert. Lösung: Kamera schon im Startup spawnen und `clear_color` explizit setzen.
- Perspektivwechsel: Best Practice ist eine einzelne Kamera und weiche Transform-Transitions. Mehrere Kameras nur, wenn wirklich mehrere Ansichten gleichzeitig oder stark unterschiedliche Render-Pipelines nötig sind.

---

## 1) DistanceFog zieht die Framerate runter – warum und was tun?

### Ursache
- `DistanceFog` wird als (mindestens) ein weiterer Fullscreen-Pass ausgeführt, der den Depth-Buffer liest und per Pixel mischt.
- Das ist fill-rate- und bandbreitenlastig. Je höher die Auflösung, desto teurer.
- In Kombination mit weiteren Post-Processing-Pässen (HDR/Tonemapping, Bloom, Depth of Field) summieren sich die Kosten.
- Auf macOS/Metal und integrierten GPUs ist der Effekt stärker sichtbar. Debug-Builds sind generell deutlich langsamer als Release.

### Maßnahmen
- In Release starten:
  ```sh
  cargo run --release
  ```
- Post-Processing testweise einzeln deaktivieren und Fog isoliert messen (Bloom/DOF sind besonders teuer).
- Render-Auflösung senken bzw. Resolution-Scale nutzen; MSAA-Level reduzieren.
- Fog-Parameter enger fassen (kleinerer Wirkbereich, z. B. `start`/`end` näher zusammen), um sichtbare Fläche und Overdraw zu verringern.
- HDR nur aktivieren, wenn es für Bloom/DOF wirklich benötigt wird.

---

## 2) Pinkes Aufblitzen beim Start

### Ursache
- Das erste präsentierte Bild der Surface wurde noch nicht von einer Kamera gecleart; WGPU/Bevy zeigt dann ein Debug-Pink (uninitialisierter Framebuffer) in Frame 0.

### Lösung
- Kamera im Startup sofort spawnen und `clear_color` explizit setzen. Beispiel:

```rust
use bevy::prelude::*;

pub struct CameraClearPlugin;

impl Plugin for CameraClearPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera_immediately);
    }
}

fn spawn_camera_immediately(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.6, 0.6, 0.6)),
                ..default()
            },
            ..default()
        },
    ));
}
```

- Stelle sicher, dass keine andere Kamera später ohne `clear_color`-Override spawnt und die primäre Kamera in Frame 0 existiert.

---

## 3) Perspektivwechsel: eine Kamera vs. mehrere Kameras

### Best Practice
- Für das Umschalten der Perspektive innerhalb derselben Ansicht: eine einzige Kamera beibehalten und bei Input die `Transform` (Position/Rotation) sanft animieren. Das ist am einfachsten, robust und am günstigsten.

### Mehrere Kameras – wann sinnvoll?
- Wenn du gleichzeitig mehrere Ansichten rendern musst (Splitscreen, Minimap, zweites Fenster/Display).
- Wenn stark unterschiedliche Render-Settings/Layer/Post-Processing pro Ansicht nötig sind.

### Kostenfallen und Tipps
- Wenn mehrere Kameras in dasselbe Fenster rendern, unbedingt nur eine `is_active = true` lassen, die restlichen `false`, damit nicht mehrere Fullscreen-Pässe (Tonemapping, Fog, Bloom etc.) pro Frame ausgeführt werden.
- Für Dual-Window/Multimonitor kann jede Kamera ein eigenes Fenster (`RenderTarget::Window`) besitzen.

---

## 4) Beispiel A: Multi-Camera-Plugin (Single-Window switch oder Dual-Window)

Dieses Beispiel zeigt zwei Kameras. Im Single-Window-Modus schaltest du per Taste `C` zwischen ihnen um. Im Dual-Window-Modus rendert die zweite Kamera in ein zweites Fenster.

```rust
use bevy::prelude::*;
use bevy::window::WindowRef;
use bevy::render::camera::RenderTarget;

pub struct MultiCameraPlugin { pub dual_window: bool }
impl MultiCameraPlugin { pub fn single_window() -> Self { Self { dual_window: false } } pub fn dual_window() -> Self { Self { dual_window: true } } }

#[derive(Resource)] struct CameraSet { main: Entity, alt: Entity, dual_window: bool }

impl Plugin for MultiCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSet { main: Entity::from_raw(0), alt: Entity::from_raw(0), dual_window: self.dual_window });
        app.add_systems(Startup, spawn_cameras);
        if !self.dual_window { app.add_systems(Update, switch_active_camera); }
    }
}

fn spawn_cameras(mut commands: Commands, mut cam_set: ResMut<CameraSet>) {
    // Hauptkamera im Primary Window
    let main = commands
        .spawn((
            Name::new("MainCamera"),
            Camera3dBundle {
                camera: Camera {
                    clear_color: ClearColorConfig::Custom(Color::srgb(0.6, 0.6, 0.6)),
                    target: RenderTarget::Window(WindowRef::Primary),
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        ))
        .id();

    // Alternative Kamera
    let alt = if cam_set.dual_window {
        // Zweites Fenster
        let second_win = commands.spawn(Window { title: "Alt View".into(), ..default() }).id();
        commands
            .spawn((
                Name::new("AltCamera"),
                Camera3dBundle {
                    camera: Camera {
                        clear_color: ClearColorConfig::Custom(Color::srgb(0.6, 0.6, 0.6)),
                        target: RenderTarget::Window(WindowRef::Entity(second_win)),
                        ..default()
                    },
                    transform: Transform::from_xyz(-8.0, 6.0, -12.0).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                },
            ))
            .id()
    } else {
        // Gleiche Surface; zunächst inaktiv
        commands
            .spawn((
                Name::new("AltCamera"),
                Camera3dBundle {
                    camera: Camera {
                        is_active: false,
                        clear_color: ClearColorConfig::Custom(Color::srgb(0.6, 0.6, 0.6)),
                        target: RenderTarget::Window(WindowRef::Primary),
                        ..default()
                    },
                    transform: Transform::from_xyz(-8.0, 6.0, -12.0).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                },
            ))
            .id()
    };

    cam_set.main = main;
    cam_set.alt = alt;
}

fn switch_active_camera(keys: Res<ButtonInput<KeyCode>>, cam_set: Res<CameraSet>, mut q_cam: Query<&mut Camera>) {
    if !keys.just_pressed(KeyCode::KeyC) || cam_set.dual_window { return; }
    let mut cam_main = q_cam.get_mut(cam_set.main).ok();
    let mut cam_alt = q_cam.get_mut(cam_set.alt).ok();
    if let (Some(mut a), Some(mut b)) = (cam_main.as_mut(), cam_alt.as_mut()) {
        let next_main_active = !a.is_active;
        a.is_active = next_main_active;
        b.is_active = !next_main_active;
    }
}
```

Integration:
- Entferne andere Kamera-Spawner, um Doppel-Render zu vermeiden.
- Plugin registrieren: `app.add_plugins(MultiCameraPlugin::single_window());` oder `dual_window()`.

---

## 5) Beispiel B: Perspektivwechsel mit einer Kamera (weiche Transition)

Ein Plugin, das vordefinierte Viewpoints per Klick/Taste durchschaltet und die Kamera weich interpoliert.

```rust
use bevy::prelude::*;

pub struct CameraSwitchPlugin;
impl Plugin for CameraSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Viewpoints>()
            .init_resource::<Transition>()
            .add_systems(Update, (cycle_view_on_click, animate_camera_transition));
    }
}

#[derive(Clone)] struct Viewpoint { pos: Vec3, look_at: Vec3 }

#[derive(Resource)] struct Viewpoints { list: Vec<Viewpoint>, idx: usize }
impl Default for Viewpoints { fn default() -> Self { Self { list: vec![ Viewpoint { pos: Vec3::new(8.0, 6.0, 12.0), look_at: Vec3::ZERO }, Viewpoint { pos: Vec3::new(-8.0, 10.0, -12.0), look_at: Vec3::ZERO }, Viewpoint { pos: Vec3::new(0.0, 20.0, 0.01), look_at: Vec3::ZERO }, ], idx: 0 } } }

#[derive(Resource)] struct Transition { active: bool, t: f32, duration: f32, start_pos: Vec3, start_rot: Quat, target_pos: Vec3, target_rot: Quat }
impl Default for Transition { fn default() -> Self { Self { active: false, t: 0.0, duration: 0.5, start_pos: Vec3::ZERO, start_rot: Quat::IDENTITY, target_pos: Vec3::ZERO, target_rot: Quat::IDENTITY } } }

fn cycle_view_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut vp: ResMut<Viewpoints>,
    mut tr: ResMut<Transition>,
    mut q_cam: Query<(&mut Transform, &Camera), With<Camera3d>>,
) {
    if tr.active { return; }
    if !(mouse.just_pressed(MouseButton::Left) || keys.just_pressed(KeyCode::KeyC)) { return; }

    // aktive 3D-Kamera finden
    let mut cam_transform_opt: Option<Mut<Transform>> = None;
    for (transform, cam) in q_cam.iter_mut() { if cam.is_active { cam_transform_opt = Some(transform); break; } }
    let mut cam_transform = if let Some(t) = cam_transform_opt { t } else if let Some((t, _)) = q_cam.iter_mut().next() { t } else { return; };

    // nächsten Viewpoint
    vp.idx = (vp.idx + 1) % vp.list.len();
    let vp_next = &vp.list[vp.idx];

    let start_pos = cam_transform.translation;
    let start_rot = cam_transform.rotation;
    let target_pos = vp_next.pos;
    let target_rot = Transform::from_translation(vp_next.pos).looking_at(vp_next.look_at, Vec3::Y).rotation;

    tr.active = true; tr.t = 0.0; tr.start_pos = start_pos; tr.start_rot = start_rot; tr.target_pos = target_pos; tr.target_rot = target_rot;
}

fn animate_camera_transition(time: Res<Time>, mut tr: ResMut<Transition>, mut q_cam: Query<(&mut Transform, &Camera), With<Camera3d>>) {
    if !tr.active { return; }
    tr.t = (tr.t + time.delta_seconds() / tr.duration).clamp(0.0, 1.0);
    for (mut transform, cam) in q_cam.iter_mut() {
        if !cam.is_active { continue; }
        transform.translation = tr.start_pos.lerp(tr.target_pos, tr.t);
        transform.rotation = tr.start_rot.slerp(tr.target_rot, tr.t);
        break;
    }
    if tr.t >= 1.0 { tr.active = false; }
}
```

Tipps:
- Falls du bereits eine Orbit-/Freelook-Steuerung hast, pausiere deren Input während `Transition.active`, damit sie die Interpolation nicht "überfährt".
- Viewpoints können aus Daten (z. B. JSON) geladen oder im Editor gesetzt werden.

---

## 6) Quick Profiling & Troubleshooting
- Immer erst mit `cargo run --release` messen; Debug verfälscht Post-Processing-Kosten massiv.
- Schrittweise Features ein-/ausschalten (Fog, Bloom, DOF), um Hotspots zu isolieren.
- Auflösung/Resolution-Scale ist der stärkste Hebel, wenn Fullscreen-Pässe dominieren.
- Auf macOS: Achte auf die integrierte GPU in MacBooks. Externe Displays/High-DPI erhöhen die Last deutlich.

---

## Projektintegration (Hinweise)
- In `src/setup.rs` wird bereits `orbiting_camera::spawn_dynamic_orbit_camera` im `Startup` registriert. Stelle sicher, dass mindestens eine Kamera in Frame 0 existiert und ein definiertes `clear_color` besitzt, um das pinke Frame zu vermeiden.
- Nutze entweder das Multi-Camera- oder das Single-Camera-Transitions-Beispiel – nicht beides gleichzeitig. Entferne doppelte Kamera-Spawns.


