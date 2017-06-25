use wayland_server::protocol::wl_shell_surface;

use easy_wayland::EasyEventLoopHandle;

pub struct ShellSurfaceHandler;

impl ShellSurfaceHandler {
    pub fn new() -> ShellSurfaceHandler {
        ShellSurfaceHandler {}
    }
}

impl wl_shell_surface::Handler for ShellSurfaceHandler {}


declare_handler!(ShellSurfaceHandler,
                 wl_shell_surface::Handler,
                 wl_shell_surface::WlShellSurface);
