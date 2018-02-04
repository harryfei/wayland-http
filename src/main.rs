extern crate wayland_server;
extern crate wayland_protocols;
#[macro_use]
extern crate wayland_sys;
#[macro_use]
extern crate smithay;
extern crate image;
extern crate time;
extern crate actix;
extern crate actix_web;
extern crate libc;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate http;
extern crate futures;
extern crate sugar;
extern crate futures_timer;


mod wayland;
mod http_server;
mod window_map;
mod easy_wayland;
mod window_manager;

fn main() {
    let window_map = wayland::run_wayland_thread();
    http_server::run_http_server(window_map);
}
