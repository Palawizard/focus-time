use std::sync::Arc;

use chrono::Utc;
use focus_domain::{
    PomodoroControlState, PomodoroPhase, PomodoroSnapshot, SessionSegmentKind, TrackingCategory,
    TrackingExclusionKind, TrackingExclusionRule, UserPreference,
};
use focus_persistence::CreateSessionSegmentInput;
use focus_tracking::{
    capture_active_window, normalize_executable_name, ActiveWindowSample, TrackingCapability,
    TrackingStatus,
};
use serde::Serialize;
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};

use super::StorageService;

#[derive(Debug, Clone)]
pub struct TrackingService {
    runtime: Arc<Mutex<TrackingRuntime>>,
    storage: StorageService,
}

#[derive(Debug)]
struct TrackingRuntime {
    context: TrackingContext,
    snapshot: TrackingRuntimeSnapshot,
    open_sample: Option<OpenTrackedSample>,
}

#[derive(Debug, Clone, Default)]
struct TrackingContext {
    active_session_id: Option<i64>,
    should_sample: bool,
}

#[derive(Debug, Clone)]
struct OpenTrackedSample {
    session_id: i64,
    tracked_app_id: Option<i64>,
    sample: ActiveWindowSample,
    excluded: bool,
    started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingRuntimeSnapshot {
    pub status: TrackingStatus,
    pub tracking_enabled: bool,
    pub permission_granted: bool,
    pub onboarding_completed: bool,
    pub active_session_id: Option<i64>,
    pub active_window: Option<ActiveWindowSample>,
    pub last_error: Option<String>,
    pub is_tracking_live: bool,
}

impl TrackingService {
    pub fn new(storage: StorageService) -> Self {
        let runtime = Arc::new(Mutex::new(TrackingRuntime {
            context: TrackingContext::default(),
            snapshot: TrackingRuntimeSnapshot {
                status: focus_tracking::tracking_status(),
                tracking_enabled: false,
                permission_granted: false,
                onboarding_completed: false,
                active_session_id: None,
                active_window: None,
                last_error: None,
                is_tracking_live: false,
            },
            open_sample: None,
        }));

        spawn_tracking_loop(runtime.clone(), storage.clone());

        Self { runtime, storage }
    }

    pub async fn get_status(&self) -> TrackingRuntimeSnapshot {
        self.runtime.lock().await.snapshot.clone()
    }

    pub async fn sync_pomodoro_state(&self, state: PomodoroSnapshot) {
        let should_sample = state.control_state == PomodoroControlState::Running
            && matches!(state.phase, Some(PomodoroPhase::Focus));

        let mut runtime = self.runtime.lock().await;
        runtime.context = TrackingContext {
            active_session_id: state.session_id,
            should_sample,
        };
        runtime.snapshot.active_session_id = state.session_id;
        runtime.snapshot.is_tracking_live = should_sample && runtime.snapshot.is_tracking_live;
    }

    pub async fn list_recent_events(
        &self,
        limit: u32,
    ) -> anyhow::Result<Vec<focus_domain::TrackedWindowEvent>> {
        self.storage.list_tracked_window_events(limit).await
    }

    pub async fn list_exclusion_rules(&self) -> anyhow::Result<Vec<TrackingExclusionRule>> {
        self.storage.list_tracking_exclusion_rules().await
    }

    pub async fn create_exclusion_rule(
        &self,
        kind: TrackingExclusionKind,
        pattern: String,
    ) -> anyhow::Result<TrackingExclusionRule> {
        self.storage
            .create_tracking_exclusion_rule(kind, normalize_rule_pattern(kind, &pattern))
            .await
    }

    pub async fn delete_exclusion_rule(&self, rule_id: i64) -> anyhow::Result<()> {
        self.storage.delete_tracking_exclusion_rule(rule_id).await
    }
}

fn spawn_tracking_loop(runtime: Arc<Mutex<TrackingRuntime>>, storage: StorageService) {
    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            let status = focus_tracking::tracking_status();
            let context = {
                let runtime = runtime.lock().await;
                runtime.context.clone()
            };

            let preferences = match storage.get_user_preferences().await {
                Ok(preferences) => preferences,
                Err(error) => {
                    update_runtime_snapshot(
                        &runtime,
                        status,
                        None,
                        context.active_session_id,
                        None,
                        Some(error.to_string()),
                        false,
                    )
                    .await;
                    continue;
                }
            };

            let can_track = should_track(&status, &preferences, &context);

            if !can_track {
                if let Err(error) = flush_open_sample(&runtime, &storage).await {
                    update_runtime_snapshot(
                        &runtime,
                        status,
                        Some(&preferences),
                        context.active_session_id,
                        None,
                        Some(error.to_string()),
                        false,
                    )
                    .await;
                    continue;
                }

                update_runtime_snapshot(
                    &runtime,
                    status,
                    Some(&preferences),
                    context.active_session_id,
                    None,
                    None,
                    false,
                )
                .await;
                continue;
            }

            match capture_active_window() {
                Ok(Some(sample)) => {
                    let live_sample =
                        match process_sample(&runtime, &storage, &context, sample.clone()).await {
                            Ok(live_sample) => live_sample,
                            Err(error) => {
                                update_runtime_snapshot(
                                    &runtime,
                                    status,
                                    Some(&preferences),
                                    context.active_session_id,
                                    None,
                                    Some(error.to_string()),
                                    false,
                                )
                                .await;
                                continue;
                            }
                        };

                    update_runtime_snapshot(
                        &runtime,
                        status,
                        Some(&preferences),
                        context.active_session_id,
                        live_sample,
                        None,
                        true,
                    )
                    .await;
                }
                Ok(None) => {
                    let flush_result = flush_open_sample(&runtime, &storage).await;
                    update_runtime_snapshot(
                        &runtime,
                        status,
                        Some(&preferences),
                        context.active_session_id,
                        None,
                        flush_result.err().map(|error| error.to_string()),
                        false,
                    )
                    .await;
                }
                Err(error) => {
                    let _ = flush_open_sample(&runtime, &storage).await;
                    update_runtime_snapshot(
                        &runtime,
                        status,
                        Some(&preferences),
                        context.active_session_id,
                        None,
                        Some(error.to_string()),
                        false,
                    )
                    .await;
                }
            }
        }
    });
}

async fn process_sample(
    runtime: &Arc<Mutex<TrackingRuntime>>,
    storage: &StorageService,
    context: &TrackingContext,
    sample: ActiveWindowSample,
) -> anyhow::Result<Option<ActiveWindowSample>> {
    let session_id = match context.active_session_id {
        Some(session_id) => session_id,
        None => {
            flush_open_sample(runtime, storage).await?;
            return Ok(None);
        }
    };

    let tracked_app = storage
        .register_seen_tracked_app(
            sample.app_name.clone(),
            sample.executable.clone(),
            sample.category,
            Some(category_color(sample.category).to_string()),
        )
        .await?;
    let exclusion_rules = storage.list_tracking_exclusion_rules().await?;
    let excluded = tracked_app.is_excluded || matches_exclusion(&sample, &exclusion_rules);

    let mut runtime_lock = runtime.lock().await;
    let now = Utc::now();
    let sample_changed = runtime_lock
        .open_sample
        .as_ref()
        .map(|current| {
            current.session_id != session_id
                || current.excluded != excluded
                || current.sample.executable != sample.executable
                || current.sample.window_title != sample.window_title
        })
        .unwrap_or(true);

    if sample_changed {
        let previous = runtime_lock.open_sample.take();
        drop(runtime_lock);

        if let Some(previous) = previous {
            persist_open_sample(storage, previous, now).await?;
        }

        runtime_lock = runtime.lock().await;
        runtime_lock.open_sample = Some(OpenTrackedSample {
            session_id,
            tracked_app_id: (!excluded).then_some(tracked_app.id),
            sample: sample.clone(),
            excluded,
            started_at: now,
        });
    }

    Ok((!excluded).then_some(sample))
}

async fn flush_open_sample(
    runtime: &Arc<Mutex<TrackingRuntime>>,
    storage: &StorageService,
) -> anyhow::Result<()> {
    let open_sample = {
        let mut runtime = runtime.lock().await;
        runtime.open_sample.take()
    };

    if let Some(open_sample) = open_sample {
        persist_open_sample(storage, open_sample, Utc::now()).await?;
    }

    Ok(())
}

async fn persist_open_sample(
    storage: &StorageService,
    open_sample: OpenTrackedSample,
    ended_at: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    let duration_seconds = (ended_at - open_sample.started_at).num_seconds();
    if duration_seconds <= 0 {
        return Ok(());
    }

    if open_sample.excluded {
        storage
            .create_session_segment(CreateSessionSegmentInput {
                session_id: open_sample.session_id,
                tracked_app_id: None,
                kind: SessionSegmentKind::Idle,
                window_title: None,
                started_at: open_sample.started_at,
                ended_at,
                duration_seconds,
            })
            .await?;

        return Ok(());
    }

    storage
        .create_tracked_window_event(
            Some(open_sample.session_id),
            open_sample.tracked_app_id,
            open_sample.sample.window_title.clone(),
            open_sample.started_at,
            Some(ended_at),
        )
        .await?;
    storage
        .create_session_segment(CreateSessionSegmentInput {
            session_id: open_sample.session_id,
            tracked_app_id: open_sample.tracked_app_id,
            kind: SessionSegmentKind::Focus,
            window_title: open_sample.sample.window_title,
            started_at: open_sample.started_at,
            ended_at,
            duration_seconds,
        })
        .await?;

    Ok(())
}

async fn update_runtime_snapshot(
    runtime: &Arc<Mutex<TrackingRuntime>>,
    status: TrackingStatus,
    preferences: Option<&UserPreference>,
    active_session_id: Option<i64>,
    active_window: Option<ActiveWindowSample>,
    last_error: Option<String>,
    is_tracking_live: bool,
) {
    let mut runtime = runtime.lock().await;
    runtime.snapshot = TrackingRuntimeSnapshot {
        status,
        tracking_enabled: preferences
            .map(|value| value.tracking_enabled)
            .unwrap_or(false),
        permission_granted: preferences
            .map(|value| value.tracking_permission_granted)
            .unwrap_or(false),
        onboarding_completed: preferences
            .map(|value| value.tracking_onboarding_completed)
            .unwrap_or(false),
        active_session_id,
        active_window,
        last_error,
        is_tracking_live,
    };
}

fn should_track(
    status: &TrackingStatus,
    preferences: &UserPreference,
    context: &TrackingContext,
) -> bool {
    status.capability == TrackingCapability::Supported
        && preferences.tracking_enabled
        && preferences.tracking_permission_granted
        && preferences.tracking_onboarding_completed
        && context.should_sample
        && context.active_session_id.is_some()
}

fn matches_exclusion(sample: &ActiveWindowSample, rules: &[TrackingExclusionRule]) -> bool {
    rules.iter().any(|rule| match rule.kind {
        TrackingExclusionKind::Executable => {
            normalize_executable_name(&rule.pattern)
                == normalize_executable_name(&sample.executable)
        }
        TrackingExclusionKind::WindowTitle => sample
            .window_title
            .as_deref()
            .map(|title| {
                title
                    .to_ascii_lowercase()
                    .contains(&rule.pattern.to_ascii_lowercase())
            })
            .unwrap_or(false),
        TrackingExclusionKind::Category => {
            rule.pattern.eq_ignore_ascii_case(sample.category.as_str())
        }
    })
}

fn normalize_rule_pattern(kind: TrackingExclusionKind, pattern: &str) -> String {
    match kind {
        TrackingExclusionKind::Executable => normalize_executable_name(pattern),
        TrackingExclusionKind::WindowTitle => pattern.trim().to_string(),
        TrackingExclusionKind::Category => pattern.trim().to_ascii_lowercase(),
    }
}

fn category_color(category: TrackingCategory) -> &'static str {
    match category {
        TrackingCategory::Development => "#60b7ff",
        TrackingCategory::Browser => "#8ad2b8",
        TrackingCategory::Communication => "#ffb86b",
        TrackingCategory::Writing => "#c5a3ff",
        TrackingCategory::Design => "#ff8fb1",
        TrackingCategory::Meeting => "#ff8a80",
        TrackingCategory::Research => "#9fd3ff",
        TrackingCategory::Utilities | TrackingCategory::Unknown => "#8b949e",
    }
}
