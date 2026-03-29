use crate::services::{PomodoroService, RuntimeService, StorageService};

#[derive(Debug)]
pub struct AppState {
    pub pomodoro: PomodoroService,
    pub runtime: RuntimeService,
    pub storage: StorageService,
}

impl AppState {
    pub fn new(pomodoro: PomodoroService, runtime: RuntimeService, storage: StorageService) -> Self {
        Self {
            pomodoro,
            runtime,
            storage,
        }
    }
}
