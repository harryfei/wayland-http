use wayland_server::protocol::wl_compositor::{Handler, WlCompositor};
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::protocol::wl_region::WlRegion;

use wayland_server::{
    GlobalHandler,
    EventLoopHandle,
    Client
};

use easy_wayland::EasyEventLoopHandle;
use surface;
use region;

pub struct CompositorHandler;

impl Handler for CompositorHandler {
    fn create_surface(&mut self,
                      evqh: &mut EventLoopHandle,
                      client: &Client,
                      resource: &WlCompositor,
                      id: WlSurface) {
        println!("create surface");
        evqh.easy_register(&id, surface::SurfaceHandler{});

    }
    fn create_region(&mut self,
                     evqh: &mut EventLoopHandle,
                     client: &Client,
                     resource: &WlCompositor,
                     id: WlRegion) {
        evqh.easy_register(&id, region::RegionHandler::new());
    }
}

declare_handler!(CompositorHandler, Handler, WlCompositor);

pub struct BindHandler;

impl GlobalHandler<WlCompositor> for BindHandler {
    fn bind(&mut self,
            evlh: &mut EventLoopHandle,
            _client: &Client,
            compositor: WlCompositor
    ) {
        evlh.easy_register(&compositor, CompositorHandler{});
    }
}
