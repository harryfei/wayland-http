use std::sync::Arc;
use std::path::Path;

use image;
use time;

use wayland_server::protocol::wl_surface;
use smithay::shm::ShmGlobalToken;
use smithay::shm::BufferData;

use wayland_server::{EventLoopHandle, Client, Resource};
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::protocol::wl_buffer::WlBuffer;
use wayland_server::protocol::wl_callback::WlCallback;

use easy_wayland::EasyEventLoopHandle;

pub struct SurfaceHandler {
    shm_token: Arc<ShmGlobalToken>,
    pending_buffer: Option<WlBuffer>,
}

impl SurfaceHandler {
    pub fn new(shm_token: Arc<ShmGlobalToken>) -> SurfaceHandler {
        SurfaceHandler {
            shm_token: shm_token,
            pending_buffer: None,
        }
    }

    fn argb_to_rgba(&self, argb_color: &[u8]) -> Vec<u8> {
        let rgba_color: Vec<u8> = argb_color.chunks(4)
            .flat_map(|color_bits| {
                let mut new_color_bits = vec![0, 0, 0, 0];

                new_color_bits[0] = color_bits[2];   // red
                new_color_bits[1] = color_bits[1];   // green
                new_color_bits[2] = color_bits[0]; // blue
                new_color_bits[3] = color_bits[3]; // alpha

                return new_color_bits;
            })
            .collect();
        return rgba_color;
    }
}

impl wl_surface::Handler for SurfaceHandler {
    fn attach(&mut self,
              _evqh: &mut EventLoopHandle,
              _client: &Client,
              _resource: &WlSurface,
              _buffer: Option<&WlBuffer>,
              _x: i32,
              _y: i32) {
        println!("attach");
        self.pending_buffer = _buffer.unwrap().clone();
    }

    fn commit(&mut self, _evqh: &mut EventLoopHandle, _client: &Client, _resource: &WlSurface) {
        println!("commit");

        let buffer = self.pending_buffer.as_ref();

        if buffer.is_none() {
            return;
        }

        self.shm_token
            .with_buffer_contents(buffer.unwrap(),
                                  |slice: &[u8], buffer_metadata: BufferData| {
                let color = self.argb_to_rgba(slice);

                image::save_buffer(&Path::new("image.png"),
                                   &color,
                                   buffer_metadata.width as u32,
                                   buffer_metadata.height as u32,
                                   image::RGBA(8))
                    .unwrap()
            });
        buffer.unwrap().release();
    }

    fn frame(&mut self,
             evqh: &mut EventLoopHandle,
             client: &Client,
             resource: &WlSurface,
             callback: WlCallback) {
        let timespec = time::get_time();
        let mills: i64 = timespec.sec as i64 + (timespec.nsec as f64 / 1000.0 / 1000.0) as i64;
        callback.done(mills as u32);
    }
}


declare_handler!(SurfaceHandler, wl_surface::Handler, wl_surface::WlSurface);
