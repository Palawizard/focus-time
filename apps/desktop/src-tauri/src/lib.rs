mod bootstrap;
mod commands;
mod platform;
mod services;
mod state;

use anyhow::Context;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use bootstrap::build_state;
use commands::{
    create_local_backup, create_session, create_session_segment, create_tracking_exclusion_rule,
    delete_session, delete_tracking_exclusion_rule, export_history, get_gamification_overview,
    get_history_session_detail, get_pomodoro_state, get_runtime_health, get_stats_dashboard,
    get_tracking_status, get_user_preferences, list_daily_stats, list_history_sessions,
    list_local_backups, list_session_segments, list_sessions, list_tracked_apps,
    list_tracked_window_events, list_tracking_exclusion_rules, pause_pomodoro, replace_session,
    restore_local_backup, resume_pomodoro, save_daily_stat, save_user_preferences,
    seed_development_fixtures, skip_pomodoro_break, start_pomodoro, stop_pomodoro,
    upsert_tracked_app,
};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .rotation_strategy(RotationStrategy::KeepSome(3))
                .targets([Target::new(TargetKind::LogDir {
                    file_name: Some("focus-time".into()),
                })])
                .build(),
        )
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("Focus Time")
                .build(),
        )
        .setup(|app| {
            let state = build_state(app.handle())?;
            let preferences = tauri::async_runtime::block_on(state.storage.get_user_preferences())?;

            setup_tray(app.handle())?;
            state
                .runtime
                .apply_user_preferences(app.handle(), &preferences)?;
            state.runtime.sync_window_visibility(app.handle())?;
            app.manage(state);

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                let app_handle = window.app_handle();
                let state = app_handle.state::<AppState>();
                let behavior = state.runtime.desktop_behavior();

                if behavior.tray_enabled && behavior.close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
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
            create_local_backup,
            list_local_backups,
            restore_local_backup,
            get_tracking_status,
            list_tracked_apps,
            upsert_tracked_app,
            list_tracked_window_events,
            list_tracking_exclusion_rules,
            create_tracking_exclusion_rule,
            delete_tracking_exclusion_rule,
            export_history,
            get_stats_dashboard,
            get_gamification_overview,
            list_daily_stats,
            save_daily_stat,
            seed_development_fixtures
        ])
        .run(tauri::generate_context!())
        .expect("error while running Focus Time desktop application");
}

fn setup_tray(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    if app_handle.tray_by_id("main").is_some() {
        return Ok(());
    }

    let show_item = MenuItemBuilder::with_id("tray_show", "Show Focus Time").build(app_handle)?;
    let hide_item = MenuItemBuilder::with_id("tray_hide", "Hide window").build(app_handle)?;
    let quit_item = MenuItemBuilder::with_id("tray_quit", "Quit").build(app_handle)?;
    let menu = MenuBuilder::new(app_handle)
        .item(&show_item)
        .item(&hide_item)
        .separator()
        .item(&quit_item)
        .build()?;
    let icon = app_handle
        .default_window_icon()
        .cloned()
        .context("default window icon should be available for tray setup")?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Focus Time")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray_show" => {
                let _ = show_main_window(app);
            }
            "tray_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "tray_quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .build(app_handle)?;

    Ok(())
}

fn show_main_window(app_handle: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}
