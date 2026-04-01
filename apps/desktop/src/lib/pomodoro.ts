import type { PomodoroSnapshot, StartPomodoroRequest } from "../types/pomodoro";
import { desktopInvoke } from "./desktop-api";

export const POMODORO_STATE_EVENT = "pomodoro://state";
export const POMODORO_TRANSITION_EVENT = "pomodoro://transition";

export function getPomodoroState() {
  return desktopInvoke<PomodoroSnapshot>("get_pomodoro_state");
}

export function startPomodoro(request: StartPomodoroRequest) {
  return desktopInvoke<PomodoroSnapshot>("start_pomodoro", { request });
}

export function pausePomodoro() {
  return desktopInvoke<PomodoroSnapshot>("pause_pomodoro");
}

export function resumePomodoro() {
  return desktopInvoke<PomodoroSnapshot>("resume_pomodoro");
}

export function stopPomodoro() {
  return desktopInvoke<PomodoroSnapshot>("stop_pomodoro");
}

export function skipPomodoroBreak() {
  return desktopInvoke<PomodoroSnapshot>("skip_pomodoro_break");
}
