#[derive(Debug)]
enum WindowType {
    Horizontal(Vec<Window>),
    Vertical(Vec<Window>),
    Pane(usize),
}

#[derive(Debug)]
pub struct Window {
    width: u16,
    height: u16,
    offset_x: u16,
    offset_y: u16,
    window_type: WindowType,
}

#[derive(thiserror::Error, Debug)]
enum WindowParseError {}

pub fn parse_layout(input: &str) -> Result<Window, WindowParseError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use claim::{assert_matches, assert_ok};

    use super::*;

    #[test]
    fn parses_simple_layout() {
        let data = "6230,168x64,0,0,11";

        let window = assert_ok!(parse_layout(data));

        assert_eq!(window.width, 168);
        assert_eq!(window.height, 64);
        assert_eq!(window.offset_x, 0);
        assert_eq!(window.offset_y, 0);
        assert_matches!(window.window_type, WindowType::Pane(11));
    }

    #[test]
    fn parses_complicated_layout() {
        let data = "ecbe,168x64,0,0{84x64,0,0,11,83x64,85,0[83x32,85,0,12,83x31,85,33{41x31,85,33,13,41x31,127,33,14}]}";

        let window = assert_ok!(parse_layout(data));

        assert_eq!(window.width, 168);
        assert_eq!(window.height, 64);

        todo!("More asserts");
    }
}
