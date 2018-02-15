use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use smithay::wayland::shell::ShellSurfaceRole;

use wayland_server::protocol::wl_callback::WlCallback;
use window_manager::WindowEvent;
use futures::sync::mpsc::Sender;

pub type WindowEventSenderRegistry = HashMap<u64, Sender<WindowEvent>>;

define_roles!(Roles => [ ShellSurface, ShellSurfaceRole ] );

#[derive(Default)]
pub struct SurfaceData {
    pub id: u64,

    pub callback_queue: VecDeque<WlCallback>,
}

pub struct CompositorIData {
    pub sender_registry: Arc<Mutex<WindowEventSenderRegistry>>,
}

impl CompositorIData {
    pub fn new(sender_registry: Arc<Mutex<WindowEventSenderRegistry>>) -> Self {
        Self { sender_registry }
    }
}
