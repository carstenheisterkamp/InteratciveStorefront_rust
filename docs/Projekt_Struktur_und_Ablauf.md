# Projekt-Struktur und Ablauf

## Aktuelle Struktur des Projekts

Das Projekt ist ein interaktives Storefront-Spiel, entwickelt in **Rust** mit dem **Bevy**-Framework (einer Entity-Component-System-Engine für Spiele) und **Avian3D** für Physiksimulationen. Es basiert auf einem Cargo-Projekt (siehe `Cargo.toml` und `Cargo.lock`). Die Hauptstruktur im Workspace ist wie folgt:

- **Root-Verzeichnis**:
  - `Cargo.toml` und `Cargo.lock`: Konfiguration und Abhängigkeiten des Rust-Projekts.
  - `assets/`: Ressourcen wie Audio (Musik und Sounds), Konfiguration (`settings.json`), Schriftarten, Bilder und 3D-Modelle (z.B. `tasse.glb`, `test.glb`).
  - `src/`: Quellcode.
    - `main.rs`: Einstiegspunkt der Anwendung. Initialisiert die Bevy-App, fügt Plugins hinzu (z.B. DefaultPlugins, PhysicsPlugins, Diagnose-Plugins) und registriert Startup- und State-Systeme.
    - `setup/`: Modul für die Initialisierung.
      - `mod.rs`: Zentrales Modul, das andere Untermodule importiert (`world`, `lighting`, `camera`, `assetloader`, `gamestates`). Enthält Funktionen zum Registrieren von Startup-Systemen und State-Systemen.
      - `world.rs`, `lighting.rs`, `camera.rs`, `assetloader.rs`, `gamestates.rs`: Spezifische Module für das Spawnen der Welt, Lichter, Kamera, Laden von Assets und Definition der Spielzustände.
  - `target/`: Build-Artefakte (z.B. kompilierte Binaries wie `InteractiveStorefront`).

Das Projekt verwendet Bevy's ECS-Architektur: Entitäten (Entities) mit Komponenten (Components) und Systemen (Systems), die diese verarbeiten. Es gibt einen einfachen State-Machine für Spielzustände.

## Ablauf der Anwendung

Der Ablauf ist sequentiell und zustandsbasiert, gesteuert durch den `AppState`-Resource in `setup/mod.rs`. Hier ist eine Schritt-für-Schritt-Erklärung:

1. **Start der App** (`main.rs`):
   - Die `App` wird erstellt und mit Plugins ausgestattet (z.B. für Rendering, Physik, Diagnose).
   - `setup::register_startup_systems(&mut app)`: Registriert Startup-Systeme, die einmal beim Start ausgeführt werden:
     - `world::spawn_world`: Erstellt die Spielwelt (z.B. Szene-Elemente).
     - `lighting::spawn_directional_light`: Fügt gerichtetes Licht hinzu.
     - `camera::spawn_camera`: Platziert die Kamera.
     - `assetloader::load_assets_startup`: Lädt Assets (z.B. Modelle, Sounds) und speichert Handles in einem `AssetHandles`-Resource.
   - `setup::register_state_systems(&mut app)`: Registriert Update-Systeme für den State-Machine und fügt den initialen `AppState` ein (startet im `GameState::Loading`).

2. **Loading-Zustand** (`GameState::Loading`):
   - Ein Timer läuft 2 Sekunden (`loading_timer_system`).
   - Gleichzeitig prüft `check_assets_loaded_system`, ob alle Assets geladen sind (über `AssetServer`).
   - Sobald Timer abläuft oder Assets bereit sind, wechselt der State zu `GameState::Menu`.
   - Debug-Nachrichten werden in der Konsole ausgegeben (`state_debug_system`).

3. **Menu-Zustand** (`GameState::Menu`):
   - Wartet auf Benutzereingaben.
   - **Space-Taste**: Wechselt zu `GameState::InGame`.
   - **Escape-Taste**: Bleibt im Menu (oder wechselt zurück, falls nötig).

4. **InGame-Zustand** (`GameState::InGame`):
   - Das eigentliche Spiel läuft (z.B. Physik, Interaktionen).
   - **Space-Taste**: Wechselt zu `GameState::Paused`.
   - **Escape-Taste**: Wechselt zurück zu `GameState::Menu`.

5. **Paused-Zustand** (`GameState::Paused`):
   - Spiel ist pausiert.
   - **Space-Taste**: Wechselt zurück zu `GameState::InGame`.
   - **Escape-Taste**: Wechselt zu `GameState::Menu`.

Der State-Machine ist explizit implementiert (nicht Bevy's eingebaute States), um Flexibilität zu bieten. Eingaben werden über `ButtonInput<KeyCode>` verarbeitet. Das Spiel läuft in einer Schleife, bis es beendet wird.
