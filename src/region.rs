use wayland_server::protocol::wl_region;

use easy_wayland::EasyEventLoopHandle;

pub struct RegionHandler;

impl RegionHandler {
    pub fn new() -> RegionHandler {
        RegionHandler {}
    }
}

impl wl_region::Handler for RegionHandler {}

declare_handler!(RegionHandler, wl_region::Handler, wl_region::WlRegion);
