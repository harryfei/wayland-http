use wayland_protocols::unstable::xdg_shell::v6::server::zxdg_toplevel_v6 as xdg_toplevel;
use smithay::wayland::compositor::{compositor_init, CompositorToken, SurfaceUserImplementation};
use smithay::wayland::shell::{shell_init, PopupConfigure, ShellState,
                              ShellSurfaceUserImplementation, ToplevelConfigure};
use std::sync::{Arc, Mutex};
use wayland_server::{EventLoop, StateToken};
use window_manager::WindowEvent;

use super::window_map::WindowMap;
use super::wayland_window_manager::WaylandWindowManager;
use super::idata::*;

pub type MyWindowMap = WindowMap<SurfaceData, Roles, CompositorIData, ()>;

pub fn surface_implementation() -> SurfaceUserImplementation<SurfaceData, Roles, CompositorIData> {
    SurfaceUserImplementation {
        commit: |_, idata, surface, token| {
            token.with_surface_data(surface, |attributes| {
                let id = attributes.user_data.id;

                println!("commit in");
                idata.sender_registry.lock().ok().map(|mut registry| {
                    let result = registry
                        .get_mut(&id)
                        .map(|sender| sender.try_send(WindowEvent::Commit));

                    if let Some(Err(ref r)) = result {
                        println!("surface commit {} {}", r.is_full(), r.is_disconnected());
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

pub fn shell_implementation(
) -> ShellSurfaceUserImplementation<SurfaceData, Roles, CompositorIData, ShellIData, ()> {
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

            idata.serial += 1;
            ToplevelConfigure {
                size: None,
                states: vec![xdg_toplevel::State::Activated],
                serial: idata.serial,
            }
        },
        new_popup: |_, _, _| PopupConfigure {
            size: (10, 10),
            position: (10, 10),
            serial: 42,
        },
        move_: |_, _, _, _, _| {},
        resize: |_, _, _, _, _, _| {},
        grab: |_, _, _, _, _| {},
        change_display_state: |_, _, _, _, _, _, _| ToplevelConfigure {
            size: None,
            states: vec![],
            serial: 42,
        },
        show_window_menu: |_, _, _, _, _, _, _| {},
    }
}

pub fn init_shell(
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
        None,
    );

    let window_maps = WindowMap::<_, _, _, ()>::new(compositor_token);

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

    (
        compositor_token,
        shell_state_token,
        wayland_window_manager.clone(),
    )
}
