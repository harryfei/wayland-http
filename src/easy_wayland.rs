use std::any::Any;
use wayland_server::protocol::Resource;

use wayland_server::{GlobalHandler, Handler, EventLoopHandle, EventLoop, Global};

pub trait EasyEventLoop {
    fn easy_register_global<H, R>(&mut self, handler: H, version: i32) -> (Global, usize)
        where H: GlobalHandler<R> + Send + Any + 'static,
              R: Resource;
}

impl EasyEventLoop for EventLoop {
    fn easy_register_global<H, R>(&mut self, handler: H, version: i32) -> (Global, usize)
        where H: GlobalHandler<R> + Send + Any + 'static,
              R: Resource
    {
        let handler_id = self.add_handler(handler);
        let global = self.register_global::<R, H>(handler_id, version);
        return (global, handler_id);
    }
}

pub trait EasyEventLoopHandle {
    fn easy_register<H, R>(&mut self, resource: &R, handler: H) -> usize
        where H: Handler<R> + Send + Any + 'static,
              R: Resource;
}

impl EasyEventLoopHandle for EventLoopHandle {
    fn easy_register<H, R>(&mut self, resource: &R, handler: H) -> usize
        where H: Handler<R> + Send + Any + 'static,
              R: Resource
    {
        let handler_id = self.add_handler(handler);
        self.register::<R, H>(resource, handler_id);
        return handler_id;
    }
}
