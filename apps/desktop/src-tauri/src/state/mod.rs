use crate::services::{PomodoroService, RuntimeService, StorageService, TrackingService};

#[derive(Debug)]
pub struct AppState {
    pub pomodoro: PomodoroService,
    pub tracker: TrackingService,
    pub runtime: RuntimeService,
    pub storage: StorageService,
}

impl AppState {
    pub fn new(
        pomodoro: PomodoroService,
        tracker: TrackingService,
        runtime: RuntimeService,
        storage: StorageService,
    ) -> Self {
        Self {
            pomodoro,
            tracker,
            runtime,
            storage,
        }
    }
}
