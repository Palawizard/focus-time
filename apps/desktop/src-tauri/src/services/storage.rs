use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use focus_domain::{
    build_gamification_overview, Achievement, BuildGamificationOverviewInput, DailyStat,
    GamificationOverview, Session, SessionSegment, SessionStatus, TrackedApp, TrackedWindowEvent,
    TrackingCategory, TrackingExclusionKind, TrackingExclusionRule, UserPreference,
};
use focus_persistence::{
    connect_database, run_migrations, seed_development_data, CreateSessionInput,
    CreateSessionSegmentInput, CreateTrackedWindowEventInput, CreateTrackingExclusionRuleInput,
    DevelopmentSeedReport, ListSessionsPageInput, RegisterTrackedAppInput, ReplaceSessionInput,
    Repositories, SaveDailyStatInput, SessionHistoryFiltersInput, UpdateSessionInput,
    UpsertTrackedAppInput,
};
use focus_stats::{BuildDashboardInput, StatsDashboard, StatsPeriod};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct StorageService {
    pool: SqlitePool,
    repositories: Repositories,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryFiltersInput {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub min_duration_seconds: Option<i64>,
    pub max_duration_seconds: Option<i64>,
    pub preset_label: Option<String>,
    pub status: Option<SessionStatus>,
    pub tracked_app_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySessionApp {
    pub tracked_app_id: i64,
    pub name: String,
    pub executable: String,
    pub category: TrackingCategory,
    pub color_hex: Option<String>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySessionSummary {
    pub session: Session,
    pub total_duration_seconds: i64,
    pub tracked_apps: Vec<HistorySessionApp>,
    pub interruption_count: i64,
    pub interruption_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySessionsPage {
    pub items: Vec<HistorySessionSummary>,
    pub next_offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegmentDetail {
    pub segment: SessionSegment,
    pub tracked_app: Option<TrackedApp>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySessionDetail {
    pub session: Session,
    pub total_duration_seconds: i64,
    pub tracked_apps: Vec<HistorySessionApp>,
    pub segments: Vec<SessionSegmentDetail>,
    pub tracked_window_events: Vec<TrackedWindowEvent>,
    pub interruption_count: i64,
    pub interruption_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct ReplaceSessionDetailsInput {
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

#[derive(Debug, Clone, Copy)]
pub enum HistoryExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryExportResult {
    pub path: String,
    pub format: &'static str,
    pub sessions_exported: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupArchiveSummary {
    pub file_name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupArchive {
    format_version: u32,
    created_at: DateTime<Utc>,
    schema_version: u32,
    payload: BackupPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupPayload {
    user_preferences: UserPreference,
    sessions: Vec<Session>,
    session_segments: Vec<SessionSegment>,
    tracked_apps: Vec<TrackedApp>,
    tracked_window_events: Vec<TrackedWindowEvent>,
    tracking_exclusion_rules: Vec<TrackingExclusionRule>,
    daily_stats: Vec<DailyStat>,
    achievements: Vec<Achievement>,
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

    pub async fn list_history_sessions(
        &self,
        limit: u32,
        offset: u32,
        filters: HistoryFiltersInput,
    ) -> anyhow::Result<HistorySessionsPage> {
        let sessions = self
            .repositories
            .sessions
            .list_filtered(ListSessionsPageInput {
                limit: Some(limit),
                offset,
                filters: filters.clone().into_persistence(),
            })
            .await?;

        let items = self.build_history_summaries(sessions).await?;
        let next_offset = (items.len() as u32 == limit).then_some(offset + limit);

        Ok(HistorySessionsPage { items, next_offset })
    }

    pub async fn get_history_session_detail(
        &self,
        session_id: i64,
    ) -> anyhow::Result<HistorySessionDetail> {
        let session = self.repositories.sessions.get_by_id(session_id).await?;
        let summaries = self
            .build_history_summaries(vec![session.clone()])
            .await?
            .into_iter()
            .next()
            .context("failed to assemble session summary")?;
        let segments = self.repositories.sessions.list_segments(session_id).await?;
        let tracked_apps = self.repositories.tracked_apps.list().await?;
        let tracked_apps_by_id = tracked_apps
            .into_iter()
            .map(|tracked_app| (tracked_app.id, tracked_app))
            .collect::<HashMap<_, _>>();
        let segment_details = segments
            .into_iter()
            .map(|segment| SessionSegmentDetail {
                tracked_app: segment
                    .tracked_app_id
                    .and_then(|tracked_app_id| tracked_apps_by_id.get(&tracked_app_id).cloned()),
                segment,
            })
            .collect();
        let tracked_window_events = self
            .repositories
            .tracking
            .list_window_events_for_session(session_id)
            .await?;

        Ok(HistorySessionDetail {
            session,
            total_duration_seconds: summaries.total_duration_seconds,
            tracked_apps: summaries.tracked_apps,
            segments: segment_details,
            tracked_window_events,
            interruption_count: summaries.interruption_count,
            interruption_seconds: summaries.interruption_seconds,
        })
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
        ended_at: DateTime<Utc>,
        actual_focus_seconds: i64,
        break_seconds: i64,
        status: SessionStatus,
    ) -> anyhow::Result<Session> {
        let session = self
            .repositories
            .sessions
            .update(UpdateSessionInput {
                session_id,
                ended_at: Some(ended_at),
                actual_focus_seconds,
                break_seconds,
                status,
            })
            .await
            .map_err(anyhow::Error::from)?;
        self.refresh_gamification_state().await?;

        Ok(session)
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
        let preferences = self
            .repositories
            .preferences
            .save(preferences)
            .await
            .map_err(anyhow::Error::from)?;
        self.refresh_gamification_state().await?;

        Ok(preferences)
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

    pub async fn replace_session(
        &self,
        input: ReplaceSessionDetailsInput,
    ) -> anyhow::Result<Session> {
        let session = self
            .repositories
            .sessions
            .replace(ReplaceSessionInput {
                session_id: input.session_id,
                started_at: input.started_at,
                ended_at: input.ended_at,
                planned_focus_minutes: input.planned_focus_minutes,
                actual_focus_seconds: input.actual_focus_seconds,
                break_seconds: input.break_seconds,
                status: input.status,
                preset_label: input.preset_label,
                note: input.note,
            })
            .await
            .map_err(anyhow::Error::from)?;
        self.refresh_gamification_state().await?;

        Ok(session)
    }

    pub async fn delete_session(&self, session_id: i64) -> anyhow::Result<()> {
        self.repositories
            .tracking
            .delete_window_events_for_session(session_id)
            .await?;
        self.repositories.sessions.delete(session_id).await?;

        Ok(())
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
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
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

    pub async fn get_stats_dashboard(&self, period: StatsPeriod) -> anyhow::Result<StatsDashboard> {
        let today = Utc::now().date_naive();
        let range = focus_stats::resolve_range(period, today, today);
        let sessions = self
            .repositories
            .sessions
            .list_filtered(ListSessionsPageInput {
                limit: None,
                offset: 0,
                filters: SessionHistoryFiltersInput {
                    date_to: Some(range.end_date.format("%Y-%m-%d").to_string()),
                    ..SessionHistoryFiltersInput::default()
                },
            })
            .await?;
        let current_session_ids = sessions
            .iter()
            .filter(|session| {
                let date = session.started_at.date_naive();
                date >= range.start_date && date <= range.end_date
            })
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let current_segments = self
            .repositories
            .sessions
            .list_segments_for_sessions(&current_session_ids)
            .await?;
        let tracked_apps = self.repositories.tracked_apps.list().await?;

        Ok(focus_stats::build_dashboard(BuildDashboardInput {
            period,
            anchor_date: today,
            today,
            sessions,
            current_segments,
            tracked_apps,
        }))
    }

    pub async fn get_gamification_overview(&self) -> anyhow::Result<GamificationOverview> {
        let today = Utc::now().date_naive();
        let sessions = self
            .repositories
            .sessions
            .list_filtered(ListSessionsPageInput {
                limit: None,
                offset: 0,
                filters: SessionHistoryFiltersInput::default(),
            })
            .await?;
        let preferences = self.repositories.preferences.get().await?;
        let unlocked_achievements = self.repositories.achievements.list().await?;
        let unlocked_slugs = unlocked_achievements
            .iter()
            .filter(|achievement| achievement.unlocked_at.is_some())
            .map(|achievement| achievement.slug.clone())
            .collect::<HashSet<_>>();
        let overview = build_gamification_overview(BuildGamificationOverviewInput {
            today,
            sessions: sessions.clone(),
            preferences: preferences.clone(),
            unlocked_achievements,
        });

        if self
            .persist_unlocked_achievements(&overview, &unlocked_slugs)
            .await?
        {
            let refreshed_achievements = self.repositories.achievements.list().await?;

            return Ok(build_gamification_overview(
                BuildGamificationOverviewInput {
                    today,
                    sessions,
                    preferences,
                    unlocked_achievements: refreshed_achievements,
                },
            ));
        }

        Ok(overview)
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

    pub async fn export_history(
        &self,
        export_root: PathBuf,
        format: HistoryExportFormat,
        filters: HistoryFiltersInput,
    ) -> anyhow::Result<HistoryExportResult> {
        let sessions = self
            .repositories
            .sessions
            .list_filtered(ListSessionsPageInput {
                limit: None,
                offset: 0,
                filters: filters.into_persistence(),
            })
            .await?;
        let summaries = self.build_history_summaries(sessions).await?;

        fs::create_dir_all(&export_root)?;

        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let (extension, format_label, content) = match format {
            HistoryExportFormat::Csv => ("csv", "csv", build_history_csv(&summaries)),
            HistoryExportFormat::Json => {
                ("json", "json", serde_json::to_string_pretty(&summaries)?)
            }
        };
        let path = export_root.join(format!("focus-time-history-{timestamp}.{extension}"));
        fs::write(&path, content)?;

        Ok(HistoryExportResult {
            path: path.display().to_string(),
            format: format_label,
            sessions_exported: summaries.len(),
        })
    }

    pub async fn create_backup(
        &self,
        backup_root: PathBuf,
    ) -> anyhow::Result<BackupArchiveSummary> {
        fs::create_dir_all(&backup_root)?;

        let preferences = self.repositories.preferences.get().await?;
        let sessions = self
            .repositories
            .sessions
            .list_filtered(ListSessionsPageInput {
                limit: None,
                offset: 0,
                filters: SessionHistoryFiltersInput::default(),
            })
            .await?;
        let session_ids = sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let session_segments = self
            .repositories
            .sessions
            .list_segments_for_sessions(&session_ids)
            .await?;
        let tracked_apps = self.repositories.tracked_apps.list().await?;
        let tracked_window_events = self.repositories.tracking.list_all_window_events().await?;
        let tracking_exclusion_rules = self.repositories.tracking.list_exclusion_rules().await?;
        let daily_stats = self.repositories.daily_stats.list_all().await?;
        let achievements = self.repositories.achievements.list().await?;

        let archive = BackupArchive {
            format_version: 1,
            created_at: Utc::now(),
            schema_version: focus_persistence::storage_profile().schema_version,
            payload: BackupPayload {
                user_preferences: preferences,
                sessions,
                session_segments,
                tracked_apps,
                tracked_window_events,
                tracking_exclusion_rules,
                daily_stats,
                achievements,
            },
        };
        let file_name = format!(
            "focus-time-backup-{}.json",
            archive.created_at.format("%Y%m%d-%H%M%S")
        );
        let path = backup_root.join(file_name);
        fs::write(&path, serde_json::to_string_pretty(&archive)?)?;
        log::info!("Created local backup at {}", path.display());

        summarize_backup_file(&path)
    }

    pub async fn list_backups(
        &self,
        backup_root: PathBuf,
    ) -> anyhow::Result<Vec<BackupArchiveSummary>> {
        fs::create_dir_all(&backup_root)?;
        let mut backups = fs::read_dir(&backup_root)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .filter_map(|path| summarize_backup_file(&path).ok())
            .collect::<Vec<_>>();

        backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));

        Ok(backups)
    }

    pub async fn restore_backup(
        &self,
        backup_path: PathBuf,
    ) -> anyhow::Result<BackupArchiveSummary> {
        let archive = read_backup_archive(&backup_path)?;
        let mut transaction = self.pool.begin().await?;

        sqlx::query("DELETE FROM tracked_window_events")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM session_segments")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM daily_stats")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM tracking_exclusion_rules")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM achievements")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM tracked_apps")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM sessions")
            .execute(&mut *transaction)
            .await?;

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
              sound_enabled = ?,
              weekly_focus_goal_minutes = ?,
              weekly_completed_sessions_goal = ?,
              launch_on_startup = ?,
              tray_enabled = ?,
              close_to_tray = ?,
              theme = ?,
              updated_at = ?
            WHERE id = 1
            "#,
        )
        .bind(archive.payload.user_preferences.focus_minutes)
        .bind(archive.payload.user_preferences.short_break_minutes)
        .bind(archive.payload.user_preferences.long_break_minutes)
        .bind(archive.payload.user_preferences.sessions_until_long_break)
        .bind(archive.payload.user_preferences.auto_start_breaks)
        .bind(archive.payload.user_preferences.auto_start_focus)
        .bind(archive.payload.user_preferences.tracking_enabled)
        .bind(archive.payload.user_preferences.tracking_permission_granted)
        .bind(
            archive
                .payload
                .user_preferences
                .tracking_onboarding_completed,
        )
        .bind(archive.payload.user_preferences.notifications_enabled)
        .bind(archive.payload.user_preferences.sound_enabled)
        .bind(archive.payload.user_preferences.weekly_focus_goal_minutes)
        .bind(
            archive
                .payload
                .user_preferences
                .weekly_completed_sessions_goal,
        )
        .bind(archive.payload.user_preferences.launch_on_startup)
        .bind(archive.payload.user_preferences.tray_enabled)
        .bind(archive.payload.user_preferences.close_to_tray)
        .bind(archive.payload.user_preferences.theme.as_str())
        .bind(archive.payload.user_preferences.updated_at.to_rfc3339())
        .execute(&mut *transaction)
        .await?;

        for session in &archive.payload.sessions {
            sqlx::query(
                r#"
                INSERT INTO sessions (
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
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(session.id)
            .bind(session.started_at.to_rfc3339())
            .bind(session.ended_at.map(|value| value.to_rfc3339()))
            .bind(session.planned_focus_minutes)
            .bind(session.actual_focus_seconds)
            .bind(session.break_seconds)
            .bind(session.status.as_str())
            .bind(session.preset_label.clone())
            .bind(session.note.clone())
            .bind(session.created_at.to_rfc3339())
            .bind(session.updated_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for tracked_app in &archive.payload.tracked_apps {
            sqlx::query(
                r#"
                INSERT INTO tracked_apps (
                  id,
                  name,
                  executable,
                  category,
                  color_hex,
                  is_excluded,
                  created_at,
                  updated_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(tracked_app.id)
            .bind(tracked_app.name.clone())
            .bind(tracked_app.executable.clone())
            .bind(tracked_app.category.as_str())
            .bind(tracked_app.color_hex.clone())
            .bind(tracked_app.is_excluded)
            .bind(tracked_app.created_at.to_rfc3339())
            .bind(tracked_app.updated_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for segment in &archive.payload.session_segments {
            sqlx::query(
                r#"
                INSERT INTO session_segments (
                  id,
                  session_id,
                  tracked_app_id,
                  kind,
                  window_title,
                  started_at,
                  ended_at,
                  duration_seconds,
                  created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(segment.id)
            .bind(segment.session_id)
            .bind(segment.tracked_app_id)
            .bind(segment.kind.as_str())
            .bind(segment.window_title.clone())
            .bind(segment.started_at.to_rfc3339())
            .bind(segment.ended_at.to_rfc3339())
            .bind(segment.duration_seconds)
            .bind(segment.created_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for event in &archive.payload.tracked_window_events {
            sqlx::query(
                r#"
                INSERT INTO tracked_window_events (
                  id,
                  session_id,
                  tracked_app_id,
                  window_title,
                  started_at,
                  ended_at,
                  created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(event.id)
            .bind(event.session_id)
            .bind(event.tracked_app_id)
            .bind(event.window_title.clone())
            .bind(event.started_at.to_rfc3339())
            .bind(event.ended_at.map(|value| value.to_rfc3339()))
            .bind(event.created_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for rule in &archive.payload.tracking_exclusion_rules {
            sqlx::query(
                r#"
                INSERT INTO tracking_exclusion_rules (
                  id,
                  kind,
                  pattern,
                  created_at,
                  updated_at
                )
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(rule.id)
            .bind(rule.kind.as_str())
            .bind(rule.pattern.clone())
            .bind(rule.created_at.to_rfc3339())
            .bind(rule.updated_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for stat in &archive.payload.daily_stats {
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
                "#,
            )
            .bind(stat.date.format("%Y-%m-%d").to_string())
            .bind(stat.focus_seconds)
            .bind(stat.break_seconds)
            .bind(stat.completed_sessions)
            .bind(stat.interrupted_sessions)
            .bind(stat.top_app_id)
            .bind(stat.updated_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        for achievement in &archive.payload.achievements {
            sqlx::query(
                r#"
                INSERT INTO achievements (
                  id,
                  slug,
                  title,
                  unlocked_at,
                  created_at
                )
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(achievement.id)
            .bind(achievement.slug.clone())
            .bind(achievement.title.clone())
            .bind(achievement.unlocked_at.map(|value| value.to_rfc3339()))
            .bind(achievement.created_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        self.refresh_gamification_state().await?;
        log::info!("Restored local backup from {}", backup_path.display());

        summarize_backup_file(&backup_path)
    }

    async fn build_history_summaries(
        &self,
        sessions: Vec<Session>,
    ) -> anyhow::Result<Vec<HistorySessionSummary>> {
        if sessions.is_empty() {
            return Ok(Vec::new());
        }

        let session_ids = sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let app_usage = self
            .repositories
            .sessions
            .list_app_usage(&session_ids)
            .await?;
        let interruptions = self
            .repositories
            .sessions
            .list_interruptions(&session_ids)
            .await?;
        let tracked_apps = self.repositories.tracked_apps.list().await?;
        let tracked_apps_by_id = tracked_apps
            .into_iter()
            .map(|tracked_app| (tracked_app.id, tracked_app))
            .collect::<HashMap<_, _>>();

        let mut app_usage_by_session = HashMap::<i64, Vec<HistorySessionApp>>::new();
        for usage in app_usage {
            let Some(tracked_app) = tracked_apps_by_id.get(&usage.tracked_app_id) else {
                continue;
            };

            app_usage_by_session
                .entry(usage.session_id)
                .or_default()
                .push(HistorySessionApp {
                    tracked_app_id: tracked_app.id,
                    name: tracked_app.name.clone(),
                    executable: tracked_app.executable.clone(),
                    category: tracked_app.category,
                    color_hex: tracked_app.color_hex.clone(),
                    duration_seconds: usage.duration_seconds,
                });
        }

        let interruption_by_session = interruptions
            .into_iter()
            .map(|item| (item.session_id, item))
            .collect::<HashMap<_, _>>();

        Ok(sessions
            .into_iter()
            .map(|session| {
                let total_duration_seconds =
                    session.actual_focus_seconds.max(0) + session.break_seconds.max(0);
                let interruption = interruption_by_session.get(&session.id);

                HistorySessionSummary {
                    tracked_apps: app_usage_by_session.remove(&session.id).unwrap_or_default(),
                    total_duration_seconds,
                    interruption_count: interruption
                        .map(|value| value.interruption_count)
                        .unwrap_or_default(),
                    interruption_seconds: interruption
                        .map(|value| value.interruption_seconds)
                        .unwrap_or_default(),
                    session,
                }
            })
            .collect())
    }
}

impl HistoryFiltersInput {
    fn into_persistence(self) -> SessionHistoryFiltersInput {
        SessionHistoryFiltersInput {
            date_from: self.date_from,
            date_to: self.date_to,
            min_duration_seconds: self.min_duration_seconds,
            max_duration_seconds: self.max_duration_seconds,
            preset_label: self.preset_label,
            status: self.status,
            tracked_app_id: self.tracked_app_id,
        }
    }
}

impl StorageService {
    async fn refresh_gamification_state(&self) -> anyhow::Result<()> {
        let _ = self.get_gamification_overview().await?;

        Ok(())
    }

    async fn persist_unlocked_achievements(
        &self,
        overview: &GamificationOverview,
        unlocked_slugs: &HashSet<String>,
    ) -> anyhow::Result<bool> {
        let mut has_new_unlocks = false;

        for achievement in &overview.achievements {
            let Some(unlocked_at) = achievement.unlocked_at else {
                continue;
            };

            if unlocked_slugs.contains(&achievement.slug) {
                continue;
            }

            self.repositories
                .achievements
                .unlock(&achievement.slug, &achievement.title, unlocked_at)
                .await?;
            has_new_unlocks = true;
        }

        Ok(has_new_unlocks)
    }
}

fn build_history_csv(items: &[HistorySessionSummary]) -> String {
    let mut lines = vec![
        "id,started_at,ended_at,status,preset_label,planned_focus_minutes,actual_focus_seconds,break_seconds,total_duration_seconds,interruption_count,interruption_seconds,apps,note".to_string(),
    ];

    for item in items {
        let apps = item
            .tracked_apps
            .iter()
            .map(|tracked_app| format!("{} ({})", tracked_app.name, tracked_app.duration_seconds))
            .collect::<Vec<_>>()
            .join(" | ");

        lines.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            item.session.id,
            escape_csv(&item.session.started_at.to_rfc3339()),
            escape_csv(
                &item
                    .session
                    .ended_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_default()
            ),
            escape_csv(item.session.status.as_str()),
            escape_csv(item.session.preset_label.as_deref().unwrap_or_default()),
            item.session.planned_focus_minutes,
            item.session.actual_focus_seconds,
            item.session.break_seconds,
            item.total_duration_seconds,
            item.interruption_count,
            item.interruption_seconds,
            escape_csv(&apps),
            escape_csv(item.session.note.as_deref().unwrap_or_default()),
        ));
    }

    lines.join("\n")
}

fn escape_csv(value: &str) -> String {
    let needs_quotes = value.contains(',') || value.contains('"') || value.contains('\n');
    let escaped = value.replace('"', "\"\"");

    if needs_quotes {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

fn summarize_backup_file(path: &PathBuf) -> anyhow::Result<BackupArchiveSummary> {
    let archive = read_backup_archive(path)?;
    let metadata = fs::metadata(path)?;

    Ok(BackupArchiveSummary {
        file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .context("backup file name should be valid unicode")?
            .to_string(),
        path: path.display().to_string(),
        created_at: archive.created_at,
        size_bytes: metadata.len(),
    })
}

fn read_backup_archive(path: &PathBuf) -> anyhow::Result<BackupArchive> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read backup archive at {}", path.display()))?;
    let archive = serde_json::from_str::<BackupArchive>(&contents)
        .with_context(|| format!("failed to parse backup archive at {}", path.display()))?;

    Ok(archive)
}
