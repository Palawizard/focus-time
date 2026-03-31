use std::path::PathBuf;

use anyhow::Context;
use chrono::{NaiveDate, Utc};
use focus_domain::{
    DailyStat, Session, SessionSegment, SessionStatus, TrackedApp, TrackedWindowEvent,
    TrackingCategory, TrackingExclusionKind, TrackingExclusionRule, UserPreference,
};
use focus_persistence::{
    connect_database, run_migrations, seed_development_data, CreateSessionInput,
    CreateSessionSegmentInput, CreateTrackedWindowEventInput, CreateTrackingExclusionRuleInput,
    DevelopmentSeedReport, RegisterTrackedAppInput, Repositories, SaveDailyStatInput,
    UpdateSessionInput, UpsertTrackedAppInput,
};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct StorageService {
    pool: SqlitePool,
    repositories: Repositories,
}

impl StorageService {
    pub async fn new(database_path: PathBuf) -> anyhow::Result<Self> {
        let pool = connect_database(&database_path).await?;
        run_migrations(&pool).await?;
        let repositories = Repositories::new(pool.clone());

        Ok(Self { pool, repositories })
    }

    pub async fn list_sessions(&self, limit: u32) -> anyhow::Result<Vec<Session>> {
        self.repositories
            .sessions
            .list(limit)
            .await
            .map_err(Into::into)
    }

    pub async fn create_session(
        &self,
        planned_focus_minutes: i32,
        status: SessionStatus,
        preset_label: Option<String>,
        note: Option<String>,
    ) -> anyhow::Result<Session> {
        self.repositories
            .sessions
            .create(CreateSessionInput {
                started_at: Utc::now(),
                planned_focus_minutes,
                status,
                preset_label,
                note,
            })
            .await
            .map_err(Into::into)
    }

    pub async fn update_session(
        &self,
        session_id: i64,
        ended_at: chrono::DateTime<Utc>,
        actual_focus_seconds: i64,
        break_seconds: i64,
        status: SessionStatus,
    ) -> anyhow::Result<Session> {
        self.repositories
            .sessions
            .update(UpdateSessionInput {
                session_id,
                ended_at: Some(ended_at),
                actual_focus_seconds,
                break_seconds,
                status,
            })
            .await
            .map_err(Into::into)
    }

    pub async fn list_session_segments(
        &self,
        session_id: i64,
    ) -> anyhow::Result<Vec<SessionSegment>> {
        self.repositories
            .sessions
            .list_segments(session_id)
            .await
            .map_err(Into::into)
    }

    pub async fn create_session_segment(
        &self,
        input: CreateSessionSegmentInput,
    ) -> anyhow::Result<SessionSegment> {
        self.repositories
            .sessions
            .add_segment(input)
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_preferences(&self) -> anyhow::Result<UserPreference> {
        self.repositories
            .preferences
            .get()
            .await
            .map_err(Into::into)
    }

    pub async fn save_user_preferences(
        &self,
        preferences: &UserPreference,
    ) -> anyhow::Result<UserPreference> {
        self.repositories
            .preferences
            .save(preferences)
            .await
            .map_err(Into::into)
    }

    pub async fn list_tracked_apps(&self) -> anyhow::Result<Vec<TrackedApp>> {
        self.repositories
            .tracked_apps
            .list()
            .await
            .map_err(Into::into)
    }

    pub async fn upsert_tracked_app(
        &self,
        input: UpsertTrackedAppInput,
    ) -> anyhow::Result<TrackedApp> {
        self.repositories
            .tracked_apps
            .upsert(input)
            .await
            .map_err(Into::into)
    }

    pub async fn register_seen_tracked_app(
        &self,
        name: String,
        executable: String,
        category: TrackingCategory,
        color_hex: Option<String>,
    ) -> anyhow::Result<TrackedApp> {
        self.repositories
            .tracked_apps
            .register_seen(RegisterTrackedAppInput {
                name,
                executable,
                category,
                color_hex,
            })
            .await
            .map_err(Into::into)
    }

    pub async fn list_tracked_window_events(
        &self,
        limit: u32,
    ) -> anyhow::Result<Vec<TrackedWindowEvent>> {
        self.repositories
            .tracking
            .list_window_events(limit)
            .await
            .map_err(Into::into)
    }

    pub async fn create_tracked_window_event(
        &self,
        session_id: Option<i64>,
        tracked_app_id: Option<i64>,
        window_title: Option<String>,
        started_at: chrono::DateTime<Utc>,
        ended_at: Option<chrono::DateTime<Utc>>,
    ) -> anyhow::Result<TrackedWindowEvent> {
        self.repositories
            .tracking
            .create_window_event(CreateTrackedWindowEventInput {
                session_id,
                tracked_app_id,
                window_title,
                started_at,
                ended_at,
            })
            .await
            .map_err(Into::into)
    }

    pub async fn list_tracking_exclusion_rules(
        &self,
    ) -> anyhow::Result<Vec<TrackingExclusionRule>> {
        self.repositories
            .tracking
            .list_exclusion_rules()
            .await
            .map_err(Into::into)
    }

    pub async fn create_tracking_exclusion_rule(
        &self,
        kind: TrackingExclusionKind,
        pattern: String,
    ) -> anyhow::Result<TrackingExclusionRule> {
        self.repositories
            .tracking
            .create_exclusion_rule(CreateTrackingExclusionRuleInput { kind, pattern })
            .await
            .map_err(Into::into)
    }

    pub async fn delete_tracking_exclusion_rule(&self, rule_id: i64) -> anyhow::Result<()> {
        self.repositories
            .tracking
            .delete_exclusion_rule(rule_id)
            .await
            .map_err(Into::into)
    }

    pub async fn list_daily_stats(&self, limit: u32) -> anyhow::Result<Vec<DailyStat>> {
        self.repositories
            .daily_stats
            .list(limit)
            .await
            .map_err(Into::into)
    }

    pub async fn upsert_daily_stat(
        &self,
        date: NaiveDate,
        focus_seconds: i64,
        break_seconds: i64,
        completed_sessions: i32,
        interrupted_sessions: i32,
        top_app_id: Option<i64>,
    ) -> anyhow::Result<DailyStat> {
        self.repositories
            .daily_stats
            .upsert(SaveDailyStatInput {
                date,
                focus_seconds,
                break_seconds,
                completed_sessions,
                interrupted_sessions,
                top_app_id,
            })
            .await
            .map_err(Into::into)
    }

    pub async fn seed_development_data(&self) -> anyhow::Result<DevelopmentSeedReport> {
        seed_development_data(&self.pool).await.map_err(Into::into)
    }

    pub async fn ensure_ready(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .context("failed to validate the sqlite connection")?;

        Ok(())
    }
}
