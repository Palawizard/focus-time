mod migrations;
mod schema;

use serde::Serialize;

pub use migrations::{
    list_applied_migrations, run_migrations, AppliedMigration, MigrationDefinition, MigrationError,
};
pub use schema::{initial_schema, DatabaseSchema, TableDefinition};

pub const fn crate_name() -> &'static str {
    "focus-persistence"
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageProfile {
    pub engine: &'static str,
    pub mode: &'static str,
    pub schema_version: u32,
}

pub fn storage_profile() -> StorageProfile {
    StorageProfile {
        engine: "sqlite",
        mode: "sqlite",
        schema_version: initial_schema().version,
    }
}

#[cfg(test)]
mod tests {
    use super::{initial_schema, storage_profile};

    #[test]
    fn exposes_the_current_storage_profile() {
        let profile = storage_profile();

        assert_eq!(profile.engine, "sqlite");
        assert_eq!(profile.mode, "sqlite");
        assert_eq!(profile.schema_version, 1);
    }

    #[test]
    fn exposes_the_initial_schema_definition() {
        let schema = initial_schema();

        assert_eq!(schema.version, 1);
        assert_eq!(schema.tables.len(), 7);
    }
}
