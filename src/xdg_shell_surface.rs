use wayland_protocols::unstable::xdg_shell::server::zxdg_surface_v6 as xdg_surface;
use wayland_protocols::unstable::xdg_shell::server::zxdg_toplevel_v6 as xdg_toplevel;

use easy_wayland::EasyEventLoopHandle;
use wayland_server::{GlobalHandler, EventLoopHandle, Client};

use xdg_toplevel as xdg_toplevel_impl;

pub struct XdgShellSurfaceHandler {
    serial: u32,
}

impl XdgShellSurfaceHandler {
    pub fn new() -> XdgShellSurfaceHandler {
        XdgShellSurfaceHandler { serial: 0 }
    }
}

impl xdg_surface::Handler for XdgShellSurfaceHandler {
    fn get_toplevel(&mut self,
                    evqh: &mut EventLoopHandle,
                    client: &Client,
                    resource: &xdg_surface::ZxdgSurfaceV6,
                    id: xdg_toplevel::ZxdgToplevelV6) {
        evqh.easy_register(&id, xdg_toplevel_impl::XdgTopLevelHandler::new());

        id.configure(0, 0, vec![xdg_toplevel::State::Activated.to_raw() as u8]);
        resource.configure(self.serial);
        self.serial = self.serial + 1;
        println!("get_toplevel");
    }
}


declare_handler!(XdgShellSurfaceHandler,
                 xdg_surface::Handler,
                 xdg_surface::ZxdgSurfaceV6);
