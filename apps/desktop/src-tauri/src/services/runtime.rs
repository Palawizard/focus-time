use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealth {
    pub product_name: &'static str,
    pub desktop_shell: &'static str,
    pub platform: String,
    pub persistence_mode: &'static str,
    pub workspace_crates: Vec<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeService;

impl RuntimeService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_runtime_health(&self) -> RuntimeHealth {
        let storage = focus_persistence::storage_profile();

        RuntimeHealth {
            product_name: "Focus Time",
            desktop_shell: "Tauri v2",
            platform: crate::platform::current_platform(),
            persistence_mode: storage.mode,
            workspace_crates: vec![
                focus_domain::crate_name(),
                focus_persistence::crate_name(),
                focus_stats::crate_name(),
                focus_tracking::crate_name(),
            ],
        }
    }
}
