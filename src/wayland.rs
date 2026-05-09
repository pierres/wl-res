//! Wayland output enumeration.
//!
//! For newcomers:
//! - Wayland Protocol: https://wayland-book.com/
//! - wayland-client crate: https://docs.rs/wayland-client/latest/wayland_client/
//!
//! The Wayland protocol is asynchronous. We send requests and wait for events.
//! To get display information, we must:
//! 1. Connect to the compositor.
//! 2. Get the "Registry" (the list of all available things/globals on the server).
//! 3. Identify and "Bind" to 'wl_output' globals.
//! 4. Wait for 'wl_output' to send its geometry and mode (resolution) events.

use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
    protocol::{wl_output, wl_registry},
};

/// Represents a physical or virtual display.
#[derive(Clone, Debug)]
pub struct Output {
    /// Unique ID for this output in the registry.
    pub global_name: u32,
    /// Horizontal position in the compositor's layout.
    pub x: i32,
    /// Vertical position in the compositor's layout.
    pub y: i32,
    /// Physical width in pixels.
    pub width: i32,
    /// Physical height in pixels.
    pub height: i32,
    /// Orientation of the display (Normal, 90, 180, 270, etc.).
    pub transform: wl_output::Transform,
    /// Whether we've successfully received a "Current" mode event.
    pub has_current_mode: bool,
    /// Whether the compositor sent the 'Done' event, indicating all data is received.
    pub is_done: bool,
}

// In Rust, 'Default' allows creating a type with sane default values.
impl Default for Output {
    fn default() -> Self {
        Self {
            global_name: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            transform: wl_output::Transform::Normal,
            has_current_mode: false,
            is_done: false,
        }
    }
}

impl Output {
    /// Returns the resolution as the user sees it (swapping width/height if rotated).
    pub fn logical_resolution(&self) -> (i32, i32) {
        match self.transform {
            // If rotated 90 or 270 degrees, the physical width becomes the logical height.
            wl_output::Transform::_90
            | wl_output::Transform::_270
            | wl_output::Transform::Flipped90
            | wl_output::Transform::Flipped270 => (self.height, self.width),
            _ => (self.width, self.height),
        }
    }
}

/// This is our application state. It holds the list of outputs we find.
struct State {
    outputs: Vec<Output>,
}

/// The 'Dispatch' trait is how we handle Wayland events.
/// This implementation handles events from the 'wl_registry'.
impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            // A new global object appeared on the server.
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } if interface == wl_output::WlOutput::interface().name => {
                // We bind to the 'wl_output' interface to start receiving display events.
                // We use version 4 because it is supported by almost all compositors
                // and includes the essential 'Done' event (added in v2).
                let v = version.min(4);
                registry.bind::<wl_output::WlOutput, u32, Self>(name, v, qh, name);
                state.outputs.push(Output {
                    global_name: name,
                    ..Default::default()
                });
            }
            // A global object was removed.
            wl_registry::Event::GlobalRemove { name } => {
                state.outputs.retain(|o| o.global_name != name);
            }
            _ => {}
        }
    }
}

/// This implementation handles events from the 'wl_output' objects we bound to.
impl Dispatch<wl_output::WlOutput, u32> for State {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        global_name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Find the Output in our state that matches this event's source.
        let Some(out) = state
            .outputs
            .iter_mut()
            .find(|o| o.global_name == *global_name)
        else {
            return;
        };

        match event {
            // Sent when the output's location or physical properties are known.
            wl_output::Event::Geometry {
                x, y, transform, ..
            } => {
                out.x = x;
                out.y = y;
                // 'WEnum' is an enum that can hold either a known value or a raw integer.
                if let WEnum::Value(t) = transform {
                    out.transform = t;
                }
            }
            // Sent for every supported resolution/refresh rate.
            wl_output::Event::Mode {
                flags,
                width,
                height,
                ..
            } => {
                // We only care about the mode currently being used by the display.
                let is_current = match flags {
                    WEnum::Value(f) => f.contains(wl_output::Mode::Current),
                    WEnum::Unknown(u) => (u & wl_output::Mode::Current.bits()) != 0,
                };
                if is_current {
                    out.width = width;
                    out.height = height;
                    out.has_current_mode = true;
                }
            }
            // 'Done' is critical: it means the compositor has finished sending
            // the initial set of properties for this output.
            wl_output::Event::Done => {
                out.is_done = true;
            }
            _ => {}
        }
    }
}

/// The main entry point for Wayland communication.
pub fn query_outputs() -> Result<Vec<Output>, String> {
    // 1. Establish connection to the Wayland server (usually via $WAYLAND_DISPLAY).
    let conn =
        Connection::connect_to_env().map_err(|e| format!("Failed to connect to Wayland: {e}"))?;

    // 2. The display object is the root of the protocol.
    let display = conn.display();

    // 3. Create an event queue to process incoming messages.
    let mut event_queue = conn.new_event_queue::<State>();
    let qh = event_queue.handle();

    // 4. Request the registry.
    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        outputs: Vec::new(),
    };

    // 5. Run a 'roundtrip'. This blocks until all current requests are sent
    // and all resulting events are processed.
    // This first roundtrip fills our 'state.outputs' by processing 'Global' events.
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| format!("Wayland registry roundtrip failed: {e}"))?;

    // If no displays were even found, we can stop here.
    if state.outputs.is_empty() {
        return Ok(state.outputs);
    }

    // 6. Keep running roundtrips until all bound outputs have sent their 'Done' event.
    // Technically, one roundtrip is enough for a compliant server, but we use a
    // small loop (10 is an arbitrary 'safe' number) as a failsafe against buggy
    // compositors or extreme system lag. This ensures we don't hang forever.
    for _ in 0..10 {
        if !state.outputs.is_empty() && state.outputs.iter().all(|o| o.is_done) {
            break;
        }
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| format!("Wayland output roundtrip failed: {e}"))?;
    }

    Ok(state.outputs)
}

/// Wayland has no formal "primary output" concept.
///
/// However, most compositors (GNOME, KDE, Sway) place the user's "Main" monitor
/// at the origin (0, 0) of the global coordinate system. This function picks
/// that display, falling back to the first available one if no display is at
/// the origin.
///
/// Note: CLI tools in Wayland cannot see the mouse cursor (for security), so
/// we cannot "pick the screen with the mouse" like some GUI apps do.
pub fn pick_primary(outputs: &[Output]) -> Option<&Output> {
    outputs
        .iter()
        .find(|o| o.has_current_mode && o.x == 0 && o.y == 0)
        .or_else(|| outputs.iter().find(|o| o.has_current_mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create an Output for testing.
    fn make(global_name: u32, x: i32, y: i32, width: i32, height: i32) -> Output {
        Output {
            global_name,
            x,
            y,
            width,
            height,
            transform: wl_output::Transform::Normal,
            has_current_mode: width > 0 && height > 0,
            is_done: true,
        }
    }

    #[test]
    fn logical_resolution_swaps_on_90_degree_rotation() {
        let mut out = make(1, 0, 0, 1080, 1920);
        out.transform = wl_output::Transform::_90;
        assert_eq!(out.logical_resolution(), (1920, 1080));
    }

    #[test]
    fn logical_resolution_swaps_on_270_degree_rotation() {
        let mut out = make(1, 0, 0, 1080, 1920);
        out.transform = wl_output::Transform::_270;
        assert_eq!(out.logical_resolution(), (1920, 1080));
    }

    #[test]
    fn logical_resolution_keeps_normal() {
        let out = make(1, 0, 0, 1920, 1080);
        assert_eq!(out.logical_resolution(), (1920, 1080));
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
