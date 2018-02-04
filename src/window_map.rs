use std::collections::HashMap;
use smithay::wayland::compositor::{CompositorToken, SubsurfaceRole};
use smithay::wayland::compositor::roles::Role;
use smithay::wayland::shell::{ShellSurfaceRole, ToplevelSurface};
use wayland_server::protocol::wl_surface::WlSurface;
use easy_wayland::*;
use window_manager::WindowMetaData;


pub struct Window<U, R, CID, SD> {
    toplevel: ToplevelSurface<U, R, CID, SD>,
}

pub struct WindowMap<U, R, CID, SD> {
    windows: HashMap<u64, Window<U, R, CID, SD>>,
    id_serial: u64,
}

impl<U, R, CID, SD> WindowMap<U, R, CID, SD>
    where
        U: 'static,
        R: Role<SubsurfaceRole> + Role<ShellSurfaceRole> + 'static,
        CID: 'static,
        SD: 'static,
{
    pub fn new(_ctoken: CompositorToken<U, R, CID>) -> Self {
        WindowMap {
            windows: HashMap::new(),
            id_serial: 0,
        }
    }

    pub fn insert(&mut self, toplevel: ToplevelSurface<U, R, CID, SD>) {
        let window = Window {
            toplevel: toplevel,
        };
        self.windows.insert(self.id_serial, window);

        self.id_serial = self.id_serial + 1;
    }

    pub fn get_surface(&self, id: u64) -> Option<&WlSurface> {
        self.windows.get(&id)
            .and_then(|w| w.toplevel.get_surface())
    }

    pub fn get_windows_metadata(&self) -> Vec<WindowMetaData> {
        self.windows.iter()
            .map(|(id, w)| {
                let pid = w.toplevel.get_surface()
                    .map(|s| s.get_client().get_credentials().pid);

                let title = w.toplevel.get_pending_state()
                    .map(|s| s.title);

                WindowMetaData {
                    id: *id,
                    pid,
                    title,
                }
            })
            .collect()
    }

    pub fn get_window_metadata(&self, id: u64) -> Option<WindowMetaData> {
        self.windows.get(&id)
            .map(|w| {
                let pid = w.toplevel.get_surface()
                    .map(|s| s.get_client().get_credentials().pid);

                let title = w.toplevel.get_pending_state()
                    .map(|s| s.title);

                WindowMetaData {
                    id: id,
                    pid,
                    title,
                }
            })
    }

    pub fn refresh(&mut self) {
        self.windows.retain(|_, w| w.toplevel.alive());
    }
}

