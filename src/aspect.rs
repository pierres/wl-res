pub fn parse_aspect(s: &str) -> Option<(i64, i64)> {
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

/// Largest `w x h` matching `aw:ah` that fits inside `screen_w x screen_h`.
pub fn fit_aspect(screen_w: i64, screen_h: i64, aw: i64, ah: i64) -> (i64, i64) {
    let tw = screen_h * aw / ah;
    if tw <= screen_w {
        (tw, screen_h)
    } else {
        (screen_w, screen_w * ah / aw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_aspect_accepts_all_separators() {
        assert_eq!(parse_aspect("4:3"), Some((4, 3)));
        assert_eq!(parse_aspect("16x9"), Some((16, 9)));
        assert_eq!(parse_aspect("21/9"), Some((21, 9)));
        assert_eq!(parse_aspect("2560x1440"), Some((2560, 1440)));
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
}
