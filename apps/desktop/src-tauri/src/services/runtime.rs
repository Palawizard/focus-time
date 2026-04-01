use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;

use focus_domain::UserPreference;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBehavior {
    pub tray_enabled: bool,
    pub close_to_tray: bool,
    pub launch_on_startup: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealth {
    pub product_name: &'static str,
    pub app_version: String,
    pub desktop_shell: &'static str,
    pub platform: String,
    pub persistence_mode: &'static str,
    pub workspace_crates: Vec<&'static str>,
    pub app_data_dir: String,
    pub app_log_dir: String,
    pub backup_dir: String,
    pub launch_on_startup_enabled: bool,
    pub tray_enabled: bool,
    pub close_to_tray: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeService {
    app_data_dir: PathBuf,
    app_log_dir: PathBuf,
    backup_dir: PathBuf,
    behavior: Arc<RwLock<DesktopBehavior>>,
}

impl RuntimeService {
    pub fn new(
        app_data_dir: PathBuf,
        app_log_dir: PathBuf,
        backup_dir: PathBuf,
        behavior: DesktopBehavior,
    ) -> Self {
        Self {
            app_data_dir,
            app_log_dir,
            backup_dir,
            behavior: Arc::new(RwLock::new(behavior)),
        }
    }

    pub fn get_runtime_health<R: Runtime>(&self, app_handle: &AppHandle<R>) -> RuntimeHealth {
        let storage = focus_persistence::storage_profile();
        let behavior = self.desktop_behavior();
        let launch_on_startup_enabled = app_handle.autolaunch().is_enabled().unwrap_or(false);

        RuntimeHealth {
            product_name: "Focus Time",
            app_version: app_handle.package_info().version.to_string(),
            desktop_shell: "Tauri v2",
            platform: crate::platform::current_platform(),
            persistence_mode: storage.mode,
            workspace_crates: vec![
                focus_domain::crate_name(),
                focus_persistence::crate_name(),
                focus_stats::crate_name(),
                focus_tracking::crate_name(),
            ],
            app_data_dir: self.app_data_dir.display().to_string(),
            app_log_dir: self.app_log_dir.display().to_string(),
            backup_dir: self.backup_dir.display().to_string(),
            launch_on_startup_enabled,
            tray_enabled: behavior.tray_enabled,
            close_to_tray: behavior.close_to_tray,
        }
    }

    pub fn desktop_behavior(&self) -> DesktopBehavior {
        *self
            .behavior
            .read()
            .expect("desktop behavior lock should not be poisoned")
    }

    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    pub fn apply_user_preferences<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        preferences: &UserPreference,
    ) -> anyhow::Result<()> {
        if preferences.launch_on_startup {
            app_handle.autolaunch().enable()?;
        } else {
            app_handle.autolaunch().disable()?;
        }

        if let Some(tray) = app_handle.tray_by_id("main") {
            tray.set_visible(preferences.tray_enabled)?;
        }

        self.update_desktop_behavior(DesktopBehavior {
            tray_enabled: preferences.tray_enabled,
            close_to_tray: preferences.close_to_tray,
            launch_on_startup: preferences.launch_on_startup,
        });

        Ok(())
    }

    pub fn sync_window_visibility<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
    ) -> anyhow::Result<()> {
        let behavior = self.desktop_behavior();

        if let Some(tray) = app_handle.tray_by_id("main") {
            tray.set_visible(behavior.tray_enabled)?;
        }

        Ok(())
    }

    fn update_desktop_behavior(&self, behavior: DesktopBehavior) {
        if let Ok(mut current) = self.behavior.write() {
            *current = behavior;
        }
    }
}
