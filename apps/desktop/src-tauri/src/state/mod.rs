use crate::services::RuntimeService;

#[derive(Debug)]
pub struct AppState {
    pub runtime: RuntimeService,
}

impl AppState {
    pub fn new(runtime: RuntimeService) -> Self {
        Self { runtime }
    }
}
