#[macro_use]
extern crate wayland_server;
extern crate wayland_protocols;
extern crate smithay;
extern crate image;
extern crate time;

// #[macro_use] extern crate wayland_sys;
// #[macro_use] extern crate bitflags;

mod compositor;
mod surface;
mod region;
mod shell;
mod shell_surface;
mod xdg;
mod xdg_shell_surface;
mod xdg_toplevel;

mod easy_wayland;

use wayland_server::create_display;

use easy_wayland::EasyEventLoop;

use smithay::shm::ShmGlobal;
use wayland_server::protocol::wl_shm;


fn main() {
    println!("Wayland server running!");
    let (mut dpy, mut event_loop) = create_display();
    dpy.add_socket_auto();


    // Insert the ShmGlobal as a handler to your event loop
    // Here, we specify that Yuyv and C8 format are supported
    // additionnaly to the standart Argb8888 and Xrgb8888.
    let handler_id = event_loop.add_handler_with_init(ShmGlobal::new(
        vec![wl_shm::Format::Yuyv, wl_shm::Format::C8],
        None // we don't provide a logger here
    ));
    // Register this handler to advertise a wl_shm global of version 1
    let shm_global = event_loop.register_global::<wl_shm::WlShm, ShmGlobal>(handler_id, 1);
    // Retrieve the shm token for later use to access the buffers
    let shm_token = {
        let state = event_loop.state();
        state.get_handler::<ShmGlobal>(handler_id).get_token()
    };

    // We must keep all global object in global lifetime.
    let (compositor_global, _) =
        event_loop.easy_register_global(compositor::BindHandler::new(shm_token), 3);
    let (shell_global, _) = event_loop.easy_register_global(shell::BindHandler {}, 1);
    let (xdg_global, _) = event_loop.easy_register_global(xdg::BindHandler {}, 1);


    event_loop.run();
}
