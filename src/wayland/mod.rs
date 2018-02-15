mod window_map;
mod easy_wayland;
mod implementation;
mod wayland_window_manager;
mod idata;
mod wayland_server;

use std::thread;
use std::sync::mpsc::channel;
use std::sync::Arc;
use slog_scope;

use window_manager::WindowManager;
use self::wayland_server::WaylandServerBuilder;

pub fn run_wayland_thread() -> Arc<WindowManager> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        info!("Wayland server started!");

        let logger = slog_scope::logger();

        let wayland_server = WaylandServerBuilder::new(logger).build();

        sender.send(wayland_server.get_window_manager()).unwrap();

        wayland_server.run();
    });

    receiver.recv().unwrap()
}
