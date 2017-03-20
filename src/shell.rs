use wayland_server::protocol::wl_shell;
use wayland_server::protocol::wl_shell_surface;
use wayland_server::protocol::wl_surface;
use wayland_server::{
    GlobalHandler,
    EventLoopHandle,
    Client
};

use easy_wayland::EasyEventLoopHandle;
use shell_surface;

pub struct ShellHandler;

impl wl_shell::Handler for ShellHandler {
    fn get_shell_surface(&mut self,
                         evqh: &mut EventLoopHandle,
                         _client: &Client,
                         _resource: &wl_shell::WlShell,
                         id: wl_shell_surface::WlShellSurface,
                         _surface: &wl_surface::WlSurface) {
        evqh.easy_register(&id, shell_surface::ShellSurfaceHandler{});
    }
    
}

declare_handler!(ShellHandler, wl_shell::Handler, wl_shell::WlShell);

pub struct BindHandler;

impl BindHandler {
    pub fn new() -> BindHandler {
        BindHandler{}
    }
}

impl GlobalHandler<wl_shell::WlShell> for BindHandler {
    fn bind(&mut self,
            evlh: &mut EventLoopHandle,
            _client: &Client,
            shell: wl_shell::WlShell
    ) {
        evlh.easy_register(&shell, ShellHandler{});
    }
}
