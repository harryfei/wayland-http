extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_timer;
extern crate http;
extern crate libc;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use(slog_o, slog_info, slog_debug, slog_log, slog_record, slog_record_static, slog_b,
            slog_kv)]
extern crate slog;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
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
mod window_manager;

fn app() {
    let window_map = wayland::run_wayland_thread();
    http_server::run_http_server(window_map);
}

fn main() {
    use slog::{Drain, Logger};
    // A logger facility, here we use the terminal for this example
    let log = Logger::root(
        slog_async::Async::default(slog_term::term_full().fuse()).fuse(),
        slog_o!(),
    );

    // Make sure to save the guard, see documentation for more information
    let _guard = slog_scope::set_global_logger(log);

    app();
}
