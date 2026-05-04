# screen-res

Tiny CLI that prints the primary display's resolution. Built for game launch
scripts on Wayland.

## Build

    make
    make install              # installs to ~/.local/bin
    make install PREFIX=/usr  # or system-wide

Requires SDL3 (`pkg-config sdl3`).

## Usage

    screen-res                # 2560x1440
    screen-res width          # 2560
    screen-res height         # 1440
    screen-res aspect 4:3     # 1920x1440  (largest 4:3 box that fits)
    screen-res aspect 16:9    # 2560x1440
    screen-res aspect 21:9    # 2560x1097

Add `-s` (or `--space`) to get space-separated output, ready for `read`:

    screen-res -s             # 2560 1440
    screen-res -s aspect 4:3  # 1920 1440

The aspect separator may be `:`, `x`, or `/`.

Exit codes: `0` on success, `1` on SDL/display errors, `2` on bad arguments.
Errors go to stderr; results to stdout.

## Shell examples

    # split into W and H
    read W H < <(screen-res -s aspect 4:3)
    mygame --width "$W" --height "$H"

    # use directly
    mygame --resolution "$(screen-res)"
    mygame --width "$(screen-res width)"

## Notes

- **Native pixels always.** Reports the panel's physical pixel count regardless
  of GNOME/KDE desktop scaling (100%, 125%, 150%, 200%, ...). Desktop scaling
  is a per-surface concept on Wayland; the underlying `wl_output` mode is the
  panel's true resolution, which is what fullscreen games want.
- **"Current" screen:** Wayland does not expose the cursor's output to clients
  without a focused surface, so this tool always reports the primary display.
  Detecting the launching terminal's output would require compositor-specific
  protocols and is intentionally out of scope.
