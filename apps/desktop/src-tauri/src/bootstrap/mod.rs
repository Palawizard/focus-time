use crate::services::RuntimeService;
use crate::state::AppState;

pub fn build_state() -> AppState {
    AppState::new(RuntimeService::new())
}
