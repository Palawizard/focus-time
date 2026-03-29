use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use focus_domain::{
    PomodoroControlState, PomodoroPhase, PomodoroPreset, PomodoroSessionOutcome,
    PomodoroSnapshot, PomodoroTransition, PomodoroTransitionKind,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, time};

const STATE_EVENT: &str = "pomodoro://state";
const TRANSITION_EVENT: &str = "pomodoro://transition";

#[derive(Debug, Clone)]
pub struct PomodoroService {
    runtime: Arc<Mutex<PomodoroRuntime>>,
}

#[derive(Debug, Clone)]
pub struct StartPomodoroInput {
    pub preset: PomodoroPreset,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
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

impl PomodoroService {
    pub fn new(app_handle: AppHandle) -> Self {
        let runtime = Arc::new(Mutex::new(PomodoroRuntime::new()));
        spawn_pomodoro_loop(app_handle, runtime.clone());

        Self { runtime }
    }

    pub async fn get_state(&self) -> PomodoroSnapshot {
        let runtime = self.runtime.lock().await;
        runtime.current_snapshot()
    }

    pub async fn start(
        &self,
        app_handle: &AppHandle,
        input: StartPomodoroInput,
    ) -> Result<PomodoroSnapshot, PomodoroError> {
        let (state, transition) = {
            let mut runtime = self.runtime.lock().await;
            let transition = runtime.start(input)?;
            (runtime.current_snapshot(), transition)
        };

        emit_transition(app_handle, &transition)?;
        emit_state(app_handle, &state)?;

        Ok(state)
    }

    pub async fn pause(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let (state, transition) = {
            let mut runtime = self.runtime.lock().await;
            let transition = runtime.pause()?;
            (runtime.current_snapshot(), transition)
        };

        emit_transition(app_handle, &transition)?;
        emit_state(app_handle, &state)?;

        Ok(state)
    }

    pub async fn resume(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let (state, transition) = {
            let mut runtime = self.runtime.lock().await;
            let transition = runtime.resume()?;
            (runtime.current_snapshot(), transition)
        };

        emit_transition(app_handle, &transition)?;
        emit_state(app_handle, &state)?;

        Ok(state)
    }

    pub async fn stop(&self, app_handle: &AppHandle) -> Result<PomodoroSnapshot, PomodoroError> {
        let (state, transition) = {
            let mut runtime = self.runtime.lock().await;
            let transition = runtime.stop()?;
            (runtime.current_snapshot(), transition)
        };

        emit_transition(app_handle, &transition)?;
        emit_state(app_handle, &state)?;

        Ok(state)
    }

    pub async fn skip_break(
        &self,
        app_handle: &AppHandle,
    ) -> Result<PomodoroSnapshot, PomodoroError> {
        let (state, transition) = {
            let mut runtime = self.runtime.lock().await;
            let transition = runtime.skip_break()?;
            (runtime.current_snapshot(), transition)
        };

        emit_transition(app_handle, &transition)?;
        emit_state(app_handle, &state)?;

        Ok(state)
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

    fn start(&mut self, input: StartPomodoroInput) -> Result<PomodoroTransition, PomodoroError> {
        if self.snapshot.control_state != PomodoroControlState::Idle {
            return Err(PomodoroError::AlreadyActive);
        }

        let now = Utc::now();
        let focus_seconds = i64::from(input.preset.focus_minutes) * 60;
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
            session_id: None,
            outcome: None,
        };
        self.phase_timing = Some(PhaseTiming::running(focus_seconds));

        Ok(self.build_transition(
            PomodoroTransitionKind::SessionStarted,
            "Focus started",
            format!("{} minutes on the clock.", input.preset.focus_minutes),
        ))
    }

    fn pause(&mut self) -> Result<PomodoroTransition, PomodoroError> {
        if !self.snapshot.is_running() {
            return Err(PomodoroError::NotRunning);
        }

        let now = Utc::now();
        let snapshot = self.current_snapshot();
        let phase_timing = self.phase_timing.as_mut().ok_or(PomodoroError::NoActiveSession)?;

        phase_timing.pause();
        self.snapshot = snapshot;
        self.snapshot.control_state = PomodoroControlState::Paused;
        self.snapshot.paused_at = Some(now);
        self.snapshot.phase_ends_at = None;
        self.snapshot.can_pause = false;
        self.snapshot.can_resume = true;

        Ok(self.build_transition(
            PomodoroTransitionKind::Paused,
            "Timer paused",
            "You can resume whenever you're ready.".to_string(),
        ))
    }

    fn resume(&mut self) -> Result<PomodoroTransition, PomodoroError> {
        if !self.snapshot.is_paused() {
            return Err(PomodoroError::NotPaused);
        }

        let now = Utc::now();
        let phase_timing = self.phase_timing.as_mut().ok_or(PomodoroError::NoActiveSession)?;

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

        Ok(self.build_transition(
            PomodoroTransitionKind::Resumed,
            "Timer resumed",
            "Back in motion.".to_string(),
        ))
    }

    fn stop(&mut self) -> Result<PomodoroTransition, PomodoroError> {
        if self.snapshot.control_state == PomodoroControlState::Idle {
            return Err(PomodoroError::NoActiveSession);
        }

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

        Ok(self.build_transition(
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
        ))
    }

    fn skip_break(&mut self) -> Result<PomodoroTransition, PomodoroError> {
        if !matches!(
            self.snapshot.phase,
            Some(PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak)
        ) {
            return Err(PomodoroError::NotBreakPhase);
        }

        let preset = self.snapshot.preset.clone();
        let completed_focus_blocks = self.snapshot.completed_focus_blocks;
        let completed_breaks = self.snapshot.completed_breaks;

        self.phase_timing = None;
        self.snapshot = PomodoroSnapshot::idle(preset);
        self.snapshot.completed_focus_blocks = completed_focus_blocks;
        self.snapshot.completed_breaks = completed_breaks;
        self.snapshot.outcome = Some(PomodoroSessionOutcome::SkippedBreak);

        Ok(self.build_transition(
            PomodoroTransitionKind::BreakSkipped,
            "Break skipped",
            "Ready for the next focus block.".to_string(),
        ))
    }

    fn tick(&mut self) -> TickResult {
        let previous_snapshot = self.current_snapshot();

        if self.snapshot.control_state != PomodoroControlState::Running {
            return TickResult {
                state_changed: false,
                transition: None,
            };
        }

        if previous_snapshot.remaining_seconds > 0 {
            return TickResult {
                state_changed: previous_snapshot.remaining_seconds != self.snapshot.remaining_seconds,
                transition: None,
            };
        }

        let transition = match previous_snapshot.phase {
            Some(PomodoroPhase::Focus) => self.complete_focus_phase(previous_snapshot),
            Some(PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak) => {
                self.complete_break_phase(previous_snapshot)
            }
            None => None,
        };

        TickResult {
            state_changed: true,
            transition,
        }
    }

    fn complete_focus_phase(
        &mut self,
        snapshot: PomodoroSnapshot,
    ) -> Option<PomodoroTransition> {
        self.snapshot = snapshot.clone();
        self.snapshot.completed_focus_blocks += 1;

        let break_phase = if self.snapshot.completed_focus_blocks
            % (self.snapshot.preset.sessions_until_long_break as u32)
            == 0
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

        Some(self.build_transition(
            PomodoroTransitionKind::FocusCompleted,
            "Focus complete",
            if starts_running {
                "Break started automatically.".to_string()
            } else {
                "Break is ready when you are.".to_string()
            },
        ))
    }

    fn complete_break_phase(
        &mut self,
        snapshot: PomodoroSnapshot,
    ) -> Option<PomodoroTransition> {
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

            return Some(self.build_transition(
                PomodoroTransitionKind::NextFocusStarted,
                "Next focus started",
                "A new focus block is already underway.".to_string(),
            ));
        }

        let preset = self.snapshot.preset.clone();
        let completed_focus_blocks = self.snapshot.completed_focus_blocks;
        let completed_breaks = self.snapshot.completed_breaks;
        self.phase_timing = None;
        self.snapshot = PomodoroSnapshot::idle(preset);
        self.snapshot.completed_focus_blocks = completed_focus_blocks;
        self.snapshot.completed_breaks = completed_breaks;
        self.snapshot.outcome = Some(PomodoroSessionOutcome::Completed);

        Some(self.build_transition(
            PomodoroTransitionKind::BreakCompleted,
            "Break complete",
            "Ready for the next round.".to_string(),
        ))
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
    transition: Option<PomodoroTransition>,
}

fn spawn_pomodoro_loop(app_handle: AppHandle, runtime: Arc<Mutex<PomodoroRuntime>>) {
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

                (state_changed, tick_result.transition, current_state)
            };

            if let Some(transition) = tick_result.1.as_ref() {
                if emit_transition(&app_handle, transition).is_err() {
                    break;
                }
            }

            if tick_result.0 && emit_state(&app_handle, &tick_result.2).is_err() {
                break;
            }
        }
    });
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
    use super::{PomodoroRuntime, StartPomodoroInput};
    use focus_domain::{PomodoroControlState, PomodoroPhase, PomodoroPreset, PomodoroTransitionKind};

    #[test]
    fn starts_a_focus_phase() {
        let mut runtime = PomodoroRuntime::new();

        let transition = runtime
            .start(StartPomodoroInput {
                preset: PomodoroPreset::default(),
                auto_start_breaks: true,
                auto_start_focus: false,
            })
            .expect("timer should start");

        assert_eq!(transition.kind, PomodoroTransitionKind::SessionStarted);
        assert_eq!(runtime.snapshot.phase, Some(PomodoroPhase::Focus));
        assert_eq!(runtime.snapshot.control_state, PomodoroControlState::Running);
    }

    #[test]
    fn pauses_and_resumes_without_losing_phase() {
        let mut runtime = PomodoroRuntime::new();
        runtime
            .start(StartPomodoroInput {
                preset: PomodoroPreset::default(),
                auto_start_breaks: true,
                auto_start_focus: false,
            })
            .expect("timer should start");

        runtime.pause().expect("pause should succeed");
        assert_eq!(runtime.snapshot.control_state, PomodoroControlState::Paused);

        runtime.resume().expect("resume should succeed");
        assert_eq!(runtime.snapshot.control_state, PomodoroControlState::Running);
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
            })
            .expect("timer should start");
        runtime.phase_timing.as_mut().expect("timing").committed_elapsed_seconds =
            runtime.snapshot.phase_total_seconds;

        let tick = runtime.tick();

        assert!(tick.transition.is_some());
        assert_eq!(runtime.snapshot.phase, Some(PomodoroPhase::ShortBreak));
    }
}
