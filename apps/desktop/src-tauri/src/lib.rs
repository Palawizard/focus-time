mod bootstrap;
mod commands;
mod platform;
mod services;
mod state;

use tauri::Manager;

use bootstrap::build_state;
use commands::{
    create_session, create_session_segment, get_runtime_health, get_user_preferences,
    list_daily_stats, list_session_segments, list_sessions, list_tracked_apps, save_daily_stat,
    save_user_preferences, upsert_tracked_app,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = build_state(app.handle())?;
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_runtime_health,
            list_sessions,
            create_session,
            list_session_segments,
            create_session_segment,
            get_user_preferences,
            save_user_preferences,
            list_tracked_apps,
            upsert_tracked_app,
            list_daily_stats,
            save_daily_stat
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focus Time desktop application");
}
