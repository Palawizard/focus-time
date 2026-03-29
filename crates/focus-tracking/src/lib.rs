pub const fn crate_name() -> &'static str {
    "focus-tracking"
}

pub fn adapter_mode() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows-native-planned"
    } else if cfg!(target_os = "linux") {
        "linux-adapter-planned"
    } else {
        "unsupported-platform"
    }
}

#[cfg(test)]
mod tests {
    use super::adapter_mode;

    #[test]
    fn exposes_a_tracking_mode() {
        assert!(!adapter_mode().is_empty());
    }
}
