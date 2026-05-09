/// Parses a string like "16:9" into width/height parts.
///
/// For newcomers:
/// - Options in Rust: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
/// - Iterators: https://doc.rust-lang.org/book/ch13-02-iterators.html
pub fn parse_aspect(s: &str) -> Option<(i64, i64)> {
    if let Some((a, b)) = s.split_once(':') {
        // 'parse' converts a string to a number. It returns a Result.
        // We use 'let (Ok(aw), Ok(ah))' to only proceed if both parts are valid integers.
        let (Ok(aw), Ok(ah)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) else {
            return None;
        };
        // Only accept positive dimensions.
        if aw > 0 && ah > 0 {
            return Some((aw, ah));
        }
    }
    None
}

/// Calculates the largest `w x h` box with ratio `aw:ah` that fits inside `screen_w x screen_h`.
///
/// For newcomers:
/// - Casting: We use 'as i128' because multiplying two large i64 numbers can overflow.
///   i128 is a 128-bit integer, which is much larger than the 64-bit i64.
///   Rust Book on Data Types: https://doc.rust-lang.org/book/ch03-02-data-types.html
pub fn fit_aspect(screen_w: i64, screen_h: i64, aw: i64, ah: i64) -> (i64, i64) {
    // 1. Try fitting by screen height: target_width = screen_height * (aw / ah)
    let tw = screen_h as i128 * aw as i128 / ah as i128;

    if tw <= screen_w as i128 {
        // Fits! (target_width, screen_height)
        (tw as i64, screen_h)
    } else {
        // Too wide! Fit by screen width instead: target_height = screen_width * (ah / aw)
        let th = screen_w as i128 * ah as i128 / aw as i128;
        (screen_w, th as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_aspect_accepts_separator() {
        assert_eq!(parse_aspect("4:3"), Some((4, 3)));
        assert_eq!(parse_aspect(" 16:9 "), Some((16, 9)));
        assert_eq!(parse_aspect("21 : 9"), Some((21, 9)));
    }

    #[test]
    fn parse_aspect_rejects_malformed() {
        assert_eq!(parse_aspect(""), None);
        assert_eq!(parse_aspect("4"), None);
        assert_eq!(parse_aspect("4:"), None);
        assert_eq!(parse_aspect(":3"), None);
        assert_eq!(parse_aspect("abc:def"), None);
        assert_eq!(parse_aspect("4:3:2"), None);
    }

    #[test]
    fn parse_aspect_rejects_non_positive() {
        assert_eq!(parse_aspect("0:3"), None);
        assert_eq!(parse_aspect("4:0"), None);
        assert_eq!(parse_aspect("-4:3"), None);
        assert_eq!(parse_aspect("4:-3"), None);
    }

    #[test]
    fn fit_aspect_height_bound() {
        // narrower-than-screen target: limited by screen height
        assert_eq!(fit_aspect(2560, 1440, 4, 3), (1920, 1440));
        assert_eq!(fit_aspect(1920, 1080, 4, 3), (1440, 1080));
    }

    #[test]
    fn fit_aspect_width_bound() {
        // wider-than-screen target: limited by screen width
        assert_eq!(fit_aspect(2560, 1440, 21, 9), (2560, 1097));
        assert_eq!(fit_aspect(1920, 1080, 32, 9), (1920, 540));
    }

    #[test]
    fn fit_aspect_exact_match_returns_full_screen() {
        assert_eq!(fit_aspect(2560, 1440, 16, 9), (2560, 1440));
        assert_eq!(fit_aspect(1920, 1080, 16, 9), (1920, 1080));
    }

    #[test]
    fn fit_aspect_square_uses_smaller_dimension() {
        assert_eq!(fit_aspect(1920, 1080, 1, 1), (1080, 1080));
        assert_eq!(fit_aspect(1080, 1920, 1, 1), (1080, 1080));
    }

    #[test]
    fn fit_aspect_does_not_overflow() {
        // screen_h * aw = 1440 * 9000000000000000000 > i64::MAX
        assert_eq!(
            fit_aspect(2560, 1440, 9_000_000_000_000_000_000, 1),
            (2560, 0)
        );
    }
}
