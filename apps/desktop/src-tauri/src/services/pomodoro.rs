use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use focus_domain::SessionStatus;
use focus_domain::{
    PomodoroControlState, PomodoroPhase, PomodoroPreset, PomodoroSessionOutcome, PomodoroSnapshot,
    PomodoroTransition, PomodoroTransitionKind,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, time};

use super::StorageService;

const STATE_EVENT: &str = "pomodoro://state";
const TRANSITION_EVENT: &str = "pomodoro://transition";

#[derive(Debug, Clone)]
pub struct PomodoroService {
    runtime: Arc<Mutex<PomodoroRuntime>>,
    storage: StorageService,
}

#[derive(Debug, Clone)]
pub struct StartPomodoroInput {
    pub preset: PomodoroPreset,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
    pub session_id: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum PomodoroError {
    #[error("a pomodoro session is already active")]
    AlreadyActive,
    #[error("no active pomodoro session is available")]
    NoActiveSession,
    #[error("the pomodoro session is not running")]
    NotRunning,
    #[error("the pomodoro session is not paused")]
    NotPaused,
    #[error("the active phase is not a break")]
    NotBreakPhase,
    #[error("failed to emit pomodoro event: {0}")]
    Emit(#[from] tauri::Error),
    #[error("failed to persist pomodoro session: {0}")]
    Persistence(#[from] anyhow::Error),
}

#[derive(Debug)]
struct PomodoroRuntime {
    snapshot: PomodoroSnapshot,
    next_transition_id: u64,
    phase_timing: Option<PhaseTiming>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroEvent {
    pub state: PomodoroSnapshot,
}

#[derive(Debug)]
struct PhaseTiming {
    total_seconds: i64,
    committed_elapsed_seconds: i64,
    started_at: Option<Instant>,
}

struct PomodoroUpdate {
    previous_state: PomodoroSnapshot,
    state: PomodoroSnapshot,
    transition: PomodoroTransition,
}

impl PomodoroService {
    pub fn new(app_handle: AppHandle, storage: StorageService) -> Self {
        let runtime = Arc::new(Mutex::new(PomodoroRuntime::new()));
        spawn_pomodoro_loop(app_handle, runtime.clone(), storage.clone());

        Self { runtime, storage }
    }

    pub async fn get_state(&self) -> PomodoroSnapshot {
        let runtime = self.runtime.lock().await;
        runtime.current_snapshot()
    }

    pub async fn start(
        &self,
        app_handle: &AppHandle,
        mut input: StartPomodoroInput,
    ) -> Result<PomodoroSnapshot, PomodoroError> {
        self.ensure_idle().await?;
        let session = self
            .storage
            .create_session(
                input.preset.focus_minutes,
                SessionStatus::InProgress,
                Some(input.preset.label.clone()),
                None,
            )
            .await?;
        input.session_id = Some(session.id);

        let update = {
            let mut runtime = self.runtime.lock().await;
            runtime.start(input)?
        };

        self.handle_update(app_handle, update).await
    }

    pub async fn pause(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let update = {
            let mut runtime = self.runtime.lock().await;
            runtime.pause()?
        };

        self.handle_update(app_handle, update).await
    }

    pub async fn resume(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let update = {
            let mut runtime = self.runtime.lock().await;
            runtime.resume()?
        };

        self.handle_update(app_handle, update).await
    }

    pub async fn stop(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let update = {
            let mut runtime = self.runtime.lock().await;
            runtime.stop()?
        };

        self.handle_update(app_handle, update).await
    }

    pub async fn skip_break(
        &self,
        app_handle: &AppHandle,
    ) -> Result<PomodoroSnapshot, PomodoroError> {
        let update = {
            let mut runtime = self.runtime.lock().await;
            runtime.skip_break()?
        };

        self.handle_update(app_handle, update).await
    }

    async fn ensure_idle(&self) -> Result<(), PomodoroError> {
        let runtime = self.runtime.lock().await;

        if runtime.snapshot.control_state == PomodoroControlState::Idle {
            Ok(())
        } else {
            Err(PomodoroError::AlreadyActive)
        }
    }

    async fn handle_update(
        &self,
        app_handle: &AppHandle,
        mut update: PomodoroUpdate,
    ) -> Result<PomodoroSnapshot, PomodoroError> {
        self.finalize_session_if_needed(&update.previous_state, update.transition.kind)
            .await?;

        if matches!(
            update.transition.kind,
            PomodoroTransitionKind::NextFocusStarted
        ) && update.state.session_id.is_none()
        {
            update.state =
                attach_next_session_id(&self.storage, &self.runtime, &update.state.preset).await?;
        }

        emit_transition(app_handle, &update.transition)?;
        emit_state(app_handle, &update.state)?;

        Ok(update.state)
    }

    async fn finalize_session_if_needed(
        &self,
        previous_state: &PomodoroSnapshot,
        transition_kind: PomodoroTransitionKind,
    ) -> Result<(), PomodoroError> {
        let Some(session_id) = previous_state.session_id else {
            return Ok(());
        };

        let should_finalize = matches!(
            transition_kind,
            PomodoroTransitionKind::SessionStopped
                | PomodoroTransitionKind::BreakSkipped
                | PomodoroTransitionKind::BreakCompleted
                | PomodoroTransitionKind::NextFocusStarted
        );

        if !should_finalize {
            return Ok(());
        }

        let focus_target_seconds = i64::from(previous_state.preset.focus_minutes) * 60;
        let status = if previous_state.focus_seconds_elapsed >= focus_target_seconds {
            SessionStatus::Completed
        } else {
            SessionStatus::Cancelled
        };

        self.storage
            .update_session(
                session_id,
                Utc::now(),
                previous_state.focus_seconds_elapsed,
                previous_state.break_seconds_elapsed,
                status,
            )
            .await?;

        Ok(())
    }
}

impl PomodoroRuntime {
    fn new() -> Self {
        Self {
            snapshot: PomodoroSnapshot::idle(PomodoroPreset::default()),
            next_transition_id: 1,
            phase_timing: None,
        }
    }

    fn current_snapshot(&self) -> PomodoroSnapshot {
        let mut snapshot = self.snapshot.clone();

        if let Some(phase_timing) = &self.phase_timing {
            let phase_elapsed_seconds = phase_timing.current_elapsed_seconds();
            let remaining_seconds = (phase_timing.total_seconds - phase_elapsed_seconds).max(0);
            let phase_delta = phase_elapsed_seconds - snapshot.phase_elapsed_seconds;

            snapshot.phase_elapsed_seconds = phase_elapsed_seconds;
            snapshot.remaining_seconds = remaining_seconds;

            if matches!(snapshot.phase, Some(PomodoroPhase::Focus)) {
                snapshot.focus_seconds_elapsed += phase_delta.max(0);
            } else if matches!(
                snapshot.phase,
                Some(PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak)
            ) {
                snapshot.break_seconds_elapsed += phase_delta.max(0);
            }

            if snapshot.is_running() {
                snapshot.phase_ends_at =
                    Some(Utc::now() + chrono::Duration::seconds(snapshot.remaining_seconds));
            }
        }

        snapshot
    }

    fn start(&mut self, input: StartPomodoroInput) -> Result<PomodoroUpdate, PomodoroError> {
        if self.snapshot.control_state != PomodoroControlState::Idle {
            return Err(PomodoroError::AlreadyActive);
        }

        let now = Utc::now();
        let focus_seconds = i64::from(input.preset.focus_minutes) * 60;
        let previous_state = self.current_snapshot();
        self.snapshot = PomodoroSnapshot {
            control_state: PomodoroControlState::Running,
            phase: Some(PomodoroPhase::Focus),
            preset: input.preset.clone(),
            session_started_at: Some(now),
            phase_started_at: Some(now),
            phase_ends_at: Some(now + chrono::Duration::seconds(focus_seconds)),
            paused_at: None,
            remaining_seconds: focus_seconds,
            phase_total_seconds: focus_seconds,
            phase_elapsed_seconds: 0,
            focus_seconds_elapsed: 0,
            break_seconds_elapsed: 0,
            completed_focus_blocks: self.snapshot.completed_focus_blocks,
            completed_breaks: self.snapshot.completed_breaks,
            auto_start_breaks: input.auto_start_breaks,
            auto_start_focus: input.auto_start_focus,
            can_pause: true,
            can_resume: false,
            can_stop: true,
            can_skip_break: false,
            session_id: input.session_id,
            outcome: None,
        };
        self.phase_timing = Some(PhaseTiming::running(focus_seconds));

        let transition = self.build_transition(
            PomodoroTransitionKind::SessionStarted,
            "Focus started",
            format!("{} minutes on the clock.", input.preset.focus_minutes),
        );

        Ok(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn pause(&mut self) -> Result<PomodoroUpdate, PomodoroError> {
        if !self.snapshot.is_running() {
            return Err(PomodoroError::NotRunning);
        }

        let now = Utc::now();
        let previous_state = self.current_snapshot();
        let snapshot = self.current_snapshot();
        let phase_timing = self
            .phase_timing
            .as_mut()
            .ok_or(PomodoroError::NoActiveSession)?;

        phase_timing.pause();
        self.snapshot = snapshot;
        self.snapshot.control_state = PomodoroControlState::Paused;
        self.snapshot.paused_at = Some(now);
        self.snapshot.phase_ends_at = None;
        self.snapshot.can_pause = false;
        self.snapshot.can_resume = true;

        let transition = self.build_transition(
            PomodoroTransitionKind::Paused,
            "Timer paused",
            "You can resume whenever you're ready.".to_string(),
        );

        Ok(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn resume(&mut self) -> Result<PomodoroUpdate, PomodoroError> {
        if !self.snapshot.is_paused() {
            return Err(PomodoroError::NotPaused);
        }

        let now = Utc::now();
        let previous_state = self.current_snapshot();
        let phase_timing = self
            .phase_timing
            .as_mut()
            .ok_or(PomodoroError::NoActiveSession)?;

        phase_timing.resume();
        self.snapshot.control_state = PomodoroControlState::Running;
        self.snapshot.paused_at = None;
        if self.snapshot.phase_started_at.is_none() {
            self.snapshot.phase_started_at = Some(now);
        }
        self.snapshot.phase_ends_at =
            Some(now + chrono::Duration::seconds(self.snapshot.remaining_seconds));
        self.snapshot.can_pause = true;
        self.snapshot.can_resume = false;

        let transition = self.build_transition(
            PomodoroTransitionKind::Resumed,
            "Timer resumed",
            "Back in motion.".to_string(),
        );

        Ok(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn stop(&mut self) -> Result<PomodoroUpdate, PomodoroError> {
        if self.snapshot.control_state == PomodoroControlState::Idle {
            return Err(PomodoroError::NoActiveSession);
        }

        let previous_state = self.current_snapshot();
        let snapshot = self.current_snapshot();
        let outcome = if snapshot.focus_seconds_elapsed >= snapshot.phase_total_seconds
            || snapshot.focus_seconds_elapsed >= i64::from(snapshot.preset.focus_minutes) * 60
        {
            PomodoroSessionOutcome::Completed
        } else {
            PomodoroSessionOutcome::Interrupted
        };

        self.phase_timing = None;
        self.snapshot = PomodoroSnapshot::idle(snapshot.preset);
        self.snapshot.completed_focus_blocks = snapshot.completed_focus_blocks;
        self.snapshot.completed_breaks = snapshot.completed_breaks;
        self.snapshot.outcome = Some(outcome);

        let transition = self.build_transition(
            PomodoroTransitionKind::SessionStopped,
            if outcome == PomodoroSessionOutcome::Completed {
                "Session closed"
            } else {
                "Session stopped"
            },
            if outcome == PomodoroSessionOutcome::Completed {
                "The completed focus block was saved in memory.".to_string()
            } else {
                "The timer ended before the focus block was completed.".to_string()
            },
        );

        Ok(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn skip_break(&mut self) -> Result<PomodoroUpdate, PomodoroError> {
        if !matches!(
            self.snapshot.phase,
            Some(PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak)
        ) {
            return Err(PomodoroError::NotBreakPhase);
        }

        let previous_state = self.current_snapshot();
        let preset = self.snapshot.preset.clone();
        let completed_focus_blocks = self.snapshot.completed_focus_blocks;
        let completed_breaks = self.snapshot.completed_breaks;

        self.phase_timing = None;
        self.snapshot = PomodoroSnapshot::idle(preset);
        self.snapshot.completed_focus_blocks = completed_focus_blocks;
        self.snapshot.completed_breaks = completed_breaks;
        self.snapshot.outcome = Some(PomodoroSessionOutcome::SkippedBreak);

        let transition = self.build_transition(
            PomodoroTransitionKind::BreakSkipped,
            "Break skipped",
            "Ready for the next focus block.".to_string(),
        );

        Ok(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn tick(&mut self) -> TickResult {
        let previous_snapshot = self.current_snapshot();

        if self.snapshot.control_state != PomodoroControlState::Running {
            return TickResult {
                state_changed: false,
                update: None,
            };
        }

        if previous_snapshot.remaining_seconds > 0 {
            return TickResult {
                state_changed: previous_snapshot.remaining_seconds
                    != self.snapshot.remaining_seconds,
                update: None,
            };
        }

        let update = match previous_snapshot.phase {
            Some(PomodoroPhase::Focus) => self.complete_focus_phase(previous_snapshot),
            Some(PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak) => {
                self.complete_break_phase(previous_snapshot)
            }
            None => None,
        };

        TickResult {
            state_changed: true,
            update,
        }
    }

    fn complete_focus_phase(&mut self, snapshot: PomodoroSnapshot) -> Option<PomodoroUpdate> {
        let previous_state = snapshot.clone();
        self.snapshot = snapshot.clone();
        self.snapshot.completed_focus_blocks += 1;

        let break_phase = if self
            .snapshot
            .completed_focus_blocks
            .is_multiple_of(self.snapshot.preset.sessions_until_long_break as u32)
        {
            PomodoroPhase::LongBreak
        } else {
            PomodoroPhase::ShortBreak
        };
        let break_seconds = match break_phase {
            PomodoroPhase::ShortBreak => i64::from(self.snapshot.preset.short_break_minutes) * 60,
            PomodoroPhase::LongBreak => i64::from(self.snapshot.preset.long_break_minutes) * 60,
            PomodoroPhase::Focus => 0,
        };
        let starts_running = self.snapshot.auto_start_breaks;

        self.begin_phase(break_phase, break_seconds, starts_running);

        let transition = self.build_transition(
            PomodoroTransitionKind::FocusCompleted,
            "Focus complete",
            if starts_running {
                "Break started automatically.".to_string()
            } else {
                "Break is ready when you are.".to_string()
            },
        );

        Some(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn complete_break_phase(&mut self, snapshot: PomodoroSnapshot) -> Option<PomodoroUpdate> {
        let previous_state = snapshot.clone();
        self.snapshot = snapshot;
        self.snapshot.completed_breaks += 1;

        if self.snapshot.auto_start_focus {
            let preset = self.snapshot.preset.clone();
            let auto_start_breaks = self.snapshot.auto_start_breaks;
            let auto_start_focus = self.snapshot.auto_start_focus;
            let completed_focus_blocks = self.snapshot.completed_focus_blocks;
            let completed_breaks = self.snapshot.completed_breaks;
            let focus_seconds = i64::from(preset.focus_minutes) * 60;
            let now = Utc::now();

            self.snapshot = PomodoroSnapshot {
                control_state: PomodoroControlState::Running,
                phase: Some(PomodoroPhase::Focus),
                preset,
                session_started_at: Some(now),
                phase_started_at: Some(now),
                phase_ends_at: Some(now + chrono::Duration::seconds(focus_seconds)),
                paused_at: None,
                remaining_seconds: focus_seconds,
                phase_total_seconds: focus_seconds,
                phase_elapsed_seconds: 0,
                focus_seconds_elapsed: 0,
                break_seconds_elapsed: 0,
                completed_focus_blocks,
                completed_breaks,
                auto_start_breaks,
                auto_start_focus,
                can_pause: true,
                can_resume: false,
                can_stop: true,
                can_skip_break: false,
                session_id: None,
                outcome: None,
            };
            self.phase_timing = Some(PhaseTiming::running(focus_seconds));

            let transition = self.build_transition(
                PomodoroTransitionKind::NextFocusStarted,
                "Next focus started",
                "A new focus block is already underway.".to_string(),
            );

            return Some(PomodoroUpdate {
                previous_state,
                state: self.current_snapshot(),
                transition,
            });
        }

        let preset = self.snapshot.preset.clone();
        let completed_focus_blocks = self.snapshot.completed_focus_blocks;
        let completed_breaks = self.snapshot.completed_breaks;
        self.phase_timing = None;
        self.snapshot = PomodoroSnapshot::idle(preset);
        self.snapshot.completed_focus_blocks = completed_focus_blocks;
        self.snapshot.completed_breaks = completed_breaks;
        self.snapshot.outcome = Some(PomodoroSessionOutcome::Completed);

        let transition = self.build_transition(
            PomodoroTransitionKind::BreakCompleted,
            "Break complete",
            "Ready for the next round.".to_string(),
        );

        Some(PomodoroUpdate {
            previous_state,
            state: self.current_snapshot(),
            transition,
        })
    }

    fn begin_phase(&mut self, phase: PomodoroPhase, total_seconds: i64, starts_running: bool) {
        let now = Utc::now();

        self.snapshot.control_state = if starts_running {
            PomodoroControlState::Running
        } else {
            PomodoroControlState::Paused
        };
        self.snapshot.phase = Some(phase);
        self.snapshot.phase_started_at = starts_running.then_some(now);
        self.snapshot.phase_ends_at =
            starts_running.then_some(now + chrono::Duration::seconds(total_seconds));
        self.snapshot.paused_at = (!starts_running).then_some(now);
        self.snapshot.remaining_seconds = total_seconds;
        self.snapshot.phase_total_seconds = total_seconds;
        self.snapshot.phase_elapsed_seconds = 0;
        self.snapshot.can_pause = starts_running;
        self.snapshot.can_resume = !starts_running;
        self.snapshot.can_stop = true;
        self.snapshot.can_skip_break = phase.is_break();
        self.snapshot.outcome = None;
        self.phase_timing = Some(if starts_running {
            PhaseTiming::running(total_seconds)
        } else {
            PhaseTiming::paused(total_seconds)
        });
    }

    fn build_transition(
        &mut self,
        kind: PomodoroTransitionKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> PomodoroTransition {
        let transition = PomodoroTransition {
            id: self.next_transition_id,
            kind,
            title: title.into(),
            body: body.into(),
            state: self.current_snapshot(),
        };
        self.next_transition_id += 1;

        transition
    }

    fn set_session_id(&mut self, session_id: i64) {
        self.snapshot.session_id = Some(session_id);
    }
}

impl PhaseTiming {
    fn running(total_seconds: i64) -> Self {
        Self {
            total_seconds,
            committed_elapsed_seconds: 0,
            started_at: Some(Instant::now()),
        }
    }

    fn paused(total_seconds: i64) -> Self {
        Self {
            total_seconds,
            committed_elapsed_seconds: 0,
            started_at: None,
        }
    }

    fn current_elapsed_seconds(&self) -> i64 {
        let resumed_elapsed = self
            .started_at
            .map(|started_at| started_at.elapsed().as_secs() as i64)
            .unwrap_or_default();

        (self.committed_elapsed_seconds + resumed_elapsed).min(self.total_seconds)
    }

    fn pause(&mut self) {
        self.committed_elapsed_seconds = self.current_elapsed_seconds();
        self.started_at = None;
    }

    fn resume(&mut self) {
        self.started_at = Some(Instant::now());
    }
}

struct TickResult {
    state_changed: bool,
    update: Option<PomodoroUpdate>,
}

fn spawn_pomodoro_loop(
    app_handle: AppHandle,
    runtime: Arc<Mutex<PomodoroRuntime>>,
    storage: StorageService,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(250));
        let mut last_remaining_seconds = None;

        loop {
            interval.tick().await;

            let tick_result = {
                let mut runtime = runtime.lock().await;
                let tick_result = runtime.tick();
                let current_state = runtime.current_snapshot();

                let state_changed = tick_result.state_changed
                    || last_remaining_seconds != Some(current_state.remaining_seconds);
                last_remaining_seconds = Some(current_state.remaining_seconds);

                (state_changed, tick_result.update, current_state)
            };

            let mut current_state = tick_result.2;

            if let Some(update) = tick_result.1 {
                if finalize_session_in_storage(
                    &storage,
                    &update.previous_state,
                    update.transition.kind,
                )
                .await
                .is_err()
                {
                    break;
                }

                if matches!(
                    update.transition.kind,
                    PomodoroTransitionKind::NextFocusStarted
                ) && update.state.session_id.is_none()
                {
                    match attach_next_session_id(&storage, &runtime, &update.state.preset).await {
                        Ok(state) => {
                            current_state = state;
                        }
                        Err(_) => break,
                    }
                }

                if emit_transition(&app_handle, &update.transition).is_err() {
                    break;
                }
            }

            if tick_result.0 && emit_state(&app_handle, &current_state).is_err() {
                break;
            }
        }
    });
}

async fn finalize_session_in_storage(
    storage: &StorageService,
    previous_state: &PomodoroSnapshot,
    transition_kind: PomodoroTransitionKind,
) -> anyhow::Result<()> {
    let Some(session_id) = previous_state.session_id else {
        return Ok(());
    };

    let should_finalize = matches!(
        transition_kind,
        PomodoroTransitionKind::SessionStopped
            | PomodoroTransitionKind::BreakSkipped
            | PomodoroTransitionKind::BreakCompleted
            | PomodoroTransitionKind::NextFocusStarted
    );

    if !should_finalize {
        return Ok(());
    }

    let focus_target_seconds = i64::from(previous_state.preset.focus_minutes) * 60;
    let status = if previous_state.focus_seconds_elapsed >= focus_target_seconds {
        SessionStatus::Completed
    } else {
        SessionStatus::Cancelled
    };

    storage
        .update_session(
            session_id,
            Utc::now(),
            previous_state.focus_seconds_elapsed,
            previous_state.break_seconds_elapsed,
            status,
        )
        .await?;

    Ok(())
}

async fn attach_next_session_id(
    storage: &StorageService,
    runtime: &Arc<Mutex<PomodoroRuntime>>,
    preset: &PomodoroPreset,
) -> anyhow::Result<PomodoroSnapshot> {
    let session = storage
        .create_session(
            preset.focus_minutes,
            SessionStatus::InProgress,
            Some(preset.label.clone()),
            None,
        )
        .await?;

    let mut runtime = runtime.lock().await;
    runtime.set_session_id(session.id);

    Ok(runtime.current_snapshot())
}

fn emit_state(app_handle: &AppHandle, state: &PomodoroSnapshot) -> Result<(), PomodoroError> {
    app_handle.emit(
        STATE_EVENT,
        PomodoroEvent {
            state: state.clone(),
        },
    )?;
    Ok(())
}

fn emit_transition(
    app_handle: &AppHandle,
    transition: &PomodoroTransition,
) -> Result<(), PomodoroError> {
    app_handle.emit(TRANSITION_EVENT, transition)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::finalize_session_in_storage;
    use super::{PomodoroRuntime, StartPomodoroInput};
    use focus_domain::{
        PomodoroControlState, PomodoroPhase, PomodoroPreset, PomodoroTransitionKind, SessionStatus,
    };
    use tempfile::tempdir;

    use crate::services::StorageService;

    #[test]
    fn starts_a_focus_phase() {
        let mut runtime = PomodoroRuntime::new();

        let transition = runtime
            .start(StartPomodoroInput {
                preset: PomodoroPreset::default(),
                auto_start_breaks: true,
                auto_start_focus: false,
                session_id: None,
            })
            .expect("timer should start");

        assert_eq!(
            transition.transition.kind,
            PomodoroTransitionKind::SessionStarted
        );
        assert_eq!(runtime.snapshot.phase, Some(PomodoroPhase::Focus));
        assert_eq!(
            runtime.snapshot.control_state,
            PomodoroControlState::Running
        );
    }

    #[test]
    fn pauses_and_resumes_without_losing_phase() {
        let mut runtime = PomodoroRuntime::new();
        runtime
            .start(StartPomodoroInput {
                preset: PomodoroPreset::default(),
                auto_start_breaks: true,
                auto_start_focus: false,
                session_id: None,
            })
            .expect("timer should start");

        runtime.pause().expect("pause should succeed");
        assert_eq!(runtime.snapshot.control_state, PomodoroControlState::Paused);

        runtime.resume().expect("resume should succeed");
        assert_eq!(
            runtime.snapshot.control_state,
            PomodoroControlState::Running
        );
        assert_eq!(runtime.snapshot.phase, Some(PomodoroPhase::Focus));
    }

    #[test]
    fn moves_to_break_when_focus_completes() {
        let mut runtime = PomodoroRuntime::new();
        runtime
            .start(StartPomodoroInput {
                preset: PomodoroPreset::default(),
                auto_start_breaks: true,
                auto_start_focus: false,
                session_id: None,
            })
            .expect("timer should start");
        runtime
            .phase_timing
            .as_mut()
            .expect("timing")
            .committed_elapsed_seconds = runtime.snapshot.phase_total_seconds;

        let tick = runtime.tick();

        assert!(tick.update.is_some());
        assert_eq!(runtime.snapshot.phase, Some(PomodoroPhase::ShortBreak));
    }

    #[tokio::test]
    async fn finalizes_a_persisted_session() {
        let temp = tempdir().expect("temporary directory should be created");
        let storage = StorageService::new(temp.path().join("focus-time.sqlite"))
            .await
            .expect("storage should initialize");
        storage
            .ensure_ready()
            .await
            .expect("storage should be ready");
        let session = storage
            .create_session(
                25,
                SessionStatus::InProgress,
                Some("Classic".to_string()),
                None,
            )
            .await
            .expect("session should be created");

        let mut snapshot = focus_domain::PomodoroSnapshot::idle(PomodoroPreset::default());
        snapshot.session_id = Some(session.id);
        snapshot.preset = PomodoroPreset::default();
        snapshot.focus_seconds_elapsed = 1_500;
        snapshot.break_seconds_elapsed = 300;

        finalize_session_in_storage(&storage, &snapshot, PomodoroTransitionKind::BreakCompleted)
            .await
            .expect("session should be finalized");

        let persisted = storage
            .list_sessions(1)
            .await
            .expect("session should be readable")
            .into_iter()
            .next()
            .expect("one session should exist");

        assert_eq!(persisted.status, SessionStatus::Completed);
        assert_eq!(persisted.actual_focus_seconds, 1_500);
        assert_eq!(persisted.break_seconds, 300);
        assert!(persisted.ended_at.is_some());
    }
}
