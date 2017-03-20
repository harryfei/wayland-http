use wayland_server::protocol::wl_surface;

use easy_wayland::EasyEventLoopHandle;

pub struct SurfaceHandler;

impl wl_surface::Handler for SurfaceHandler {}

declare_handler!(SurfaceHandler, wl_surface::Handler, wl_surface::WlSurface);
