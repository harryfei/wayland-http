use wayland_protocols::unstable::xdg_shell::server::zxdg_shell_v6 as xdg_shell;
use wayland_protocols::unstable::xdg_shell::server::zxdg_surface_v6 as xdg_surface;

use easy_wayland::EasyEventLoopHandle;
use wayland_server::{GlobalHandler, EventLoopHandle, Client};
use wayland_server::protocol::wl_surface::WlSurface;

use xdg_shell_surface;

pub struct XdgShellHandler;

impl xdg_shell::Handler for XdgShellHandler {
    fn get_xdg_surface(&mut self,
                       evqh: &mut EventLoopHandle,
                       client: &Client,
                       resource: &xdg_shell::ZxdgShellV6,
                       id: xdg_surface::ZxdgSurfaceV6,
                       surface: &WlSurface) {

        evqh.easy_register(&id, xdg_shell_surface::XdgShellSurfaceHandler::new());
    }
}

declare_handler!(XdgShellHandler, xdg_shell::Handler, xdg_shell::ZxdgShellV6);

pub struct BindHandler;

impl GlobalHandler<xdg_shell::ZxdgShellV6> for BindHandler {
    fn bind(&mut self,
            evlh: &mut EventLoopHandle,
            _client: &Client,
            shell: xdg_shell::ZxdgShellV6) {
        evlh.easy_register(&shell, XdgShellHandler {});
    }
}
