use std::sync::{Arc, Mutex, MutexGuard};
use window_manager::Buffer;
use failure::Error;
use window_manager::WindowManager;
use window_manager::WindowMetaData;
use sugar::SResultExt;
use futures::sync::mpsc::{channel as fchannel, Receiver};
use smithay::wayland::shell::ToplevelSurface;
use smithay::wayland::shm;
use window_manager::WindowEvent;
use smithay::wayland::compositor::CompositorToken;

use super::implementation::MyWindowMap;
use super::idata::*;

pub struct WaylandWindowManager {
    pub window_maps: Mutex<MyWindowMap>,

    pub registered_senders: Arc<Mutex<WindowEventSenderRegistry>>,

    pub ctoken: Mutex<CompositorToken<SurfaceData, Roles, CompositorIData>>,
}

impl WaylandWindowManager {
    pub fn insert_window(
        &self,
        toplevel: ToplevelSurface<SurfaceData, Roles, CompositorIData, ()>,
    ) -> Result<(), Error> {
        self.get_window_maps()
            .map(|mut w| w.insert(toplevel))
            .eat_value()
    }

    pub fn refresh(&self) -> Result<(), Error> {
        self.get_window_maps().map(|mut w| w.refresh()).eat_value()
    }

    fn get_window_maps(&self) -> Result<MutexGuard<MyWindowMap>, Error> {
        self.window_maps
            .lock()
            .map_err(|e| format_err!("lock window maps error {}", e.to_string()))
    }

    fn get_ctoken(
        &self,
    ) -> Result<MutexGuard<CompositorToken<SurfaceData, Roles, CompositorIData>>, Error> {
        self.ctoken
            .lock()
            .map_err(|e| format_err!("lock ctoken error {}", e.to_string()))
    }
}

impl WindowManager for WaylandWindowManager {
    fn get_window(&self, id: u64) -> Option<WindowMetaData> {
        match self.window_maps.lock() {
            Ok(w) => w.get_window_metadata(id),
            _ => None,
        }
    }

    fn all_windows(&self) -> Vec<WindowMetaData> {
        match self.window_maps.lock() {
            Ok(w) => w.get_windows_metadata(),
            _ => vec![],
        }
    }

    fn get_event_stream(&self, id: u64) -> Option<Receiver<WindowEvent>> {
        let window_id = id;

        self.window_maps
            .lock()
            .ok()
            .and_then(|w| {
                w.get_surface(window_id).and_then(|surface| {
                    self.ctoken
                        .lock()
                        .ok()
                        .map(|ctoken| ctoken.with_surface_data(surface, |s| s.user_data.id))
                })
            })
            .and_then(|surface_id| {
                self.registered_senders.lock().ok().map(|mut s| {
                    println!("register senders");
                    let (sender, receiver) = fchannel(10);

                    s.insert(surface_id, sender);
                    receiver
                })
            })
    }

    fn next_frame(&self, id: u64) -> Result<(), Error> {
        let w = self.get_window_maps()?;

        let surface = w.get_surface(id)
            .ok_or_else(|| format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        let callback = ctoken
            .with_surface_data(surface, |a| a.user_data.callback_queue.pop_front())
            .ok_or_else(|| format_err!("no callbck in this surface of window({})", id))?;

        use time;
        let timespec = time::get_time();
        let mills: i64 =
            timespec.sec * 1000 as i64 + (f64::from(timespec.nsec) / 1000.0 / 1000.0) as i64;

        callback.done(mills as u32);
        println!("frame done");

        Ok(())
    }

    fn release_buffer(&self, id: u64) -> Result<(), Error> {
        let w = self.get_window_maps()?;

        let surface = w.get_surface(id)
            .ok_or_else(|| format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        ctoken.with_surface_data(surface, |a| {
            if let Some(Some((ref b, (_x, _y)))) = a.buffer {
                b.release();
            }
        });

        println!("release");

        Ok(())
    }

    fn get_buffer(&self, id: u64) -> Result<Buffer, Error> {
        let w = self.get_window_maps()?;

        let surface = w.get_surface(id)
            .ok_or_else(|| format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        ctoken.with_surface_data(surface, |a| {
            match a.buffer {
                // copy buffer data
                Some(Some((ref b, (_x, _y)))) => {
                    let mut result: Result<Buffer, Error> =
                        Err(format_err!("buffer not managed by shm buffer"));

                    let _ = shm::with_buffer_contents(b, |slice, data| {
                        let offset = data.offset as usize;
                        let stride = data.stride as usize;
                        let width = data.width as usize;
                        let height = data.height as usize;
                        let mut new_vec = Vec::with_capacity(width * height * 4);
                        for i in 0..height {
                            new_vec.extend(
                                &slice[(offset + i * stride)..(offset + i * stride + width * 4)],
                            );
                        }

                        result = Ok(Buffer::Update {
                            data: new_vec,
                            size: (data.width as u32, data.height as u32),
                        });
                    });

                    result
                }
                // erase the contents
                Some(None) => Ok(Buffer::Erase),

                // do nothing
                None => Err(format_err!("no such buffer")),
            }
        })
    }
}
