use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use focus_domain::TrackingCategory;
use serde::{Deserialize, Serialize};

pub const fn crate_name() -> &'static str {
    "focus-tracking"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackingMode {
    WindowsNative,
    LinuxHyprland,
    LinuxSway,
    LinuxX11,
    LinuxWayland,
    Unsupported,
}

impl TrackingMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WindowsNative => "windows_native",
            Self::LinuxHyprland => "linux_hyprland",
            Self::LinuxSway => "linux_sway",
            Self::LinuxX11 => "linux_x11",
            Self::LinuxWayland => "linux_wayland",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackingCapability {
    Supported,
    Limited,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrackingStatus {
    pub mode: TrackingMode,
    pub capability: TrackingCapability,
    pub message: String,
    pub dependency_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindowSample {
    pub app_name: String,
    pub executable: String,
    pub category: TrackingCategory,
    pub window_title: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    #[error("tracking is not available on this platform")]
    UnsupportedPlatform,
    #[error("tracking is limited on Wayland without an X11 fallback")]
    WaylandUnsupported,
    #[error("missing compositor IPC command: {0}")]
    MissingCompositorCommand(&'static str),
    #[error("missing system dependency: {0}")]
    MissingDependency(&'static str),
    #[error("failed to read active window information: {0}")]
    CommandFailed(String),
    #[error("failed to parse active window information")]
    InvalidOutput,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn adapter_mode() -> &'static str {
    tracking_status().mode.as_str()
}

pub fn tracking_status() -> TrackingStatus {
    #[cfg(target_os = "windows")]
    {
        TrackingStatus {
            mode: TrackingMode::WindowsNative,
            capability: TrackingCapability::Supported,
            message: "Native active-window tracking is available.".to_string(),
            dependency_hint: None,
        }
    }

    #[cfg(target_os = "linux")]
    {
        if is_hyprland_session() {
            let dependency_hint = (!command_exists("hyprctl")).then(|| {
                "Install hyprland and make sure hyprctl is available in PATH.".to_string()
            });

            return TrackingStatus {
                mode: TrackingMode::LinuxHyprland,
                capability: if dependency_hint.is_some() {
                    TrackingCapability::Limited
                } else {
                    TrackingCapability::Supported
                },
                message: if dependency_hint.is_some() {
                    "Hyprland was detected, but its IPC command is missing.".to_string()
                } else {
                    "Hyprland IPC tracking is available for native Wayland windows.".to_string()
                },
                dependency_hint,
            };
        }

        if is_sway_session() {
            let dependency_hint = (!command_exists("swaymsg"))
                .then(|| "Install swaymsg to enable Sway IPC tracking.".to_string());

            return TrackingStatus {
                mode: TrackingMode::LinuxSway,
                capability: if dependency_hint.is_some() {
                    TrackingCapability::Limited
                } else {
                    TrackingCapability::Supported
                },
                message: if dependency_hint.is_some() {
                    "Sway was detected, but its IPC command is missing.".to_string()
                } else {
                    "Sway IPC tracking is available for native Wayland windows.".to_string()
                },
                dependency_hint,
            };
        }

        if env::var_os("DISPLAY").is_some() {
            let dependency_hint = (!command_exists("xdotool"))
                .then(|| "Install xdotool to enable X11 tracking.".to_string());

            return TrackingStatus {
                mode: TrackingMode::LinuxX11,
                capability: if dependency_hint.is_some() {
                    TrackingCapability::Limited
                } else {
                    TrackingCapability::Supported
                },
                message: if dependency_hint.is_some() {
                    "X11 tracking is detected, but one dependency is still missing.".to_string()
                } else {
                    "X11 tracking is available for the current desktop session.".to_string()
                },
                dependency_hint,
            };
        }

        if env::var_os("WAYLAND_DISPLAY").is_some() {
            return TrackingStatus {
                mode: TrackingMode::LinuxWayland,
                capability: TrackingCapability::Limited,
                message: "This Wayland session does not expose a compositor-specific active-window adapter yet."
                    .to_string(),
                dependency_hint: Some(
                    "Tracking is reliable on Hyprland, Sway and X11. Other compositors still need dedicated integration."
                        .to_string(),
                ),
            };
        }

        TrackingStatus {
            mode: TrackingMode::Unsupported,
            capability: TrackingCapability::Unsupported,
            message: "No supported desktop session was detected.".to_string(),
            dependency_hint: None,
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        TrackingStatus {
            mode: TrackingMode::Unsupported,
            capability: TrackingCapability::Unsupported,
            message: "This platform is not supported by the tracker yet.".to_string(),
            dependency_hint: None,
        }
    }
}

pub fn capture_active_window() -> Result<Option<ActiveWindowSample>, TrackingError> {
    #[cfg(target_os = "windows")]
    {
        windows::capture_active_window()
    }

    #[cfg(target_os = "linux")]
    {
        linux::capture_active_window()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(TrackingError::UnsupportedPlatform)
    }
}

pub fn normalize_executable_name(value: &str) -> String {
    let trimmed = value.trim();
    let file_name = Path::new(trimmed)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(trimmed);
    let lowered = file_name.to_ascii_lowercase();

    lowered
        .trim_end_matches(".exe")
        .trim_end_matches(".bin")
        .trim_end_matches(".app")
        .trim()
        .to_string()
}

pub fn normalize_app_name(value: &str) -> String {
    let normalized = normalize_executable_name(value);

    normalized
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut characters = segment.chars();

            match characters.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string()
                        + &characters.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn categorize_app(executable: &str, window_title: Option<&str>) -> TrackingCategory {
    let haystack = format!(
        "{} {}",
        normalize_executable_name(executable),
        window_title.unwrap_or_default().to_ascii_lowercase()
    );

    if contains_any(
        &haystack,
        &["code", "cursor", "zed", "nvim", "vim", "idea", "studio"],
    ) {
        return TrackingCategory::Development;
    }

    if contains_any(&haystack, &["zoom", "meet", "webex"]) {
        return TrackingCategory::Meeting;
    }

    if contains_any(
        &haystack,
        &["slack", "discord", "teams", "telegram", "signal"],
    ) {
        return TrackingCategory::Communication;
    }

    if contains_any(
        &haystack,
        &["chrome", "firefox", "browser", "arc", "brave", "safari"],
    ) {
        return TrackingCategory::Browser;
    }

    if contains_any(
        &haystack,
        &["word", "writer", "notion", "obsidian", "notes"],
    ) {
        return TrackingCategory::Writing;
    }

    if contains_any(
        &haystack,
        &["figma", "sketch", "illustrator", "photoshop", "inkscape"],
    ) {
        return TrackingCategory::Design;
    }

    if contains_any(&haystack, &["paper", "research", "docs", "wikipedia"]) {
        return TrackingCategory::Research;
    }

    if contains_any(&haystack, &["terminal", "console", "system", "settings"]) {
        return TrackingCategory::Utilities;
    }

    TrackingCategory::Unknown
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_hyprland_session() -> bool {
    env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        || env::var("XDG_CURRENT_DESKTOP")
            .map(|value| value.to_ascii_lowercase().contains("hypr"))
            .unwrap_or(false)
}

fn is_sway_session() -> bool {
    env::var_os("SWAYSOCK").is_some()
        || env::var("XDG_CURRENT_DESKTOP")
            .map(|value| value.to_ascii_lowercase().contains("sway"))
            .unwrap_or(false)
}

fn command_exists(binary: &str) -> bool {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|directory| directory.join(binary))
        .any(|candidate| candidate.exists())
}

fn choose_executable_hint(
    process_executable: Option<String>,
    compositor_class: Option<&str>,
) -> String {
    let compositor_class = compositor_class
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    match process_executable {
        Some(process_executable) => {
            let normalized = normalize_executable_name(&process_executable);

            if matches!(
                normalized.as_str(),
                "bwrap" | "flatpak" | "electron" | "python" | "java" | "steamwebhelper"
            ) {
                compositor_class.unwrap_or(process_executable)
            } else {
                process_executable
            }
        }
        None => compositor_class.unwrap_or_else(|| "unknown".to_string()),
    }
}

fn read_executable_from_pid(pid: u32) -> Result<String, TrackingError> {
    let proc_path = PathBuf::from(format!("/proc/{pid}/comm"));

    if proc_path.exists() {
        return Ok(fs::read_to_string(proc_path)?.trim().to_string());
    }

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()?;

    if !output.status.success() {
        return Err(TrackingError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{
        categorize_app, choose_executable_hint, command_exists, is_hyprland_session,
        is_sway_session, normalize_app_name, read_executable_from_pid, ActiveWindowSample,
        TrackingError,
    };
    use serde::Deserialize;
    use std::process::Command;

    pub fn capture_active_window() -> Result<Option<ActiveWindowSample>, TrackingError> {
        if is_hyprland_session() {
            return hyprland::capture_active_window();
        }

        if is_sway_session() {
            return sway::capture_active_window();
        }

        if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return Err(TrackingError::WaylandUnsupported);
        }

        if !command_exists("xdotool") {
            return Err(TrackingError::MissingDependency("xdotool"));
        }

        let window_id = run_xdotool(["getactivewindow"])?;
        if window_id.is_empty() {
            return Ok(None);
        }

        let window_title = run_xdotool(["getwindowname", &window_id])?;
        let pid_output = run_xdotool(["getwindowpid", &window_id])?;
        let pid = pid_output
            .trim()
            .parse::<u32>()
            .map_err(|_| TrackingError::InvalidOutput)?;
        let executable = read_executable_from_pid(pid)?;
        let category = categorize_app(&executable, Some(&window_title));

        Ok(Some(ActiveWindowSample {
            app_name: normalize_app_name(&executable),
            executable,
            category,
            window_title: (!window_title.trim().is_empty()).then_some(window_title),
        }))
    }

    fn run_xdotool<const N: usize>(args: [&str; N]) -> Result<String, TrackingError> {
        let output = Command::new("xdotool").args(args).output()?;

        if !output.status.success() {
            return Err(TrackingError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    mod hyprland {
        use super::*;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct HyprlandActiveWindow {
            class: Option<String>,
            initial_class: Option<String>,
            title: Option<String>,
            pid: Option<u32>,
            address: Option<String>,
        }

        pub fn capture_active_window() -> Result<Option<ActiveWindowSample>, TrackingError> {
            if !command_exists("hyprctl") {
                return Err(TrackingError::MissingCompositorCommand("hyprctl"));
            }

            let output = Command::new("hyprctl")
                .args(["-j", "activewindow"])
                .output()?;

            if !output.status.success() {
                return Err(TrackingError::CommandFailed(
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
            }

            let window = serde_json::from_slice::<HyprlandActiveWindow>(&output.stdout)
                .map_err(|_| TrackingError::InvalidOutput)?;

            if window
                .address
                .as_deref()
                .map(|value| value == "0x0")
                .unwrap_or(false)
            {
                return Ok(None);
            }

            let compositor_class = window.class.or(window.initial_class);
            let process_executable = window
                .pid
                .and_then(|pid| read_executable_from_pid(pid).ok());
            let executable =
                choose_executable_hint(process_executable, compositor_class.as_deref());
            let window_title = window.title.filter(|value| !value.trim().is_empty());
            let category = categorize_app(&executable, window_title.as_deref());

            Ok(Some(ActiveWindowSample {
                app_name: normalize_app_name(compositor_class.as_deref().unwrap_or(&executable)),
                executable,
                category,
                window_title,
            }))
        }
    }

    mod sway {
        use super::*;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct SwayNode {
            name: Option<String>,
            app_id: Option<String>,
            focused: bool,
            pid: Option<u32>,
            nodes: Vec<SwayNode>,
            floating_nodes: Vec<SwayNode>,
            window_properties: Option<SwayWindowProperties>,
        }

        #[derive(Debug, Deserialize)]
        struct SwayWindowProperties {
            class: Option<String>,
        }

        pub fn capture_active_window() -> Result<Option<ActiveWindowSample>, TrackingError> {
            if !command_exists("swaymsg") {
                return Err(TrackingError::MissingCompositorCommand("swaymsg"));
            }

            let output = Command::new("swaymsg")
                .args(["-t", "get_tree", "-r"])
                .output()?;

            if !output.status.success() {
                return Err(TrackingError::CommandFailed(
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
            }

            let tree = serde_json::from_slice::<SwayNode>(&output.stdout)
                .map_err(|_| TrackingError::InvalidOutput)?;
            let focused = find_focused_node(&tree).ok_or(TrackingError::InvalidOutput)?;

            let compositor_class = focused.app_id.clone().or_else(|| {
                focused
                    .window_properties
                    .as_ref()
                    .and_then(|value| value.class.clone())
            });
            let process_executable = focused
                .pid
                .and_then(|pid| read_executable_from_pid(pid).ok());
            let executable =
                choose_executable_hint(process_executable, compositor_class.as_deref());
            let window_title = focused
                .name
                .clone()
                .filter(|value| !value.trim().is_empty());
            let category = categorize_app(&executable, window_title.as_deref());

            Ok(Some(ActiveWindowSample {
                app_name: normalize_app_name(compositor_class.as_deref().unwrap_or(&executable)),
                executable,
                category,
                window_title,
            }))
        }

        fn find_focused_node(node: &SwayNode) -> Option<&SwayNode> {
            if node.focused && (node.app_id.is_some() || node.pid.is_some() || node.name.is_some())
            {
                return Some(node);
            }

            node.nodes
                .iter()
                .find_map(find_focused_node)
                .or_else(|| node.floating_nodes.iter().find_map(find_focused_node))
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::Path};

    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    };

    use super::{categorize_app, normalize_app_name, ActiveWindowSample, TrackingError};

    pub fn capture_active_window() -> Result<Option<ActiveWindowSample>, TrackingError> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return Ok(None);
            }

            let title_length = GetWindowTextLengthW(hwnd);
            let mut title_buffer = vec![0u16; (title_length + 1) as usize];
            let written =
                GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), title_buffer.len() as i32);
            let window_title = if written > 0 {
                Some(String::from_utf16_lossy(&title_buffer[..written as usize]))
            } else {
                None
            };

            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == 0 {
                return Ok(None);
            }

            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if process == 0 {
                return Ok(None);
            }

            let mut size = 260u32;
            let mut path_buffer = vec![0u16; size as usize];
            let success =
                QueryFullProcessImageNameW(process, 0, path_buffer.as_mut_ptr(), &mut size);
            CloseHandle(process);

            if success == 0 {
                return Err(TrackingError::InvalidOutput);
            }

            let path = OsString::from_wide(&path_buffer[..size as usize]);
            let executable = Path::new(&path)
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or(TrackingError::InvalidOutput)?
                .to_string();
            let category = categorize_app(&executable, window_title.as_deref());

            Ok(Some(ActiveWindowSample {
                app_name: normalize_app_name(&executable),
                executable,
                category,
                window_title,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        adapter_mode, categorize_app, normalize_app_name, normalize_executable_name,
        tracking_status, TrackingCategory,
    };

    #[test]
    fn exposes_a_tracking_mode() {
        assert!(!adapter_mode().is_empty());
    }

    #[test]
    fn normalizes_executable_names() {
        assert_eq!(normalize_executable_name("/usr/bin/code"), "code");
        assert_eq!(normalize_executable_name("Code.exe"), "code");
    }

    #[test]
    fn derives_human_app_names() {
        assert_eq!(normalize_app_name("code"), "Code");
        assert_eq!(
            normalize_app_name("visual-studio-code"),
            "Visual Studio Code"
        );
    }

    #[test]
    fn classifies_known_apps() {
        assert_eq!(
            categorize_app("Code.exe", Some("focus-time - Visual Studio Code")),
            TrackingCategory::Development
        );
        assert_eq!(
            categorize_app("Slack", Some("Engineering")),
            TrackingCategory::Communication
        );
    }

    #[test]
    fn exposes_runtime_status() {
        let status = tracking_status();

        assert!(!status.message.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_hyprland_active_window_when_available() {
        if !super::is_hyprland_session() || !super::command_exists("hyprctl") {
            return;
        }

        let result = super::linux::capture_active_window();
        assert!(result.is_ok());
    }
}
