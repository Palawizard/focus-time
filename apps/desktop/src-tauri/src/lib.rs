mod bootstrap;
mod commands;
mod platform;
mod services;
mod state;

use tauri::Manager;

use bootstrap::build_state;
use commands::{
    create_session, create_session_segment, get_pomodoro_state, get_runtime_health,
    get_user_preferences, list_daily_stats, list_session_segments, list_sessions,
    list_tracked_apps, pause_pomodoro, resume_pomodoro, save_daily_stat,
    save_user_preferences, seed_development_fixtures, skip_pomodoro_break, start_pomodoro,
    stop_pomodoro, upsert_tracked_app,
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
            get_pomodoro_state,
            start_pomodoro,
            pause_pomodoro,
            resume_pomodoro,
            stop_pomodoro,
            skip_pomodoro_break,
            list_sessions,
            create_session,
            list_session_segments,
            create_session_segment,
            get_user_preferences,
            save_user_preferences,
            list_tracked_apps,
            upsert_tracked_app,
            list_daily_stats,
            save_daily_stat,
            seed_development_fixtures
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focus Time desktop application");
}
