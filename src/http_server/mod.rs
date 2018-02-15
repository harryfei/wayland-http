mod state;
mod window_websocket;

use actix_web::*;
use std::sync::Arc;
use http::header;
use window_manager::WindowManager;

use self::state::State;
use self::window_websocket::WindowStreamWs;

#[derive(Fail, Debug)]
enum HttpApiError {
    #[fail(display = "internal error")]
    InternalError,
    #[fail(display = "invalid window id")]
    InvalidWindowId,
    #[fail(display = "window id not exist")]
    WindowIdNotExist,
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

    debug!("has_hdr {}", has_hdr);

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
    info!("Http api server started at {}", "http://127.0.0.1:8080");

    let state = State { window_manager };
    HttpServer::new(move || {
        Application::with_state(state.clone())
            .resource("/window", |r| r.f(all_windows))
            .resource("/window/{id}", |r| r.f(window))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
