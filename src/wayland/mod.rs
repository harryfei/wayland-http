mod window_map;
mod easy_wayland;
mod implementation;
mod wayland_window_manager;
mod idata;

use std::thread;
use wayland_server::create_display;
use smithay::wayland::shm::init_shm_global;
use wayland_server::protocol::wl_shm::Format;
use std::sync::mpsc::channel;
use std::sync::Arc;
use slog_scope;

use window_manager::WindowManager;

use self::implementation::init_shell;

pub fn run_wayland_thread() -> Arc<WindowManager> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        info!("Wayland server started!");
        let (mut dpy, mut event_loop) = create_display();

        // panic if socket create failed
        dpy.add_socket_auto().unwrap();

        let logger = slog_scope::logger();

        let _shm_global = init_shm_global(
            &mut event_loop,
            vec![Format::Yuyv, Format::C8],
            logger.clone(), // No logger
        );

        let (_compositor_token, _, window_manager) = init_shell(&mut event_loop, &logger.clone());

        sender.send(window_manager.clone()).unwrap();

        loop {
            event_loop.dispatch(Some(16)).unwrap();
            dpy.flush_clients();

            let _ = window_manager.refresh();
        }
    });

    receiver.recv().unwrap()
}
