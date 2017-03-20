use wayland_server::protocol::wl_shm::WlShm;
use wayland_server::protocol::wl_shm::Handler;
use wayland_server::protocol::wl_shm::Format;
use wayland_server::protocol::wl_shm_pool;
use std::os::unix::io::RawFd;
// use wayland_sys::server::*;

use wayland_server::{
    GlobalHandler,
    EventLoopHandle,
    Client,
    Display,
};

use easy_wayland::EasyEventLoopHandle;

struct ShmPoolHandler;
impl wl_shm_pool::Handler for ShmPoolHandler {
    
}

declare_handler!(ShmPoolHandler, wl_shm_pool::Handler, wl_shm_pool::WlShmPool);

struct ShmHandler;

impl Handler for ShmHandler {
    fn create_pool(&mut self,
                   evqh: &mut EventLoopHandle,
                   client: &Client,
                   resource: &WlShm,
                   id: wl_shm_pool::WlShmPool,
                   fd: RawFd,
                   size: i32) {

        
    }
}

declare_handler!(ShmHandler, Handler, WlShm);

pub struct BindHandler {}

impl GlobalHandler<WlShm> for BindHandler {
    fn bind(&mut self,
            evlh: &mut EventLoopHandle,
            _client: &Client,
            shm: WlShm
    ) {
        evlh.easy_register(&shm, ShmHandler{});

        shm.format(Format::Argb8888);
    }
}

// pub fn init_shm(display: &Display) {
//     let dpy = display.ptr();
//     unsafe {
//         ffi_dispatch!(
//             WAYLAND_SERVER_HANDLE,
//             wl_display_init_shm,
//             dpy
//         );

//         ffi_dispatch!(
//             WAYLAND_SERVER_HANDLE,
//             wl_display_add_shm_format,
//             dpy,
//             Format::Argb8888 as u32
//         );
//     }
    
// }
