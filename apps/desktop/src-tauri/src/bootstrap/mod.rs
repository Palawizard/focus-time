use anyhow::Context;
use tauri::Manager;

use crate::services::{PomodoroService, RuntimeService, StorageService, TrackingService};
use crate::state::AppState;

pub fn build_state(app_handle: &tauri::AppHandle) -> anyhow::Result<AppState> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("failed to resolve the app data directory")?;
    let database_path = app_data_dir.join("focus-time.sqlite");
    let storage = tauri::async_runtime::block_on(StorageService::new(database_path))?;
    tauri::async_runtime::block_on(storage.ensure_ready())?;
    let tracker = TrackingService::new(storage.clone());
    let pomodoro = PomodoroService::new(app_handle.clone(), storage.clone(), tracker.clone());

    Ok(AppState::new(
        pomodoro,
        tracker,
        RuntimeService::new(),
        storage,
    ))
}
