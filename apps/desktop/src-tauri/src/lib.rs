mod bootstrap;
mod commands;
mod platform;
mod services;
mod state;

use bootstrap::build_state;
use commands::get_runtime_health;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(build_state())
        .invoke_handler(tauri::generate_handler![get_runtime_health])
        .run(tauri::generate_context!())
        .expect("error while running Focus Time desktop application");
}
