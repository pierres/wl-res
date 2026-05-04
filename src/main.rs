//! wl-res — print the primary Wayland display's resolution.

mod aspect;
mod wayland;

use std::env;
use std::process::ExitCode;

use crate::aspect::{fit_aspect, parse_aspect};
use crate::wayland::{pick_primary, query_outputs};

fn usage(prog: &str) -> ExitCode {
    eprintln!("Usage: {prog} [-s|--space] [resolution|width|height|aspect W:H]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let outputs = match query_outputs() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let Some(out) = pick_primary(&outputs) else {
        eprintln!("No usable Wayland output found");
        return ExitCode::FAILURE;
    };

    let (w, h) = (out.width as i64, out.height as i64);

    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(|s| s.as_str()).unwrap_or("wl-res");
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
            let (tw, th) = fit_aspect(w, h, aw, ah);
            println!("{tw}{sep}{th}");
        }
        _ => return usage(prog),
    }

    ExitCode::SUCCESS
}
