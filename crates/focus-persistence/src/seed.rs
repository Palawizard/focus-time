use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::{Acquire, SqlitePool};

use crate::PersistenceError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentSeedReport {
    pub skipped: bool,
    pub sessions_inserted: u32,
    pub tracked_apps_upserted: u32,
    pub daily_stats_upserted: u32,
}

pub async fn seed_development_data(
    pool: &SqlitePool,
) -> Result<DevelopmentSeedReport, PersistenceError> {
    let existing_sessions = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM sessions")
        .fetch_one(pool)
        .await?;

    if existing_sessions > 0 {
        return Ok(DevelopmentSeedReport {
            skipped: true,
            sessions_inserted: 0,
            tracked_apps_upserted: 0,
            daily_stats_upserted: 0,
        });
    }

    let mut transaction = pool.begin().await?;
    let connection = transaction.acquire().await?;

    let now = Utc::now();
    let app_now = now.to_rfc3339();

    sqlx::query(
        r#"
        UPDATE user_preferences
        SET
          focus_minutes = 25,
          short_break_minutes = 5,
          long_break_minutes = 15,
          sessions_until_long_break = 4,
          auto_start_breaks = 0,
          auto_start_focus = 0,
          tracking_enabled = 1,
          tracking_permission_granted = 1,
          tracking_onboarding_completed = 1,
          notifications_enabled = 1,
          theme = 'system',
          updated_at = ?
        WHERE id = 1
        "#,
    )
    .bind(app_now.clone())
    .execute(&mut *connection)
    .await?;

    let code_app_id = upsert_tracked_app(
        &mut *connection,
        "Visual Studio Code",
        "Code.exe",
        "development",
        Some("#0078d4"),
        false,
        &app_now,
    )
    .await?;
    let browser_app_id = upsert_tracked_app(
        &mut *connection,
        "Arc",
        "Arc.exe",
        "browser",
        Some("#60b7ff"),
        false,
        &app_now,
    )
    .await?;

    let session_one_started = now - Duration::days(1) - Duration::minutes(90);
    let session_one_ended = session_one_started + Duration::minutes(25);
    let session_one_id = insert_session(
        &mut *connection,
        session_one_started.to_rfc3339(),
        Some(session_one_ended.to_rfc3339()),
        25,
        1_500,
        300,
        "completed",
        Some("Classic"),
        Some("UI polish"),
        &app_now,
    )
    .await?;
    insert_segment(
        &mut *connection,
        session_one_id,
        Some(code_app_id),
        "focus",
        Some("Focus Time"),
        session_one_started.to_rfc3339(),
        (session_one_started + Duration::minutes(20)).to_rfc3339(),
        1_200,
        &app_now,
    )
    .await?;
    insert_segment(
        &mut *connection,
        session_one_id,
        Some(browser_app_id),
        "break",
        Some("Docs"),
        (session_one_started + Duration::minutes(20)).to_rfc3339(),
        session_one_ended.to_rfc3339(),
        300,
        &app_now,
    )
    .await?;

    let session_two_started = now - Duration::hours(4);
    let session_two_ended = session_two_started + Duration::minutes(45);
    let session_two_id = insert_session(
        &mut *connection,
        session_two_started.to_rfc3339(),
        Some(session_two_ended.to_rfc3339()),
        45,
        2_400,
        300,
        "completed",
        Some("Deep work"),
        Some("Planning"),
        &app_now,
    )
    .await?;
    insert_segment(
        &mut *connection,
        session_two_id,
        Some(code_app_id),
        "focus",
        Some("Roadmap"),
        session_two_started.to_rfc3339(),
        (session_two_started + Duration::minutes(35)).to_rfc3339(),
        2_100,
        &app_now,
    )
    .await?;
    insert_segment(
        &mut *connection,
        session_two_id,
        Some(browser_app_id),
        "idle",
        Some("Inbox"),
        (session_two_started + Duration::minutes(35)).to_rfc3339(),
        session_two_ended.to_rfc3339(),
        600,
        &app_now,
    )
    .await?;

    upsert_daily_stat(
        &mut *connection,
        &(now - Duration::days(1))
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
        1_500,
        300,
        1,
        0,
        Some(code_app_id),
        &app_now,
    )
    .await?;
    upsert_daily_stat(
        &mut *connection,
        &now.date_naive().format("%Y-%m-%d").to_string(),
        2_400,
        300,
        1,
        0,
        Some(code_app_id),
        &app_now,
    )
    .await?;

    transaction.commit().await?;

    Ok(DevelopmentSeedReport {
        skipped: false,
        sessions_inserted: 2,
        tracked_apps_upserted: 2,
        daily_stats_upserted: 2,
    })
}

async fn upsert_tracked_app(
    connection: &mut sqlx::SqliteConnection,
    name: &str,
    executable: &str,
    category: &str,
    color_hex: Option<&str>,
    is_excluded: bool,
    timestamp: &str,
) -> Result<i64, PersistenceError> {
    sqlx::query(
        r#"
        INSERT INTO tracked_apps (
          name,
          executable,
          category,
          color_hex,
          is_excluded,
          created_at,
          updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(executable) DO UPDATE SET
          name = excluded.name,
          category = excluded.category,
          color_hex = excluded.color_hex,
          is_excluded = excluded.is_excluded,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(name)
    .bind(executable)
    .bind(category)
    .bind(color_hex)
    .bind(is_excluded)
    .bind(timestamp)
    .bind(timestamp)
    .execute(&mut *connection)
    .await?;

    let app_id = sqlx::query_scalar::<_, i64>("SELECT id FROM tracked_apps WHERE executable = ?")
        .bind(executable)
        .fetch_one(&mut *connection)
        .await?;

    Ok(app_id)
}

#[allow(clippy::too_many_arguments)]
async fn insert_session(
    connection: &mut sqlx::SqliteConnection,
    started_at: String,
    ended_at: Option<String>,
    planned_focus_minutes: i32,
    actual_focus_seconds: i64,
    break_seconds: i64,
    status: &str,
    preset_label: Option<&str>,
    note: Option<&str>,
    timestamp: &str,
) -> Result<i64, PersistenceError> {
    let session_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO sessions (
          started_at,
          ended_at,
          planned_focus_minutes,
          actual_focus_seconds,
          break_seconds,
          status,
          preset_label,
          note,
          created_at,
          updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(started_at)
    .bind(ended_at)
    .bind(planned_focus_minutes)
    .bind(actual_focus_seconds)
    .bind(break_seconds)
    .bind(status)
    .bind(preset_label)
    .bind(note)
    .bind(timestamp)
    .bind(timestamp)
    .fetch_one(&mut *connection)
    .await?;

    Ok(session_id)
}

#[allow(clippy::too_many_arguments)]
async fn insert_segment(
    connection: &mut sqlx::SqliteConnection,
    session_id: i64,
    tracked_app_id: Option<i64>,
    kind: &str,
    window_title: Option<&str>,
    started_at: String,
    ended_at: String,
    duration_seconds: i64,
    timestamp: &str,
) -> Result<(), PersistenceError> {
    sqlx::query(
        r#"
        INSERT INTO session_segments (
          session_id,
          tracked_app_id,
          kind,
          window_title,
          started_at,
          ended_at,
          duration_seconds,
          created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(session_id)
    .bind(tracked_app_id)
    .bind(kind)
    .bind(window_title)
    .bind(started_at)
    .bind(ended_at)
    .bind(duration_seconds)
    .bind(timestamp)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn upsert_daily_stat(
    connection: &mut sqlx::SqliteConnection,
    date: &str,
    focus_seconds: i64,
    break_seconds: i64,
    completed_sessions: i32,
    interrupted_sessions: i32,
    top_app_id: Option<i64>,
    timestamp: &str,
) -> Result<(), PersistenceError> {
    sqlx::query(
        r#"
        INSERT INTO daily_stats (
          stat_date,
          focus_seconds,
          break_seconds,
          completed_sessions,
          interrupted_sessions,
          top_app_id,
          updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(stat_date) DO UPDATE SET
          focus_seconds = excluded.focus_seconds,
          break_seconds = excluded.break_seconds,
          completed_sessions = excluded.completed_sessions,
          interrupted_sessions = excluded.interrupted_sessions,
          top_app_id = excluded.top_app_id,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(date)
    .bind(focus_seconds)
    .bind(break_seconds)
    .bind(completed_sessions)
    .bind(interrupted_sessions)
    .bind(top_app_id)
    .bind(timestamp)
    .execute(&mut *connection)
    .await?;

    Ok(())
}
