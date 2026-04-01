use chrono::{DateTime, NaiveDate, Utc};
use focus_domain::{
    DailyStat, Session, SessionSegment, SessionSegmentKind, SessionStatus, ThemePreference,
    TrackedApp, TrackedWindowEvent, TrackingCategory, TrackingExclusionKind, TrackingExclusionRule,
    UserPreference,
};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct SessionRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct PreferencesRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct TrackedAppRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct DailyStatRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct TrackingRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct Repositories {
    pub sessions: SessionRepository,
    pub preferences: PreferencesRepository,
    pub tracked_apps: TrackedAppRepository,
    pub daily_stats: DailyStatRepository,
    pub tracking: TrackingRepository,
}

#[derive(Debug, Clone)]
pub struct CreateSessionInput {
    pub started_at: DateTime<Utc>,
    pub planned_focus_minutes: i32,
    pub status: SessionStatus,
    pub preset_label: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateSessionInput {
    pub session_id: i64,
    pub ended_at: Option<DateTime<Utc>>,
    pub actual_focus_seconds: i64,
    pub break_seconds: i64,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Default)]
pub struct SessionHistoryFiltersInput {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub min_duration_seconds: Option<i64>,
    pub max_duration_seconds: Option<i64>,
    pub preset_label: Option<String>,
    pub status: Option<SessionStatus>,
    pub tracked_app_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ListSessionsPageInput {
    pub limit: Option<u32>,
    pub offset: u32,
    pub filters: SessionHistoryFiltersInput,
}

#[derive(Debug, Clone)]
pub struct ReplaceSessionInput {
    pub session_id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub planned_focus_minutes: i32,
    pub actual_focus_seconds: i64,
    pub break_seconds: i64,
    pub status: SessionStatus,
    pub preset_label: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionSegmentInput {
    pub session_id: i64,
    pub tracked_app_id: Option<i64>,
    pub kind: SessionSegmentKind,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct UpsertTrackedAppInput {
    pub name: String,
    pub executable: String,
    pub category: TrackingCategory,
    pub color_hex: Option<String>,
    pub is_excluded: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterTrackedAppInput {
    pub name: String,
    pub executable: String,
    pub category: TrackingCategory,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTrackedWindowEventInput {
    pub session_id: Option<i64>,
    pub tracked_app_id: Option<i64>,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CreateTrackingExclusionRuleInput {
    pub kind: TrackingExclusionKind,
    pub pattern: String,
}

#[derive(Debug, Clone)]
pub struct SaveDailyStatInput {
    pub date: NaiveDate,
    pub focus_seconds: i64,
    pub break_seconds: i64,
    pub completed_sessions: i32,
    pub interrupted_sessions: i32,
    pub top_app_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SessionAppUsageSlice {
    pub session_id: i64,
    pub tracked_app_id: i64,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct SessionInterruptionSlice {
    pub session_id: i64,
    pub interruption_count: i64,
    pub interruption_seconds: i64,
}

impl Repositories {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            sessions: SessionRepository::new(pool.clone()),
            preferences: PreferencesRepository::new(pool.clone()),
            tracked_apps: TrackedAppRepository::new(pool.clone()),
            tracking: TrackingRepository::new(pool.clone()),
            daily_stats: DailyStatRepository::new(pool),
        }
    }
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: CreateSessionInput) -> Result<Session, PersistenceError> {
        let now = Utc::now();
        let session_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO sessions (
              started_at,
              planned_focus_minutes,
              status,
              preset_label,
              note,
              created_at,
              updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(input.started_at.to_rfc3339())
        .bind(input.planned_focus_minutes)
        .bind(input.status.as_str())
        .bind(input.preset_label)
        .bind(input.note)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        self.get_by_id(session_id).await
    }

    pub async fn list(&self, limit: u32) -> Result<Vec<Session>, PersistenceError> {
        let rows = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT
              id,
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
            FROM sessions
            ORDER BY started_at DESC
            LIMIT ?
            "#,
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(SessionRow::try_into_domain).collect()
    }

    pub async fn list_filtered(
        &self,
        input: ListSessionsPageInput,
    ) -> Result<Vec<Session>, PersistenceError> {
        let mut query = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
              id,
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
            FROM sessions
            WHERE 1 = 1
            "#,
        );

        apply_session_history_filters(&mut query, &input.filters);
        query.push(" ORDER BY started_at DESC");

        if let Some(limit) = input.limit {
            query
                .push(" LIMIT ")
                .push_bind(i64::from(limit))
                .push(" OFFSET ")
                .push_bind(i64::from(input.offset));
        } else if input.offset > 0 {
            query
                .push(" LIMIT -1 OFFSET ")
                .push_bind(i64::from(input.offset));
        }

        let rows = query
            .build_query_as::<SessionRow>()
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(SessionRow::try_into_domain).collect()
    }

    pub async fn get_by_id(&self, session_id: i64) -> Result<Session, PersistenceError> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT
              id,
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
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn add_segment(
        &self,
        input: CreateSessionSegmentInput,
    ) -> Result<SessionSegment, PersistenceError> {
        let created_at = Utc::now().to_rfc3339();

        let segment_id = sqlx::query_scalar::<_, i64>(
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
            RETURNING id
            "#,
        )
        .bind(input.session_id)
        .bind(input.tracked_app_id)
        .bind(input.kind.as_str())
        .bind(input.window_title)
        .bind(input.started_at.to_rfc3339())
        .bind(input.ended_at.to_rfc3339())
        .bind(input.duration_seconds)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, SessionSegmentRow>(
            r#"
            SELECT
              id,
              session_id,
              tracked_app_id,
              kind,
              window_title,
              started_at,
              ended_at,
              duration_seconds,
              created_at
            FROM session_segments
            WHERE id = ?
            "#,
        )
        .bind(segment_id)
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn list_segments(
        &self,
        session_id: i64,
    ) -> Result<Vec<SessionSegment>, PersistenceError> {
        let rows = sqlx::query_as::<_, SessionSegmentRow>(
            r#"
            SELECT
              id,
              session_id,
              tracked_app_id,
              kind,
              window_title,
              started_at,
              ended_at,
              duration_seconds,
              created_at
            FROM session_segments
            WHERE session_id = ?
            ORDER BY started_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(SessionSegmentRow::try_into_domain)
            .collect()
    }

    pub async fn list_segments_for_sessions(
        &self,
        session_ids: &[i64],
    ) -> Result<Vec<SessionSegment>, PersistenceError> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
              id,
              session_id,
              tracked_app_id,
              kind,
              window_title,
              started_at,
              ended_at,
              duration_seconds,
              created_at
            FROM session_segments
            WHERE session_id IN (
            "#,
        );
        {
            let mut separated = query.separated(", ");

            for session_id in session_ids {
                separated.push_bind(session_id);
            }
        }
        query.push(
            r#"
            )
            ORDER BY started_at ASC
            "#,
        );

        let rows = query
            .build_query_as::<SessionSegmentRow>()
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(SessionSegmentRow::try_into_domain)
            .collect()
    }

    pub async fn update(&self, input: UpdateSessionInput) -> Result<Session, PersistenceError> {
        sqlx::query(
            r#"
            UPDATE sessions
            SET
              ended_at = ?,
              actual_focus_seconds = ?,
              break_seconds = ?,
              status = ?,
              updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.ended_at.map(|value| value.to_rfc3339()))
        .bind(input.actual_focus_seconds)
        .bind(input.break_seconds)
        .bind(input.status.as_str())
        .bind(Utc::now().to_rfc3339())
        .bind(input.session_id)
        .execute(&self.pool)
        .await?;

        self.get_by_id(input.session_id).await
    }

    pub async fn replace(&self, input: ReplaceSessionInput) -> Result<Session, PersistenceError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET
              started_at = ?,
              ended_at = ?,
              planned_focus_minutes = ?,
              actual_focus_seconds = ?,
              break_seconds = ?,
              status = ?,
              preset_label = ?,
              note = ?,
              updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.started_at.to_rfc3339())
        .bind(input.ended_at.map(|value| value.to_rfc3339()))
        .bind(input.planned_focus_minutes)
        .bind(input.actual_focus_seconds)
        .bind(input.break_seconds)
        .bind(input.status.as_str())
        .bind(input.preset_label)
        .bind(input.note)
        .bind(Utc::now().to_rfc3339())
        .bind(input.session_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound(format!(
                "session {}",
                input.session_id
            )));
        }

        self.get_by_id(input.session_id).await
    }

    pub async fn delete(&self, session_id: i64) -> Result<(), PersistenceError> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound(format!("session {session_id}")));
        }

        Ok(())
    }

    pub async fn list_app_usage(
        &self,
        session_ids: &[i64],
    ) -> Result<Vec<SessionAppUsageSlice>, PersistenceError> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
              session_id,
              tracked_app_id,
              SUM(duration_seconds) AS duration_seconds
            FROM session_segments
            WHERE tracked_app_id IS NOT NULL
              AND session_id IN (
            "#,
        );
        {
            let mut separated = query.separated(", ");

            for session_id in session_ids {
                separated.push_bind(session_id);
            }
        }
        query.push(
            r#"
              )
            GROUP BY session_id, tracked_app_id
            ORDER BY session_id ASC, duration_seconds DESC
            "#,
        );

        let rows = query
            .build_query_as::<SessionAppUsageRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| SessionAppUsageSlice {
                session_id: row.session_id,
                tracked_app_id: row.tracked_app_id,
                duration_seconds: row.duration_seconds,
            })
            .collect())
    }

    pub async fn list_interruptions(
        &self,
        session_ids: &[i64],
    ) -> Result<Vec<SessionInterruptionSlice>, PersistenceError> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
              session_id,
              SUM(CASE WHEN kind = 'idle' THEN 1 ELSE 0 END) AS interruption_count,
              SUM(CASE WHEN kind = 'idle' THEN duration_seconds ELSE 0 END) AS interruption_seconds
            FROM session_segments
            WHERE session_id IN (
            "#,
        );
        {
            let mut separated = query.separated(", ");

            for session_id in session_ids {
                separated.push_bind(session_id);
            }
        }
        query.push(
            r#"
            )
            GROUP BY session_id
            ORDER BY session_id ASC
            "#,
        );

        let rows = query
            .build_query_as::<SessionInterruptionRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| SessionInterruptionSlice {
                session_id: row.session_id,
                interruption_count: row.interruption_count,
                interruption_seconds: row.interruption_seconds,
            })
            .collect())
    }
}

impl PreferencesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<UserPreference, PersistenceError> {
        let row = sqlx::query_as::<_, UserPreferenceRow>(
            r#"
            SELECT
              focus_minutes,
              short_break_minutes,
              long_break_minutes,
              sessions_until_long_break,
              auto_start_breaks,
              auto_start_focus,
              tracking_enabled,
              tracking_permission_granted,
              tracking_onboarding_completed,
              notifications_enabled,
              theme,
              updated_at
            FROM user_preferences
            WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn save(
        &self,
        preferences: &UserPreference,
    ) -> Result<UserPreference, PersistenceError> {
        sqlx::query(
            r#"
            UPDATE user_preferences
            SET
              focus_minutes = ?,
              short_break_minutes = ?,
              long_break_minutes = ?,
              sessions_until_long_break = ?,
              auto_start_breaks = ?,
              auto_start_focus = ?,
              tracking_enabled = ?,
              tracking_permission_granted = ?,
              tracking_onboarding_completed = ?,
              notifications_enabled = ?,
              theme = ?,
              updated_at = ?
            WHERE id = 1
            "#,
        )
        .bind(preferences.focus_minutes)
        .bind(preferences.short_break_minutes)
        .bind(preferences.long_break_minutes)
        .bind(preferences.sessions_until_long_break)
        .bind(preferences.auto_start_breaks)
        .bind(preferences.auto_start_focus)
        .bind(preferences.tracking_enabled)
        .bind(preferences.tracking_permission_granted)
        .bind(preferences.tracking_onboarding_completed)
        .bind(preferences.notifications_enabled)
        .bind(preferences.theme.as_str())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.get().await
    }
}

impl TrackedAppRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<TrackedApp>, PersistenceError> {
        let rows = sqlx::query_as::<_, TrackedAppRow>(
            r#"
            SELECT
              id,
              name,
              executable,
              category,
              color_hex,
              is_excluded,
              created_at,
              updated_at
            FROM tracked_apps
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TrackedAppRow::try_into_domain)
            .collect()
    }

    pub async fn upsert(
        &self,
        input: UpsertTrackedAppInput,
    ) -> Result<TrackedApp, PersistenceError> {
        let now = Utc::now().to_rfc3339();

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
        .bind(input.name)
        .bind(input.executable.clone())
        .bind(input.category.as_str())
        .bind(input.color_hex)
        .bind(input.is_excluded)
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, TrackedAppRow>(
            r#"
            SELECT
              id,
              name,
              executable,
              category,
              color_hex,
              is_excluded,
              created_at,
              updated_at
            FROM tracked_apps
            WHERE executable = ?
            "#,
        )
        .bind(input.executable)
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn find_by_executable(
        &self,
        executable: &str,
    ) -> Result<Option<TrackedApp>, PersistenceError> {
        let row = sqlx::query_as::<_, TrackedAppRow>(
            r#"
            SELECT
              id,
              name,
              executable,
              category,
              color_hex,
              is_excluded,
              created_at,
              updated_at
            FROM tracked_apps
            WHERE executable = ?
            "#,
        )
        .bind(executable)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TrackedAppRow::try_into_domain).transpose()
    }

    pub async fn register_seen(
        &self,
        input: RegisterTrackedAppInput,
    ) -> Result<TrackedApp, PersistenceError> {
        let now = Utc::now().to_rfc3339();

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
            VALUES (?, ?, ?, ?, 0, ?, ?)
            ON CONFLICT(executable) DO UPDATE SET
              name = excluded.name,
              category = excluded.category,
              color_hex = COALESCE(tracked_apps.color_hex, excluded.color_hex),
              updated_at = excluded.updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.executable.clone())
        .bind(input.category.as_str())
        .bind(input.color_hex)
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.find_by_executable(&input.executable)
            .await?
            .ok_or_else(|| PersistenceError::UnknownEnumValue("tracked app missing".to_string()))
    }
}

impl DailyStatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, limit: u32) -> Result<Vec<DailyStat>, PersistenceError> {
        let rows = sqlx::query_as::<_, DailyStatRow>(
            r#"
            SELECT
              stat_date,
              focus_seconds,
              break_seconds,
              completed_sessions,
              interrupted_sessions,
              top_app_id,
              updated_at
            FROM daily_stats
            ORDER BY stat_date DESC
            LIMIT ?
            "#,
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(DailyStatRow::try_into_domain)
            .collect()
    }

    pub async fn upsert(&self, input: SaveDailyStatInput) -> Result<DailyStat, PersistenceError> {
        let updated_at = Utc::now().to_rfc3339();

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
        .bind(input.date.format("%Y-%m-%d").to_string())
        .bind(input.focus_seconds)
        .bind(input.break_seconds)
        .bind(input.completed_sessions)
        .bind(input.interrupted_sessions)
        .bind(input.top_app_id)
        .bind(updated_at)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, DailyStatRow>(
            r#"
            SELECT
              stat_date,
              focus_seconds,
              break_seconds,
              completed_sessions,
              interrupted_sessions,
              top_app_id,
              updated_at
            FROM daily_stats
            WHERE stat_date = ?
            "#,
        )
        .bind(input.date.format("%Y-%m-%d").to_string())
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }
}

impl TrackingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_window_events(
        &self,
        limit: u32,
    ) -> Result<Vec<TrackedWindowEvent>, PersistenceError> {
        let rows = sqlx::query_as::<_, TrackedWindowEventRow>(
            r#"
            SELECT
              tracked_window_events.id,
              tracked_window_events.session_id,
              tracked_window_events.tracked_app_id,
              tracked_apps.name AS app_name,
              tracked_apps.executable,
              tracked_apps.category,
              tracked_window_events.window_title,
              tracked_window_events.started_at,
              tracked_window_events.ended_at,
              tracked_window_events.created_at
            FROM tracked_window_events
            LEFT JOIN tracked_apps
              ON tracked_apps.id = tracked_window_events.tracked_app_id
            ORDER BY tracked_window_events.started_at DESC
            LIMIT ?
            "#,
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TrackedWindowEventRow::try_into_domain)
            .collect()
    }

    pub async fn create_window_event(
        &self,
        input: CreateTrackedWindowEventInput,
    ) -> Result<TrackedWindowEvent, PersistenceError> {
        let created_at = Utc::now().to_rfc3339();

        let event_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO tracked_window_events (
              session_id,
              tracked_app_id,
              window_title,
              started_at,
              ended_at,
              created_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(input.session_id)
        .bind(input.tracked_app_id)
        .bind(input.window_title)
        .bind(input.started_at.to_rfc3339())
        .bind(input.ended_at.map(|value| value.to_rfc3339()))
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, TrackedWindowEventRow>(
            r#"
            SELECT
              tracked_window_events.id,
              tracked_window_events.session_id,
              tracked_window_events.tracked_app_id,
              tracked_apps.name AS app_name,
              tracked_apps.executable,
              tracked_apps.category,
              tracked_window_events.window_title,
              tracked_window_events.started_at,
              tracked_window_events.ended_at,
              tracked_window_events.created_at
            FROM tracked_window_events
            LEFT JOIN tracked_apps
              ON tracked_apps.id = tracked_window_events.tracked_app_id
            WHERE tracked_window_events.id = ?
            "#,
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn list_window_events_for_session(
        &self,
        session_id: i64,
    ) -> Result<Vec<TrackedWindowEvent>, PersistenceError> {
        let rows = sqlx::query_as::<_, TrackedWindowEventRow>(
            r#"
            SELECT
              tracked_window_events.id,
              tracked_window_events.session_id,
              tracked_window_events.tracked_app_id,
              tracked_apps.name AS app_name,
              tracked_apps.executable,
              tracked_apps.category,
              tracked_window_events.window_title,
              tracked_window_events.started_at,
              tracked_window_events.ended_at,
              tracked_window_events.created_at
            FROM tracked_window_events
            LEFT JOIN tracked_apps
              ON tracked_apps.id = tracked_window_events.tracked_app_id
            WHERE tracked_window_events.session_id = ?
            ORDER BY tracked_window_events.started_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TrackedWindowEventRow::try_into_domain)
            .collect()
    }

    pub async fn delete_window_events_for_session(
        &self,
        session_id: i64,
    ) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM tracked_window_events WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_exclusion_rules(
        &self,
    ) -> Result<Vec<TrackingExclusionRule>, PersistenceError> {
        let rows = sqlx::query_as::<_, TrackingExclusionRuleRow>(
            r#"
            SELECT
              id,
              kind,
              pattern,
              created_at,
              updated_at
            FROM tracking_exclusion_rules
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(TrackingExclusionRuleRow::try_into_domain)
            .collect()
    }

    pub async fn create_exclusion_rule(
        &self,
        input: CreateTrackingExclusionRuleInput,
    ) -> Result<TrackingExclusionRule, PersistenceError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO tracking_exclusion_rules (kind, pattern, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(kind, pattern) DO UPDATE SET
              updated_at = excluded.updated_at
            "#,
        )
        .bind(input.kind.as_str())
        .bind(input.pattern.clone())
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, TrackingExclusionRuleRow>(
            r#"
            SELECT
              id,
              kind,
              pattern,
              created_at,
              updated_at
            FROM tracking_exclusion_rules
            WHERE kind = ? AND pattern = ?
            "#,
        )
        .bind(input.kind.as_str())
        .bind(input.pattern)
        .fetch_one(&self.pool)
        .await?;

        row.try_into_domain()
    }

    pub async fn delete_exclusion_rule(&self, rule_id: i64) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM tracking_exclusion_rules WHERE id = ?")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct SessionRow {
    id: i64,
    started_at: String,
    ended_at: Option<String>,
    planned_focus_minutes: i32,
    actual_focus_seconds: i64,
    break_seconds: i64,
    status: String,
    preset_label: Option<String>,
    note: Option<String>,
    created_at: String,
    updated_at: String,
}

impl SessionRow {
    fn try_into_domain(self) -> Result<Session, PersistenceError> {
        Ok(Session {
            id: self.id,
            started_at: parse_datetime(&self.started_at)?,
            ended_at: self
                .ended_at
                .map(|value| parse_datetime(&value))
                .transpose()?,
            planned_focus_minutes: self.planned_focus_minutes,
            actual_focus_seconds: self.actual_focus_seconds,
            break_seconds: self.break_seconds,
            status: parse_session_status(&self.status)?,
            preset_label: self.preset_label,
            note: self.note,
            created_at: parse_datetime(&self.created_at)?,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct SessionSegmentRow {
    id: i64,
    session_id: i64,
    tracked_app_id: Option<i64>,
    kind: String,
    window_title: Option<String>,
    started_at: String,
    ended_at: String,
    duration_seconds: i64,
    created_at: String,
}

impl SessionSegmentRow {
    fn try_into_domain(self) -> Result<SessionSegment, PersistenceError> {
        Ok(SessionSegment {
            id: self.id,
            session_id: self.session_id,
            tracked_app_id: self.tracked_app_id,
            kind: parse_segment_kind(&self.kind)?,
            window_title: self.window_title,
            started_at: parse_datetime(&self.started_at)?,
            ended_at: parse_datetime(&self.ended_at)?,
            duration_seconds: self.duration_seconds,
            created_at: parse_datetime(&self.created_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct SessionAppUsageRow {
    session_id: i64,
    tracked_app_id: i64,
    duration_seconds: i64,
}

#[derive(Debug, FromRow)]
struct SessionInterruptionRow {
    session_id: i64,
    interruption_count: i64,
    interruption_seconds: i64,
}

#[derive(Debug, FromRow)]
struct TrackedAppRow {
    id: i64,
    name: String,
    executable: String,
    category: String,
    color_hex: Option<String>,
    is_excluded: bool,
    created_at: String,
    updated_at: String,
}

impl TrackedAppRow {
    fn try_into_domain(self) -> Result<TrackedApp, PersistenceError> {
        Ok(TrackedApp {
            id: self.id,
            name: self.name,
            executable: self.executable,
            category: parse_tracking_category(&self.category)?,
            color_hex: self.color_hex,
            is_excluded: self.is_excluded,
            created_at: parse_datetime(&self.created_at)?,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct TrackedWindowEventRow {
    id: i64,
    session_id: Option<i64>,
    tracked_app_id: Option<i64>,
    app_name: Option<String>,
    executable: Option<String>,
    category: Option<String>,
    window_title: Option<String>,
    started_at: String,
    ended_at: Option<String>,
    created_at: String,
}

impl TrackedWindowEventRow {
    fn try_into_domain(self) -> Result<TrackedWindowEvent, PersistenceError> {
        Ok(TrackedWindowEvent {
            id: self.id,
            session_id: self.session_id,
            tracked_app_id: self.tracked_app_id,
            app_name: self.app_name,
            executable: self.executable,
            category: self
                .category
                .as_deref()
                .map(parse_tracking_category)
                .transpose()?,
            window_title: self.window_title,
            started_at: parse_datetime(&self.started_at)?,
            ended_at: self
                .ended_at
                .map(|value| parse_datetime(&value))
                .transpose()?,
            created_at: parse_datetime(&self.created_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct TrackingExclusionRuleRow {
    id: i64,
    kind: String,
    pattern: String,
    created_at: String,
    updated_at: String,
}

impl TrackingExclusionRuleRow {
    fn try_into_domain(self) -> Result<TrackingExclusionRule, PersistenceError> {
        Ok(TrackingExclusionRule {
            id: self.id,
            kind: parse_tracking_exclusion_kind(&self.kind)?,
            pattern: self.pattern,
            created_at: parse_datetime(&self.created_at)?,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct DailyStatRow {
    stat_date: String,
    focus_seconds: i64,
    break_seconds: i64,
    completed_sessions: i32,
    interrupted_sessions: i32,
    top_app_id: Option<i64>,
    updated_at: String,
}

impl DailyStatRow {
    fn try_into_domain(self) -> Result<DailyStat, PersistenceError> {
        Ok(DailyStat {
            date: NaiveDate::parse_from_str(&self.stat_date, "%Y-%m-%d")
                .map_err(PersistenceError::InvalidDate)?,
            focus_seconds: self.focus_seconds,
            break_seconds: self.break_seconds,
            completed_sessions: self.completed_sessions,
            interrupted_sessions: self.interrupted_sessions,
            top_app_id: self.top_app_id,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

#[derive(Debug, FromRow)]
struct UserPreferenceRow {
    focus_minutes: i32,
    short_break_minutes: i32,
    long_break_minutes: i32,
    sessions_until_long_break: i32,
    auto_start_breaks: bool,
    auto_start_focus: bool,
    tracking_enabled: bool,
    tracking_permission_granted: bool,
    tracking_onboarding_completed: bool,
    notifications_enabled: bool,
    theme: String,
    updated_at: String,
}

impl UserPreferenceRow {
    fn try_into_domain(self) -> Result<UserPreference, PersistenceError> {
        Ok(UserPreference {
            focus_minutes: self.focus_minutes,
            short_break_minutes: self.short_break_minutes,
            long_break_minutes: self.long_break_minutes,
            sessions_until_long_break: self.sessions_until_long_break,
            auto_start_breaks: self.auto_start_breaks,
            auto_start_focus: self.auto_start_focus,
            tracking_enabled: self.tracking_enabled,
            tracking_permission_granted: self.tracking_permission_granted,
            tracking_onboarding_completed: self.tracking_onboarding_completed,
            notifications_enabled: self.notifications_enabled,
            theme: parse_theme_preference(&self.theme)?,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, PersistenceError> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(PersistenceError::InvalidTimestamp)?
        .with_timezone(&Utc))
}

fn parse_session_status(value: &str) -> Result<SessionStatus, PersistenceError> {
    match value {
        "planned" => Ok(SessionStatus::Planned),
        "in_progress" => Ok(SessionStatus::InProgress),
        "completed" => Ok(SessionStatus::Completed),
        "cancelled" => Ok(SessionStatus::Cancelled),
        _ => Err(PersistenceError::UnknownEnumValue(value.to_string())),
    }
}

fn parse_segment_kind(value: &str) -> Result<SessionSegmentKind, PersistenceError> {
    match value {
        "focus" => Ok(SessionSegmentKind::Focus),
        "break" => Ok(SessionSegmentKind::Break),
        "idle" => Ok(SessionSegmentKind::Idle),
        _ => Err(PersistenceError::UnknownEnumValue(value.to_string())),
    }
}

fn parse_theme_preference(value: &str) -> Result<ThemePreference, PersistenceError> {
    match value {
        "system" => Ok(ThemePreference::System),
        "light" => Ok(ThemePreference::Light),
        "dark" => Ok(ThemePreference::Dark),
        _ => Err(PersistenceError::UnknownEnumValue(value.to_string())),
    }
}

fn parse_tracking_category(value: &str) -> Result<TrackingCategory, PersistenceError> {
    match value {
        "development" => Ok(TrackingCategory::Development),
        "browser" => Ok(TrackingCategory::Browser),
        "communication" => Ok(TrackingCategory::Communication),
        "writing" => Ok(TrackingCategory::Writing),
        "design" => Ok(TrackingCategory::Design),
        "meeting" => Ok(TrackingCategory::Meeting),
        "research" => Ok(TrackingCategory::Research),
        "utilities" => Ok(TrackingCategory::Utilities),
        "unknown" => Ok(TrackingCategory::Unknown),
        _ => Err(PersistenceError::UnknownEnumValue(value.to_string())),
    }
}

fn parse_tracking_exclusion_kind(value: &str) -> Result<TrackingExclusionKind, PersistenceError> {
    match value {
        "executable" => Ok(TrackingExclusionKind::Executable),
        "window_title" => Ok(TrackingExclusionKind::WindowTitle),
        "category" => Ok(TrackingExclusionKind::Category),
        _ => Err(PersistenceError::UnknownEnumValue(value.to_string())),
    }
}

fn apply_session_history_filters(
    query: &mut QueryBuilder<'_, Sqlite>,
    filters: &SessionHistoryFiltersInput,
) {
    if let Some(date_from) = filters.date_from.clone() {
        query
            .push(" AND substr(started_at, 1, 10) >= ")
            .push_bind(date_from);
    }

    if let Some(date_to) = filters.date_to.clone() {
        query
            .push(" AND substr(started_at, 1, 10) <= ")
            .push_bind(date_to);
    }

    if let Some(min_duration_seconds) = filters.min_duration_seconds {
        query
            .push(" AND (actual_focus_seconds + break_seconds) >= ")
            .push_bind(min_duration_seconds);
    }

    if let Some(max_duration_seconds) = filters.max_duration_seconds {
        query
            .push(" AND (actual_focus_seconds + break_seconds) <= ")
            .push_bind(max_duration_seconds);
    }

    if let Some(preset_label) = filters.preset_label.clone() {
        query.push(" AND preset_label = ").push_bind(preset_label);
    }

    if let Some(status) = filters.status {
        query.push(" AND status = ").push_bind(status.as_str());
    }

    if let Some(tracked_app_id) = filters.tracked_app_id {
        query.push(
            " AND EXISTS (SELECT 1 FROM session_segments WHERE session_segments.session_id = sessions.id AND session_segments.tracked_app_id = ",
        );
        query.push_bind(tracked_app_id).push(")");
    }
}
