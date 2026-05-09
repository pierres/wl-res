//! wl-res — print the primary Wayland display's resolution.
//!
//! For newcomers:
//! - Rust Main & Args: https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html
//! - Match Statement: https://doc.rust-lang.org/book/ch06-02-match-control-flow-intro.html

mod aspect;
mod wayland;

use std::env;
use std::process::ExitCode;

use crate::aspect::{fit_aspect, parse_aspect};
use crate::wayland::{pick_primary, query_outputs};

const HELP: &str = "\
wl-res — Tiny Wayland display resolution tool

Usage: wl-res [OPTIONS] [COMMAND]

Commands:
  resolution      Print current resolution (e.g. 2560x1440) [default]
  width           Print current width only
  height          Print current height only
  aspect W:H      Print largest resolution with aspect ratio W:H that fits

Options:
  -s, --space     Use a space separator instead of 'x'
  -h, --help      Print this help message
  -v, --version   Print version information";

fn main() -> ExitCode {
    // env::args() is an iterator over the command line arguments.
    // .collect() turns the iterator into a Vec (list).
    let args: Vec<String> = env::args().collect();
    let mut sep = 'x';
    let mut positional: Vec<&str> = Vec::new();

    // Simple manual argument parsing.
    for a in &args[1..] {
        match a.as_str() {
            "-s" | "--space" => sep = ' ',
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "-v" | "--version" => {
                println!("wl-res {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            // Starts with '-' but not recognized.
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                eprintln!("\n{HELP}");
                return ExitCode::from(2);
            }
            // Not an option, so it's a command or ratio.
            other => positional.push(other),
        }
    }

    // 1. Ask Wayland for all displays.
    let outputs = match query_outputs() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    // 2. Pick the "primary" one (at 0,0).
    let Some(out) = pick_primary(&outputs) else {
        eprintln!("No usable Wayland output found");
        return ExitCode::FAILURE;
    };

    // 3. Get the logical resolution (handles rotation).
    let (w, h) = {
        let (lw, lh) = out.logical_resolution();
        (lw as i64, lh as i64)
    };

    // 4. Execute the requested command.
    let cmd = positional.first().copied().unwrap_or("resolution");
    match cmd {
        "resolution" => println!("{w}{sep}{h}"),
        "width" => println!("{w}"),
        "height" => println!("{h}"),
        "aspect" => {
            let Some(spec) = positional.get(1) else {
                eprintln!("Error: 'aspect' command requires a ratio (e.g. 16:9)");
                eprintln!("\n{HELP}");
                return ExitCode::from(2);
            };
            let Some((aw, ah)) = parse_aspect(spec) else {
                eprintln!("Invalid aspect ratio {spec} (expected e.g. 4:3)");
                return ExitCode::from(2);
            };
            let (tw, th) = fit_aspect(w, h, aw, ah);
            println!("{tw}{sep}{th}");
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("\n{HELP}");
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}
