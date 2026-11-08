//! Tauri application shell. Wires the shared `jarvis-*` crates into a
//! desktop window and exposes a minimal set of commands to the frontend.

mod commands;
mod state;

use state::AppState;
use tracing::info;

pub fn run() {
    let config = jarvis_core::AppConfig::load().expect("failed to load configuration");
    jarvis_core::logging::init(&config.log_level);
    info!(?config.environment, "starting JARVIS desktop");

    tauri::Builder::default()
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![commands::send_message])
        .run(tauri::generate_context!())
        .expect("error while running JARVIS desktop application");
}
