use std::sync::Arc;
use window_manager::WindowManager;

#[derive(Clone)]
pub struct State {
    pub window_manager: Arc<WindowManager>,
}
