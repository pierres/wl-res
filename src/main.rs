//! screen-res — print the primary Wayland display's resolution.
//!
//! Talks Wayland directly via `wl_registry` + `wl_output`, no SDL dependency.
//! Reports native panel pixels (the `wl_output::mode` event always carries
//! physical pixels, independent of any per-surface fractional scaling).

use std::env;
use std::process::ExitCode;

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle, WEnum,
};

#[derive(Default)]
struct Output {
    name: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    has_current_mode: bool,
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
                    name,
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
        let Some(out) = state.outputs.iter_mut().find(|o| o.name == *global_name) else {
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

fn parse_aspect(s: &str) -> Option<(i64, i64)> {
    for sep in [':', 'x', '/'] {
        if let Some((a, b)) = s.split_once(sep) {
            let (Ok(aw), Ok(ah)) = (a.parse::<i64>(), b.parse::<i64>()) else {
                continue;
            };
            if aw > 0 && ah > 0 {
                return Some((aw, ah));
            }
        }
    }
    None
}

fn usage(prog: &str) -> ExitCode {
    eprintln!("Usage: {prog} [-s|--space] [resolution|width|height|aspect W:H]");
    ExitCode::from(2)
}

fn pick_primary(outputs: &[Output]) -> Option<&Output> {
    // Wayland has no formal "primary" concept; treat the output anchored at
    // (0, 0) as primary, falling back to whichever output reported a mode first.
    outputs
        .iter()
        .find(|o| o.has_current_mode && o.x == 0 && o.y == 0)
        .or_else(|| outputs.iter().find(|o| o.has_current_mode))
}

fn main() -> ExitCode {
    let conn = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to Wayland: {e}");
            return ExitCode::FAILURE;
        }
    };

    let display = conn.display();
    let mut event_queue = conn.new_event_queue::<State>();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        outputs: Vec::new(),
    };

    // First roundtrip drains globals; second drains per-output info events.
    for _ in 0..2 {
        if let Err(e) = event_queue.roundtrip(&mut state) {
            eprintln!("Wayland roundtrip failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    let Some(out) = pick_primary(&state.outputs) else {
        eprintln!("No usable Wayland output found");
        return ExitCode::FAILURE;
    };

    let w = out.width as i64;
    let h = out.height as i64;

    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(|s| s.as_str()).unwrap_or("screen-res");
    let mut sep = 'x';
    let mut positional: Vec<&str> = Vec::new();
    for a in &args[1..] {
        match a.as_str() {
            "-s" | "--space" => sep = ' ',
            other => positional.push(other),
        }
    }

    let cmd = positional.first().copied().unwrap_or("resolution");
    match cmd {
        "resolution" => println!("{w}{sep}{h}"),
        "width" => println!("{w}"),
        "height" => println!("{h}"),
        "aspect" => {
            let Some(spec) = positional.get(1) else {
                return usage(prog);
            };
            let Some((aw, ah)) = parse_aspect(spec) else {
                eprintln!("Invalid aspect ratio {spec} (expected e.g. 4:3)");
                return ExitCode::from(2);
            };
            let (mut tw, mut th) = (h * aw / ah, h);
            if tw > w {
                tw = w;
                th = w * ah / aw;
            }
            println!("{tw}{sep}{th}");
        }
        _ => return usage(prog),
    }

    ExitCode::SUCCESS
}
