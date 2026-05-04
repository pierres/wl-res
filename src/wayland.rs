//! Wayland output enumeration.
//!
//! Two roundtrips are enough: the first drains `wl_registry::global` events
//! and binds every advertised `wl_output`; the second drains the per-output
//! `geometry`/`mode`/`done` events that follow.

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle, WEnum,
};

#[derive(Default, Clone, Debug)]
pub struct Output {
    pub global_name: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub has_current_mode: bool,
}

struct State {
    outputs: Vec<Output>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_output" {
                let v = version.min(4);
                registry.bind::<wl_output::WlOutput, u32, Self>(name, v, qh, name);
                state.outputs.push(Output {
                    global_name: name,
                    ..Default::default()
                });
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for State {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        global_name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(out) = state
            .outputs
            .iter_mut()
            .find(|o| o.global_name == *global_name)
        else {
            return;
        };
        match event {
            wl_output::Event::Geometry { x, y, .. } => {
                out.x = x;
                out.y = y;
            }
            wl_output::Event::Mode {
                flags,
                width,
                height,
                ..
            } => {
                if let WEnum::Value(f) = flags {
                    if f.contains(wl_output::Mode::Current) {
                        out.width = width;
                        out.height = height;
                        out.has_current_mode = true;
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn query_outputs() -> Result<Vec<Output>, String> {
    let conn = Connection::connect_to_env()
        .map_err(|e| format!("Failed to connect to Wayland: {e}"))?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue::<State>();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        outputs: Vec::new(),
    };
    for _ in 0..2 {
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| format!("Wayland roundtrip failed: {e}"))?;
    }
    Ok(state.outputs)
}

/// Wayland has no formal "primary output" concept; this picks the output
/// anchored at (0, 0), falling back to the first output that reported a mode.
pub fn pick_primary(outputs: &[Output]) -> Option<&Output> {
    outputs
        .iter()
        .find(|o| o.has_current_mode && o.x == 0 && o.y == 0)
        .or_else(|| outputs.iter().find(|o| o.has_current_mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(global_name: u32, x: i32, y: i32, width: i32, height: i32) -> Output {
        Output {
            global_name,
            x,
            y,
            width,
            height,
            has_current_mode: width > 0 && height > 0,
        }
    }

    #[test]
    fn pick_primary_prefers_origin_anchored_output() {
        let outputs = vec![
            make(1, 2560, 0, 1920, 1080),
            make(2, 0, 0, 2560, 1440),
            make(3, -1920, 0, 1920, 1080),
        ];
        assert_eq!(pick_primary(&outputs).unwrap().global_name, 2);
    }

    #[test]
    fn pick_primary_falls_back_to_first_with_mode() {
        let outputs = vec![
            Output {
                global_name: 1,
                ..Default::default()
            },
            make(2, 1000, 500, 1920, 1080),
            make(3, 3000, 500, 2560, 1440),
        ];
        assert_eq!(pick_primary(&outputs).unwrap().global_name, 2);
    }

    #[test]
    fn pick_primary_returns_none_when_no_modes_reported() {
        let outputs = vec![
            Output {
                global_name: 1,
                ..Default::default()
            },
            Output {
                global_name: 2,
                ..Default::default()
            },
        ];
        assert!(pick_primary(&outputs).is_none());
    }

    #[test]
    fn pick_primary_returns_none_for_empty() {
        assert!(pick_primary(&[]).is_none());
    }
}
