use actix::*;
use actix_web::*;
use std::sync::Arc;
use http::header;
use window_manager::WindowManager;
use window_manager::WindowEvent;
use window_manager::Buffer;

#[derive(Fail, Debug)]
enum HttpApiError {
    #[fail(display = "internal error")] InternalError,
    #[fail(display = "invalid window id")] InvalidWindowId,
    #[fail(display = "window id not exist")] WindowIdNotExist,
}

impl error::ResponseError for HttpApiError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            HttpApiError::InternalError => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR, Body::Empty)
            }
            HttpApiError::InvalidWindowId => {
                HttpResponse::new(StatusCode::BAD_REQUEST, Body::Empty)
            }
            HttpApiError::WindowIdNotExist => HttpResponse::new(StatusCode::NOT_FOUND, Body::Empty),
        }
    }
}

#[derive(Clone)]
struct State {
    window_manager: Arc<WindowManager>,
}

struct NextFrame;

impl ResponseType for NextFrame {
    type Item = ();
    type Error = ();
}

struct WindowStreamWs {
    window_id: u64,
}

impl WindowStreamWs {
    fn new(window_id: u64) -> Self {
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

                println!("commit");

                match buffer {
                    Ok(Buffer::Update {
                        data,
                        size: (width, height),
                    }) => {
                        ctx.text(&json!({
                            "width": width,
                            "height": height,
                        }).to_string());

                        println!("{} {}", width, height);

                        ctx.binary(data);
                    }
                    Ok(Buffer::Erase) => {
                        println!("erase");
                    }
                    Err(e) => {
                        println!("{}", e);
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

fn window(req: HttpRequest<State>) -> Result<HttpResponse, HttpApiError> {
    let has_hdr = if let Some(hdr) = req.headers().get(header::UPGRADE) {
        if let Ok(s) = hdr.to_str() {
            s.to_lowercase().contains("websocket")
        } else {
            false
        }
    } else {
        false
    };

    println!("has_hdr {}", has_hdr);

    let id = req.match_info()["id"]
        .parse::<u64>()
        .map_err(|_| HttpApiError::InvalidWindowId)?;

    if !has_hdr {
        let wm = &req.state().window_manager;

        let window = wm.get_window(id).ok_or(HttpApiError::WindowIdNotExist)?;

        httpcodes::HTTPOk
            .build()
            .json(window)
            .map_err(|_| HttpApiError::InternalError)
    } else {
        ws::start(req, WindowStreamWs::new(id)).map_err(|_| HttpApiError::InternalError)
    }
}

fn all_windows(req: HttpRequest<State>) -> Result<HttpResponse, Error> {
    let windows_metas = &req.state().window_manager.all_windows();
    httpcodes::HTTPOk.build().json(windows_metas)
}

pub fn run_http_server(window_manager: Arc<WindowManager>) {
    let state = State { window_manager };
    HttpServer::new(move || {
        Application::with_state(state.clone())
            .resource("/window", |r| r.f(all_windows))
            .resource("/window/{id}", |r| r.f(window))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
