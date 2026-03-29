mod connection;
mod migrations;
mod repositories;
mod schema;
mod seed;

use serde::Serialize;
use thiserror::Error;

pub use connection::connect_database;
pub use migrations::{
    list_applied_migrations, run_migrations, AppliedMigration, MigrationDefinition, MigrationError,
};
pub use repositories::{
    CreateSessionInput, CreateSessionSegmentInput, DailyStatRepository, PreferencesRepository,
    Repositories, SaveDailyStatInput, SessionRepository, TrackedAppRepository,
    UpsertTrackedAppInput,
};
pub use schema::{initial_schema, DatabaseSchema, TableDefinition};
pub use seed::{seed_development_data, DevelopmentSeedReport};

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

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(#[from] chrono::ParseError),
    #[error("invalid date: {0}")]
    InvalidDate(#[source] chrono::ParseError),
    #[error("unknown enum value: {0}")]
    UnknownEnumValue(String),
}

#[cfg(test)]
mod tests;
