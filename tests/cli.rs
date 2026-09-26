use std::process::Command;

#[test]
fn invalid_arguments_fail_before_connecting_to_wayland() {
    for (args, expected_error) in [
        (vec!["unknown"], "Unknown command: unknown"),
        (vec!["aspect"], "requires a ratio"),
        (vec!["aspect", "bad"], "Invalid aspect ratio bad"),
        (vec!["width", "extra"], "Unexpected argument: extra"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_wl-res"))
            .args(args)
            .env("WAYLAND_DISPLAY", "wl-res-nonexistent-socket")
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
