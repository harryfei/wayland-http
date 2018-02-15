use wayland_server::{create_display, Display, EventLoop};
use smithay::wayland::shm::init_shm_global;
use smithay::wayland::seat::Seat;
use wayland_server::protocol::wl_shm::Format;
use std::sync::Arc;
use slog;

use window_manager::WindowManager;
use super::implementation::init_shell;
use super::wayland_window_manager::WaylandWindowManager;

pub struct WaylandServer {
    dpy: Display,
    event_loop: EventLoop,
    wm: Arc<WaylandWindowManager>,
}

impl WaylandServer {
    pub fn get_window_manager(&self) -> Arc<WindowManager> {
        self.wm.clone()
    }

    pub fn run(mut self) {
        loop {
            self.event_loop.dispatch(Some(16)).unwrap();
            self.dpy.flush_clients();

            let _ = self.wm.refresh();
        }
    }
}

pub struct WaylandServerBuilder {
    logger: slog::Logger,
}

impl WaylandServerBuilder {
    pub fn new(logger: slog::Logger) -> Self {
        Self { logger }
    }

    fn add_socket(&self, dpy: &mut Display) {
        // panic if socket create failed
        dpy.add_socket_auto().unwrap();
    }

    fn init_shm(&self, event_loop: &mut EventLoop) {
        let _shm_global = init_shm_global(
            event_loop,
            vec![Format::Yuyv, Format::C8],
            self.logger.clone(), // No logger
        );
    }

    fn init_shell(&self, event_loop: &mut EventLoop) -> Arc<WaylandWindowManager> {
        let (_compositor_token, _, window_manager) = init_shell(event_loop, &self.logger.clone());
        window_manager
    }

    fn init_seat(&self, event_loop: &mut EventLoop) {
        let (seat_token, _seat_global) =
            Seat::new(event_loop, "wayland-http".into(), self.logger.clone());

        let _pointer = event_loop.state().get_mut(&seat_token).add_pointer();
        let _keyboard = event_loop
            .state()
            .get_mut(&seat_token)
            .add_keyboard("", "", "", None, 1000, 500)
            .expect("Failed to initialize the keyboard");
    }

    pub fn build(&self) -> WaylandServer {
        let (mut dpy, mut event_loop) = create_display();

        self.add_socket(&mut dpy);

        self.init_shm(&mut event_loop);

        // init compositor and shell(include xdg_shell) global
        let wm = self.init_shell(&mut event_loop);

        self.init_seat(&mut event_loop);

        // init seat global
        WaylandServer {
            dpy,
            event_loop,
            wm,
        }
    }
}
