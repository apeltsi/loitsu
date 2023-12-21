pub mod mouse;

pub struct InputState {
    pub mouse: mouse::MouseState,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mouse: mouse::MouseState {
                position: (0.0, 0.0),
                left_button: false,
                right_button: false,
                middle_button: false,
                drag_start: None,
                last_position: None,
            },
        }
    }
}
