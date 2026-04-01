use sqlx::{Executor, SqlitePool};
use thiserror::Error;

const MIGRATION_0001: &str =
    include_str!("../../../apps/desktop/src-tauri/migrations/0001_initial_schema.sql");
const MIGRATION_0002: &str =
    include_str!("../../../apps/desktop/src-tauri/migrations/0002_supporting_indexes.sql");
const MIGRATION_0003: &str =
    include_str!("../../../apps/desktop/src-tauri/migrations/0003_tracking_foundation.sql");
const MIGRATION_0004: &str =
    include_str!("../../../apps/desktop/src-tauri/migrations/0004_gamification_foundation.sql");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationDefinition {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

const MIGRATIONS: [MigrationDefinition; 4] = [
    MigrationDefinition {
        version: 1,
        name: "initial_schema",
        sql: MIGRATION_0001,
    },
    MigrationDefinition {
        version: 2,
        name: "supporting_indexes",
        sql: MIGRATION_0002,
    },
    MigrationDefinition {
        version: 3,
        name: "tracking_foundation",
        sql: MIGRATION_0003,
    },
    MigrationDefinition {
        version: 4,
        name: "gamification_foundation",
        sql: MIGRATION_0004,
    },
];

pub async fn run_migrations(pool: &SqlitePool) -> Result<Vec<AppliedMigration>, MigrationError> {
    ensure_migrations_table(pool).await?;

    let mut applied = Vec::new();

    for migration in MIGRATIONS {
        let already_applied = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM __focus_time_migrations WHERE version = ?",
        )
        .bind(migration.version)
        .fetch_one(pool)
        .await?;

        if already_applied > 0 {
            continue;
        }

        let mut transaction = pool.begin().await?;

        for statement in migration
            .sql
            .split(';')
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
        {
            transaction.execute(statement).await?;
        }

        sqlx::query(
            "INSERT INTO __focus_time_migrations (version, name, applied_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(migration.version)
        .bind(migration.name)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        applied.push(AppliedMigration {
            version: migration.version,
            name: migration.name.to_string(),
        });
    }

    Ok(applied)
}

pub async fn list_applied_migrations(
    pool: &SqlitePool,
) -> Result<Vec<AppliedMigration>, MigrationError> {
    ensure_migrations_table(pool).await?;

    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT version, name FROM __focus_time_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(version, name)| AppliedMigration { version, name })
        .collect())
}

async fn ensure_migrations_table(pool: &SqlitePool) -> Result<(), MigrationError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS __focus_time_migrations (
          version INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          applied_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
