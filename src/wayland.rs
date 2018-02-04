use std::thread;

use wayland_server::create_display;
use wayland_protocols::unstable::xdg_shell::v6::server::zxdg_toplevel_v6 as xdg_toplevel;
use smithay::wayland::shm::init_shm_global;
use wayland_server::protocol::wl_shm::Format;
use wayland_server::protocol::wl_callback::WlCallback;

use std::sync::mpsc::channel;

use window_map::WindowMap;
use smithay::wayland::compositor::{compositor_init, CompositorToken,
                                   SurfaceUserImplementation};
use smithay::wayland::shell::{shell_init, PopupConfigure, ShellState, ShellSurfaceRole,
                              ShellSurfaceUserImplementation, ToplevelConfigure, ToplevelSurface};
use smithay::wayland::shm;
use std::sync::{Arc, Mutex, MutexGuard};
use wayland_server::{EventLoop, StateToken};
use std::collections::{HashMap, VecDeque};
use window_manager::{WindowEvent, WindowManager, WindowMetaData};
use futures::sync::mpsc::{channel as fchannel, Sender, Receiver};
use failure::Error;
use window_manager::Buffer;
use sugar::SResultExt;


define_roles!(Roles => [ ShellSurface, ShellSurfaceRole ] );

type WindowEventSenderRegistry = HashMap<u64, Sender<WindowEvent>>;

pub type MyWindowMap = WindowMap<
    SurfaceData,
    Roles,
    CompositorIData,
    (),
>;

struct WaylandWindowManager {
    window_maps: Mutex<MyWindowMap>,

    registered_senders: Arc<Mutex<WindowEventSenderRegistry>>,

    ctoken: Mutex<CompositorToken<SurfaceData, Roles, CompositorIData>>,

}

impl WaylandWindowManager {
    fn insert_window(&self, toplevel: ToplevelSurface<SurfaceData, Roles, CompositorIData, ()>)
        -> Result<(), Error>
    {
        self.get_window_maps()
            .map(|mut w| w.insert(toplevel))
            .eat_value()
    }

    fn refresh(&self) -> Result<(), Error> {
        self.get_window_maps()
            .map(|mut w| w.refresh())
            .eat_value()
    }

    fn get_window_maps(&self) -> Result<MutexGuard<MyWindowMap>, Error> {
        self.window_maps.lock()
            .map_err(|e| format_err!("lock window maps error {}", e.to_string()))
    }

    fn get_ctoken(&self) -> Result<MutexGuard<CompositorToken<SurfaceData, Roles, CompositorIData>>, Error> {
        self.ctoken.lock()
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
            _ => vec!(),
        }
    }

    fn get_event_stream(&self, id: u64) -> Option<Receiver<WindowEvent>> {
        let window_id = id;

        self.window_maps.lock()
            .ok()
            .and_then(|w| {
                w.get_surface(window_id)
                    .and_then(|surface| {
                        self.ctoken.lock()
                            .ok()
                            .map(|ctoken| {
                                ctoken.with_surface_data(surface, |s| s.user_data.id)
                            })
                    })
            })
            .and_then(|surface_id| {


                self.registered_senders.lock()
                    .ok()
                    .map(|mut s| {
                        println!("register senders");
                        let (sender, receiver) = fchannel(10);

                        s.insert(surface_id, sender);
                        receiver
                    })
            })
    }

    fn next_frame(&self, id: u64) -> Result<(), Error> {
        let w = self.get_window_maps()?;

        let surface = w
            .get_surface(id)
            .ok_or(format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        let callback = ctoken.with_surface_data(surface, |a| {
            a.user_data.callback_queue.pop_front()
        })
        .ok_or(format_err!("no callbck in this surface of window({})", id))?;

        use time;
        let timespec = time::get_time();
        let mills: i64 =
            timespec.sec * 1000 as i64 + (timespec.nsec as f64 / 1000.0 / 1000.0) as i64;

        callback.done(mills as u32);
        println!("frame done");

        Ok(())
    }

    fn release_buffer(&self, id: u64) -> Result<(), Error> {
        let w = self.get_window_maps()?;

        let surface = w
            .get_surface(id)
            .ok_or(format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        ctoken.with_surface_data(surface, |a| {
            if let Some(Some((ref b, (_x, _y)))) = a.buffer { b.release(); }
        });

        println!("release");


        Ok(())
    }

    fn get_buffer(&self, id: u64) -> Result<Buffer, Error> {
        let w = self.get_window_maps()?;

        let surface = w
            .get_surface(id)
            .ok_or(format_err!("no surface of window({})", id))?;

        let ctoken = self.get_ctoken()?;

        let buffer = ctoken.with_surface_data(surface, |a| {
            match a.buffer {

                // copy buffer data
                Some(Some((ref b, (_x, _y)))) => {
                    let mut result: Result<Buffer, Error> = Err(format_err!("buffer not managed by shm buffer"));

                    let _ = shm::with_buffer_contents(b, |slice, data| {
                        let offset = data.offset as usize;
                        let stride = data.stride as usize;
                        let width = data.width as usize;
                        let height = data.height as usize;
                        let mut new_vec = Vec::with_capacity(width * height * 4);
                        for i in 0..height {
                            new_vec
                                .extend(&slice[(offset + i * stride)..(offset + i * stride + width * 4)]);
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
        });

        buffer
    }
}

#[derive(Default)]
pub struct SurfaceData {
    id: u64,

    callback_queue: VecDeque<WlCallback>,
}

pub struct CompositorIData {
    sender_registry: Arc<Mutex<WindowEventSenderRegistry>>,
}

impl CompositorIData {
    fn new(sender_registry: Arc<Mutex<WindowEventSenderRegistry>>) -> Self {
        Self {
            sender_registry,
        }
    }
}

pub fn surface_implementation() -> SurfaceUserImplementation<SurfaceData, Roles, CompositorIData> {
    SurfaceUserImplementation {
        commit: |_, idata, surface, token| {
            token.with_surface_data(surface, |attributes| {
                let id = attributes.user_data.id;

                println!("commit in");
                idata.sender_registry.lock()
                    .ok()
                    .map(|mut registry| {
                        let result = registry.get_mut(&id)
                            .map(|sender| sender.try_send(WindowEvent::Commit));

                        match result {
                            Some(Err(ref r)) => {
                                println!("surface commit {} {}", r.is_full(), r.is_disconnected());
                            },
                            _ => {}
                        }
                        match result {
                            Some(Err(ref r)) if r.is_disconnected() => {
                                registry.remove(&id);
                            }
                            _ => {}
                        }
                    });
            });
        },
        frame: |_, _, surface, callback, token| {
            println!("frame in");

            token.with_surface_data(surface, |attributes| {
                attributes.user_data.callback_queue.push_back(callback);
            });
        },
    }
}

pub struct ShellIData {
    pub token: CompositorToken<SurfaceData, Roles, CompositorIData>,
    window_manager: Arc<WaylandWindowManager>,

    pub serial: u32,

    pub surface_id_serial: u64,

}

pub fn shell_implementation() -> ShellSurfaceUserImplementation<SurfaceData, Roles, CompositorIData, ShellIData, ()>
{
    ShellSurfaceUserImplementation {
        new_client: |_, _, _| {},
        client_pong: |_, _, _| {},
        new_toplevel: |_, idata, toplevel| {

            // assign unique surface id for surface of each toplevel.
            if let Some(s) = toplevel.get_surface() {
                idata.token.with_surface_data(s, |attributes| {
                    let data = &mut (attributes.user_data);
                    data.id = idata.surface_id_serial;
                });

                idata.surface_id_serial += 1;
            }


            // store toplevel in window manager
            let _ = idata.window_manager.insert_window(toplevel);

            idata.serial = idata.serial + 1;
            ToplevelConfigure {
                size: None,
                states: vec![xdg_toplevel::State::Activated],
                serial: idata.serial,
            }
        },
        new_popup: |_, _, _| {
            PopupConfigure {
                size: (10, 10),
                position: (10, 10),
                serial: 42,
            }
        },
        move_: |_, _, _, _, _| {},
        resize: |_, _, _, _, _, _| {},
        grab: |_, _, _, _, _| {},
        change_display_state: |_, _, _, _, _, _, _| {
            ToplevelConfigure {
                size: None,
                states: vec![],
                serial: 42,
            }
        },
        show_window_menu: |_, _, _, _, _, _, _| {},
    }
}

fn init_shell(
    evl: &mut EventLoop,
) -> (
    CompositorToken<SurfaceData, Roles, CompositorIData>,
    StateToken<ShellState<SurfaceData, Roles, CompositorIData, ()>>,
    Arc<WaylandWindowManager>,
) {
    let window_event_sender_registry = Arc::new(Mutex::new(WindowEventSenderRegistry::new()));

    let (compositor_token, _, _) = compositor_init(
        evl,
        surface_implementation(),
        CompositorIData::new(window_event_sender_registry.clone()),
        None);

    let window_maps = WindowMap::<_, _, _, ()>::new(
        compositor_token,
    );

    let wayland_window_manager = Arc::new(WaylandWindowManager {
        window_maps: Mutex::new(window_maps),
        registered_senders: window_event_sender_registry.clone(),
        ctoken: Mutex::new(compositor_token),
    });

    let (shell_state_token, _, _) = shell_init(
        evl,
        compositor_token,
        shell_implementation(),
        ShellIData {
            token: compositor_token,
            window_manager: wayland_window_manager.clone(),
            serial: 0,
            surface_id_serial: 0,
        },
        None,
    );

    (compositor_token, shell_state_token, wayland_window_manager.clone())
}


pub fn run_wayland_thread() -> Arc<WindowManager> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        println!("Wayland server running!");
        let (mut dpy, mut event_loop) = create_display();

        // panic if socket create failed
        dpy.add_socket_auto().unwrap();

        let _shm_global = init_shm_global(
            &mut event_loop,
            vec![Format::Yuyv, Format::C8],
            None, // No logger
        );

        let (_compositor_token, _, window_manager) = init_shell(&mut event_loop);

        sender.send(window_manager.clone()).unwrap();

        loop {
            event_loop.dispatch(Some(16)).unwrap();
            dpy.flush_clients();

            let _ = window_manager.refresh();
        }
    });

    receiver.recv().unwrap()
}
