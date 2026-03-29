use chrono::{DateTime, NaiveDate, Utc};
use focus_domain::{
    DailyStat, Session, SessionSegment, SessionSegmentKind, SessionStatus, ThemePreference,
    TrackedApp, UserPreference,
};
use sqlx::{FromRow, SqlitePool};

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
pub struct Repositories {
    pub sessions: SessionRepository,
    pub preferences: PreferencesRepository,
    pub tracked_apps: TrackedAppRepository,
    pub daily_stats: DailyStatRepository,
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
    pub color_hex: Option<String>,
    pub is_excluded: bool,
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

impl Repositories {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            sessions: SessionRepository::new(pool.clone()),
            preferences: PreferencesRepository::new(pool.clone()),
            tracked_apps: TrackedAppRepository::new(pool.clone()),
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

    pub async fn save(&self, preferences: &UserPreference) -> Result<UserPreference, PersistenceError> {
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

        rows.into_iter().map(TrackedAppRow::try_into_domain).collect()
    }

    pub async fn upsert(
        &self,
        input: UpsertTrackedAppInput,
    ) -> Result<TrackedApp, PersistenceError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO tracked_apps (name, executable, color_hex, is_excluded, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(executable) DO UPDATE SET
              name = excluded.name,
              color_hex = excluded.color_hex,
              is_excluded = excluded.is_excluded,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(input.name)
        .bind(input.executable.clone())
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

        rows.into_iter().map(DailyStatRow::try_into_domain).collect()
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
            ended_at: self.ended_at.map(|value| parse_datetime(&value)).transpose()?,
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
struct TrackedAppRow {
    id: i64,
    name: String,
    executable: String,
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
            color_hex: self.color_hex,
            is_excluded: self.is_excluded,
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
