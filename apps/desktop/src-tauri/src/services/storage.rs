use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;
use chrono::{NaiveDate, Utc};
use focus_domain::{
    DailyStat, Session, SessionSegment, SessionStatus, TrackedApp, TrackedWindowEvent,
    TrackingCategory, TrackingExclusionKind, TrackingExclusionRule, UserPreference,
};
use focus_persistence::{
    connect_database, run_migrations, seed_development_data, CreateSessionInput,
    CreateSessionSegmentInput, CreateTrackedWindowEventInput, CreateTrackingExclusionRuleInput,
    DevelopmentSeedReport, ListSessionsPageInput, RegisterTrackedAppInput, ReplaceSessionInput,
    Repositories, SaveDailyStatInput, SessionHistoryFiltersInput, UpdateSessionInput,
    UpsertTrackedAppInput,
};
use serde::Serialize;
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
    pub started_at: chrono::DateTime<Utc>,
    pub ended_at: Option<chrono::DateTime<Utc>>,
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

    pub async fn replace_session(
        &self,
        input: ReplaceSessionDetailsInput,
    ) -> anyhow::Result<Session> {
        self.repositories
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
            .map_err(Into::into)
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
