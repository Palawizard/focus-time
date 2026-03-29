use serde::Serialize;

pub const fn crate_name() -> &'static str {
    "focus-persistence"
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageProfile {
    pub engine: &'static str,
    pub mode: &'static str,
}

pub fn storage_profile() -> StorageProfile {
    StorageProfile {
        engine: "sqlite",
        mode: "sqlite-planned",
    }
}

#[cfg(test)]
mod tests {
    use super::storage_profile;

    #[test]
    fn exposes_the_default_storage_profile() {
        let profile = storage_profile();

        assert_eq!(profile.engine, "sqlite");
        assert_eq!(profile.mode, "sqlite-planned");
    }
}
