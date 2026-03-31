use tempfile::tempdir;

use crate::{
    connect_database, list_applied_migrations, run_migrations, seed_development_data,
    storage_profile,
};

#[test]
fn exposes_the_current_storage_profile() {
    let profile = storage_profile();

    assert_eq!(profile.engine, "sqlite");
    assert_eq!(profile.mode, "sqlite");
    assert_eq!(profile.schema_version, 3);
}

#[test]
fn exposes_the_initial_schema_definition() {
    let schema = crate::initial_schema();

    assert_eq!(schema.version, 3);
    assert_eq!(schema.tables.len(), 8);
}

#[tokio::test]
async fn migrates_an_empty_database() {
    let temp = tempdir().expect("temporary directory should be created");
    let database_path = temp.path().join("focus-time.sqlite");
    let pool = connect_database(&database_path)
        .await
        .expect("database should open");

    let applied = run_migrations(&pool)
        .await
        .expect("migrations should run on an empty database");

    assert_eq!(applied.len(), 3);

    let applied_versions = list_applied_migrations(&pool)
        .await
        .expect("migration history should be readable");
    assert_eq!(applied_versions.len(), 3);

    let preferences_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM user_preferences")
        .fetch_one(&pool)
        .await
        .expect("preferences row should exist");
    assert_eq!(preferences_count, 1);
}

#[tokio::test]
async fn preserves_existing_data_when_migrations_run_again() {
    let temp = tempdir().expect("temporary directory should be created");
    let database_path = temp.path().join("focus-time.sqlite");

    let first_pool = connect_database(&database_path)
        .await
        .expect("database should open");
    run_migrations(&first_pool)
        .await
        .expect("first migration pass should succeed");
    let seed_report = seed_development_data(&first_pool)
        .await
        .expect("fixture seed should succeed");
    assert!(!seed_report.skipped);
    first_pool.close().await;

    let second_pool = connect_database(&database_path)
        .await
        .expect("database should reopen");
    let applied = run_migrations(&second_pool)
        .await
        .expect("second migration pass should succeed");
    assert!(applied.is_empty());

    let session_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM sessions")
        .fetch_one(&second_pool)
        .await
        .expect("sessions should remain available");
    assert_eq!(session_count, 2);

    let applied_versions = list_applied_migrations(&second_pool)
        .await
        .expect("migration history should still be available");
    assert_eq!(applied_versions.len(), 3);
}
