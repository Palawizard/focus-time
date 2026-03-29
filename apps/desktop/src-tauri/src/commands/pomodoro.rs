use focus_domain::{PomodoroPreset, PomodoroSnapshot};
use serde::Deserialize;

use crate::{
    services::StartPomodoroInput,
    state::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPomodoroRequest {
    pub label: String,
    pub focus_minutes: i32,
    pub short_break_minutes: i32,
    pub long_break_minutes: i32,
    pub sessions_until_long_break: i32,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
}

#[tauri::command]
pub async fn get_pomodoro_state(
    state: tauri::State<'_, AppState>,
) -> Result<PomodoroSnapshot, String> {
    Ok(state.pomodoro.get_state().await)
}

#[tauri::command]
pub async fn start_pomodoro(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: StartPomodoroRequest,
) -> Result<PomodoroSnapshot, String> {
    state
        .pomodoro
        .start(
            &app_handle,
            StartPomodoroInput {
                preset: PomodoroPreset {
                    label: request.label,
                    focus_minutes: request.focus_minutes,
                    short_break_minutes: request.short_break_minutes,
                    long_break_minutes: request.long_break_minutes,
                    sessions_until_long_break: request.sessions_until_long_break,
                },
                auto_start_breaks: request.auto_start_breaks,
                auto_start_focus: request.auto_start_focus,
            },
        )
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn pause_pomodoro(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<PomodoroSnapshot, String> {
    state
        .pomodoro
        .pause(&app_handle)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn resume_pomodoro(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<PomodoroSnapshot, String> {
    state
        .pomodoro
        .resume(&app_handle)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stop_pomodoro(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<PomodoroSnapshot, String> {
    state
        .pomodoro
        .stop(&app_handle)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skip_pomodoro_break(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<PomodoroSnapshot, String> {
    state
        .pomodoro
        .skip_break(&app_handle)
        .await
        .map_err(|error| error.to_string())
}
