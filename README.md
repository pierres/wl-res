# wl-res

Tiny CLI that prints the primary display's resolution. Built for game launch
scripts on Wayland.

Pure-Rust Wayland client — no SDL, no libwayland-client at runtime. The release
binary depends only on libc.

## Build

    cargo build --release           # binary at ./target/release/wl-res
    cargo install --path .          # installs to ~/.cargo/bin

## Usage

    wl-res                # 2560x1440
    wl-res width          # 2560
    wl-res height         # 1440
    wl-res aspect 4:3     # 1920x1440  (largest 4:3 box that fits)
    wl-res aspect 16:9    # 2560x1440
    wl-res aspect 21:9    # 2560x1097

Add `-s` (or `--space`) to get space-separated output, ready for `read`:

    wl-res -s             # 2560 1440
    wl-res -s aspect 4:3  # 1920 1440

The aspect separator may be `:`, `x`, or `/`.

Exit codes: `0` on success, `1` on Wayland/display errors, `2` on bad arguments.
Errors go to stderr; results to stdout.

## Shell examples

    # split into W and H
    read W H < <(wl-res -s aspect 4:3)
    mygame --width "$W" --height "$H"

    # use directly
    mygame --resolution "$(wl-res)"
    mygame --width "$(wl-res width)"

## Notes

- **Native pixels always.** Reports the panel's physical pixel count regardless
  of GNOME/KDE desktop scaling (100%, 125%, 150%, 200%, ...). Desktop scaling
  is a per-surface concept on Wayland; the underlying `wl_output` mode is the
  panel's true resolution, which is what fullscreen games want.
- **"Primary" screen:** Wayland has no formal "primary output" concept. This
  tool picks the output anchored at coordinate (0, 0), falling back to the
  first one announced by the compositor. Detecting the *launching terminal's*
  output would require compositor-specific protocols and is intentionally out
  of scope.
