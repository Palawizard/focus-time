use tempfile::tempdir;

use crate::{
    connect_database, list_applied_migrations, run_migrations, seed_development_data,
    storage_profile, ListSessionsPageInput, ReplaceSessionInput, Repositories,
    SessionHistoryFiltersInput,
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

#[tokio::test]
async fn filters_and_replaces_history_sessions() {
    let temp = tempdir().expect("temporary directory should be created");
    let database_path = temp.path().join("focus-time.sqlite");
    let pool = connect_database(&database_path)
        .await
        .expect("database should open");
    run_migrations(&pool).await.expect("migrations should run");
    seed_development_data(&pool)
        .await
        .expect("fixture seed should succeed");

    let repositories = Repositories::new(pool.clone());
    let code_app_id =
        sqlx::query_scalar::<_, i64>("SELECT id FROM tracked_apps WHERE executable = 'Code.exe'")
            .fetch_one(&pool)
            .await
            .expect("seeded tracked app should exist");

    let filtered = repositories
        .sessions
        .list_filtered(ListSessionsPageInput {
            limit: Some(10),
            offset: 0,
            filters: SessionHistoryFiltersInput {
                preset_label: Some("Deep work".to_string()),
                tracked_app_id: Some(code_app_id),
                ..SessionHistoryFiltersInput::default()
            },
        })
        .await
        .expect("history filter should succeed");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].preset_label.as_deref(), Some("Deep work"));

    let replaced = repositories
        .sessions
        .replace(ReplaceSessionInput {
            session_id: filtered[0].id,
            started_at: filtered[0].started_at,
            ended_at: filtered[0].ended_at,
            planned_focus_minutes: 50,
            actual_focus_seconds: 2_700,
            break_seconds: 120,
            status: focus_domain::SessionStatus::Completed,
            preset_label: Some("Deep work+".to_string()),
            note: Some("Retrospective review".to_string()),
        })
        .await
        .expect("session replacement should succeed");

    assert_eq!(replaced.planned_focus_minutes, 50);
    assert_eq!(replaced.actual_focus_seconds, 2_700);
    assert_eq!(replaced.break_seconds, 120);
    assert_eq!(replaced.preset_label.as_deref(), Some("Deep work+"));
    assert_eq!(replaced.note.as_deref(), Some("Retrospective review"));
}

#[tokio::test]
async fn deletes_history_sessions_and_cascades_segments() {
    let temp = tempdir().expect("temporary directory should be created");
    let database_path = temp.path().join("focus-time.sqlite");
    let pool = connect_database(&database_path)
        .await
        .expect("database should open");
    run_migrations(&pool).await.expect("migrations should run");
    seed_development_data(&pool)
        .await
        .expect("fixture seed should succeed");

    let repositories = Repositories::new(pool.clone());
    let sessions = repositories
        .sessions
        .list_filtered(ListSessionsPageInput {
            limit: None,
            offset: 0,
            filters: SessionHistoryFiltersInput::default(),
        })
        .await
        .expect("seeded sessions should be readable");
    let deleted_session_id = sessions[0].id;

    repositories
        .sessions
        .delete(deleted_session_id)
        .await
        .expect("session deletion should succeed");

    let remaining_sessions = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM sessions")
        .fetch_one(&pool)
        .await
        .expect("remaining sessions should be countable");
    let remaining_segments =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM session_segments WHERE session_id = ?")
            .bind(deleted_session_id)
            .fetch_one(&pool)
            .await
            .expect("remaining segments should be countable");

    assert_eq!(remaining_sessions, 1);
    assert_eq!(remaining_segments, 0);
}
