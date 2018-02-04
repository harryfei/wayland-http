use wayland_server::Client;
use wayland_server::Resource;
use wayland_sys::server::*;
use libc::{pid_t, uid_t, gid_t};

pub struct Credentials {
    pub pid: pid_t,
    pub uid: uid_t,
    pub gid: gid_t,
}

pub trait ResourceExt {
    fn get_client(&self) -> Client;
}
impl<T: Resource> ResourceExt for T {
    fn get_client(&self) -> Client {
        unsafe {
            let wl_client_ptr = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_resource_get_client, self.ptr());
            Client::from_ptr(wl_client_ptr)
        }
    }
}


pub trait ClientExt {
    fn get_credentials(&self) -> Credentials;
}

impl ClientExt for Client {

    fn get_credentials(&self) -> Credentials {

        let mut pid: pid_t = 0;
        let mut uid: uid_t = 0;
        let mut gid: gid_t = 0;

        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_client_get_credentials, self.ptr(),
                &mut pid as *mut pid_t, &mut uid as *mut uid_t, &mut gid as *mut gid_t)
        };

        Credentials {
            pid: pid,
            uid: uid,
            gid: gid,
        }
    }
}