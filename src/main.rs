extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_timer;
extern crate http;
extern crate image;
extern crate libc;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate smithay;
extern crate sugar;
extern crate time;
extern crate wayland_protocols;
extern crate wayland_server;
#[macro_use]
extern crate wayland_sys;

mod wayland;
mod http_server;
mod window_map;
mod easy_wayland;
mod window_manager;

fn main() {
    let window_map = wayland::run_wayland_thread();
    http_server::run_http_server(window_map);
}
