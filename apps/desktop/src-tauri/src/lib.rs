mod bootstrap;
mod commands;
mod platform;
mod services;
mod state;

use tauri::Manager;

use bootstrap::build_state;
use commands::{
    create_session, create_session_segment, create_tracking_exclusion_rule, delete_session,
    delete_tracking_exclusion_rule, export_history, get_history_session_detail, get_pomodoro_state,
    get_runtime_health, get_tracking_status, get_user_preferences, list_daily_stats,
    list_history_sessions, list_session_segments, list_sessions, list_tracked_apps,
    list_tracked_window_events, list_tracking_exclusion_rules, pause_pomodoro, replace_session,
    resume_pomodoro, save_daily_stat, save_user_preferences, seed_development_fixtures,
    skip_pomodoro_break, start_pomodoro, stop_pomodoro, upsert_tracked_app,
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
            list_history_sessions,
            get_history_session_detail,
            create_session,
            replace_session,
            delete_session,
            list_session_segments,
            create_session_segment,
            get_user_preferences,
            save_user_preferences,
            get_tracking_status,
            list_tracked_apps,
            upsert_tracked_app,
            list_tracked_window_events,
            list_tracking_exclusion_rules,
            create_tracking_exclusion_rule,
            delete_tracking_exclusion_rule,
            export_history,
            list_daily_stats,
            save_daily_stat,
            seed_development_fixtures
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focus Time desktop application");
}
