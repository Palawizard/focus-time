import { invoke } from "@tauri-apps/api/core";

import type {
  PomodoroSnapshot,
  StartPomodoroRequest,
} from "../types/pomodoro";

export const POMODORO_STATE_EVENT = "pomodoro://state";
export const POMODORO_TRANSITION_EVENT = "pomodoro://transition";

export function getPomodoroState() {
  return invoke<PomodoroSnapshot>("get_pomodoro_state");
}

export function startPomodoro(request: StartPomodoroRequest) {
  return invoke<PomodoroSnapshot>("start_pomodoro", { request });
}

export function pausePomodoro() {
  return invoke<PomodoroSnapshot>("pause_pomodoro");
}

export function resumePomodoro() {
  return invoke<PomodoroSnapshot>("resume_pomodoro");
}

export function stopPomodoro() {
  return invoke<PomodoroSnapshot>("stop_pomodoro");
}

export function skipPomodoroBreak() {
  return invoke<PomodoroSnapshot>("skip_pomodoro_break");
}
