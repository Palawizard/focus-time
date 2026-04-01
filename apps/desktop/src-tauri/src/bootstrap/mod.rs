use anyhow::Context;
use tauri::Manager;

use crate::services::{
    DesktopBehavior, PomodoroService, RuntimeService, StorageService, TrackingService,
};
use crate::state::AppState;

pub fn build_state(app_handle: &tauri::AppHandle) -> anyhow::Result<AppState> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("failed to resolve the app data directory")?;
    let app_log_dir = app_handle
        .path()
        .app_log_dir()
        .context("failed to resolve the app log directory")?;
    let backup_dir = app_handle
        .path()
        .document_dir()
        .unwrap_or_else(|_| app_data_dir.clone())
        .join("focus-time-backups");
    let database_path = app_data_dir.join("focus-time.sqlite");
    let storage = tauri::async_runtime::block_on(StorageService::new(database_path))?;
    tauri::async_runtime::block_on(storage.ensure_ready())?;
    let preferences = tauri::async_runtime::block_on(storage.get_user_preferences())?;
    let tracker = TrackingService::new(storage.clone());
    let pomodoro = PomodoroService::new(app_handle.clone(), storage.clone(), tracker.clone());
    let runtime = RuntimeService::new(
        app_data_dir,
        app_log_dir,
        backup_dir,
        DesktopBehavior {
            tray_enabled: preferences.tray_enabled,
            close_to_tray: preferences.close_to_tray,
            launch_on_startup: preferences.launch_on_startup,
        },
    );

    Ok(AppState::new(pomodoro, tracker, runtime, storage))
}
