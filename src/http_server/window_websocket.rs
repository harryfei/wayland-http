use actix::*;
use actix_web::*;
use window_manager::WindowEvent;
use window_manager::Buffer;

use super::state::State;

struct NextFrame;

impl ResponseType for NextFrame {
    type Item = ();
    type Error = ();
}

pub struct WindowStreamWs {
    window_id: u64,
}

impl WindowStreamWs {
    pub fn new(window_id: u64) -> Self {
        Self { window_id }
    }
}

impl Actor for WindowStreamWs {
    type Context = ws::WebsocketContext<Self, State>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let window_event_stream = ctx.state().window_manager.get_event_stream(self.window_id);

        if let Some(s) = window_event_stream {
            ctx.add_message_stream(s);
        }

        // render the buffer once websocket connected
        ctx.notify(WindowEvent::Commit);
    }
}

/// Define Handler for `ws::Message` message
impl Handler<ws::Message> for WindowStreamWs {
    type Result = ();

    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(_text) => {
                // The web only send next_frame event now.
                ctx.notify(NextFrame);
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Closed | ws::Message::Error => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl ResponseType for WindowEvent {
    type Item = ();
    type Error = ();
}

impl Handler<WindowEvent> for WindowStreamWs {
    type Result = ();

    fn handle(&mut self, msg: WindowEvent, ctx: &mut Self::Context) {
        match msg {
            WindowEvent::Commit => {
                // copy buffer data
                let buffer = (&ctx.state().window_manager).get_buffer(self.window_id);

                // release buffer
                let _ = (&ctx.state().window_manager).release_buffer(self.window_id);

                debug!("commit");

                match buffer {
                    Ok(Buffer::Update {
                        data,
                        size: (width, height),
                    }) => {
                        ctx.text(&json!({
                            "width": width,
                            "height": height,
                        }).to_string());

                        debug!("{} {}", width, height);

                        ctx.binary(data);
                    }
                    Ok(Buffer::Erase) => {
                        debug!("erase");
                    }
                    Err(e) => {
                        debug!("{}", e);
                    }
                }
            }
        }
    }
}

impl Handler<NextFrame> for WindowStreamWs {
    type Result = ();

    fn handle(&mut self, _msg: NextFrame, ctx: &mut Self::Context) {
        let _ = ctx.state().window_manager.next_frame(self.window_id);
    }
}
