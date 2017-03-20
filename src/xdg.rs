use wayland_protocols::unstable::xdg_shell::server::zxdg_shell_v6 as xdg_shell;
// ::wl_shell;
// use wayland_server::protocol::wl_shell_surface;
// use wayland_server::protocol::wl_surface;

use easy_wayland::EasyEventLoopHandle;
use wayland_server::{
    GlobalHandler,
    EventLoopHandle,
    Client
};

pub struct XdgShellHandler;

impl xdg_shell::Handler for XdgShellHandler{}

declare_handler!(XdgShellHandler, xdg_shell::Handler, xdg_shell::ZxdgShellV6);

pub struct BindHandler;

impl GlobalHandler<xdg_shell::ZxdgShellV6> for BindHandler {
    fn bind(&mut self,
            evlh: &mut EventLoopHandle,
            _client: &Client,
            shell: xdg_shell::ZxdgShellV6,
    ) {
        evlh.easy_register(&shell, XdgShellHandler{});
    }
}
