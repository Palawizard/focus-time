use crate::services::{RuntimeService, StorageService};

#[derive(Debug)]
pub struct AppState {
    pub runtime: RuntimeService,
    pub storage: StorageService,
}

impl AppState {
    pub fn new(runtime: RuntimeService, storage: StorageService) -> Self {
        Self { runtime, storage }
    }
}
