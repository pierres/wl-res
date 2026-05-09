# wl-res

**wl-res** is a utility for querying the primary display's resolution on Wayland.
It is designed for game launch scripts.

## Usage

```bash
wl-res                # 2560x1440
wl-res width          # 2560
wl-res height         # 1440
wl-res aspect 4:3     # 1920x1440  (largest 4:3 box that fits inside screen)
wl-res aspect 16:9    # 2560x1440
wl-res aspect 21:9    # 2560x1097
```

### Options

- `-s, --space`: Use a space separator instead of `x` (useful for `read` in shell).
- `-h, --help`: Print help information.
- `-v, --version`: Print version information.

### Shell Example

```bash
# Split resolution into variables for a game launcher
read W H < <(wl-res -s aspect 4:3)
gamescope -w "$W" -h "$H" -- %command%
```

## Exit Codes

- `0`: Success.
- `1`: Wayland connection or display error.
- `2`: Invalid arguments or unknown command.

## Notes

- **Primary Screen:** Wayland has no formal "primary output" concept. This tool
  picks the output anchored at coordinate (0, 0), falling back to the first one
  announced by the compositor.
