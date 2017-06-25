use std::sync::Arc;

use wayland_server::protocol::wl_compositor::{Handler, WlCompositor};
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::protocol::wl_region::WlRegion;

use wayland_server::{GlobalHandler, EventLoopHandle, Client};
use smithay::shm::ShmGlobalToken;

use easy_wayland::EasyEventLoopHandle;
use surface;
use region;

pub struct CompositorHandler {
    shm_token: Arc<ShmGlobalToken>,
}

impl CompositorHandler {
    pub fn new(shm_token: Arc<ShmGlobalToken>) -> CompositorHandler {
        CompositorHandler { shm_token: shm_token }
    }
}

impl Handler for CompositorHandler {
    fn create_surface(&mut self,
                      evqh: &mut EventLoopHandle,
                      client: &Client,
                      resource: &WlCompositor,
                      id: WlSurface) {
        println!("create surface");
        evqh.easy_register(&id, surface::SurfaceHandler::new(self.shm_token.clone()));
    }
    fn create_region(&mut self,
                     evqh: &mut EventLoopHandle,
                     client: &Client,
                     resource: &WlCompositor,
                     id: WlRegion) {
        println!("create region");
        evqh.easy_register(&id, region::RegionHandler::new());
    }
}

declare_handler!(CompositorHandler, Handler, WlCompositor);

pub struct BindHandler {
    shm_token: Arc<ShmGlobalToken>,
}

impl BindHandler {
    pub fn new(shm_token: ShmGlobalToken) -> BindHandler {
        BindHandler { shm_token: Arc::new(shm_token) }
    }
}

impl GlobalHandler<WlCompositor> for BindHandler {
    fn bind(&mut self, evlh: &mut EventLoopHandle, _client: &Client, compositor: WlCompositor) {
        evlh.easy_register(&compositor, CompositorHandler::new(self.shm_token.clone()));
    }
}
