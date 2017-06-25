use wayland_protocols::unstable::xdg_shell::server::zxdg_toplevel_v6 as xdg_toplevel;

use easy_wayland::EasyEventLoopHandle;
use wayland_server::{GlobalHandler, EventLoopHandle, Client};

pub struct XdgTopLevelHandler;

impl XdgTopLevelHandler {
    pub fn new() -> XdgTopLevelHandler {
        XdgTopLevelHandler {}
    }
}

impl xdg_toplevel::Handler for XdgTopLevelHandler {
    // fn get_toplevel(
    //       &mut self,
    //       evqh: &mut EventLoopHandle,
    //       client: &Client,
    //       resource: &xdg_surface::ZxdgSurfaceV6,
    //       id: xdg_toplevel::ZxdgToplevelV6
    // ) {
    //     println!("get_toplevel");
    // }
}


declare_handler!(XdgTopLevelHandler,
                 xdg_toplevel::Handler,
                 xdg_toplevel::ZxdgToplevelV6);
