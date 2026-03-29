use crate::services::RuntimeHealth;
use crate::state::AppState;

#[tauri::command]
pub fn get_runtime_health(state: tauri::State<'_, AppState>) -> RuntimeHealth {
    state.runtime.get_runtime_health()
}
