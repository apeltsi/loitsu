use winit::event::VirtualKeyCode;

use crate::error;

pub mod mouse;

pub struct InputState {
    pub mouse: mouse::MouseState,
    pub down_keys: Vec<VirtualKeyCode>,
    pub new_keys: Vec<VirtualKeyCode>,
    pub up_keys: Vec<VirtualKeyCode>,
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
            down_keys: Vec::new(),
            new_keys: Vec::new(),
            up_keys: Vec::new(),
        }
    }

    pub fn get_key(&self, key: VirtualKeyCode) -> bool {
        self.down_keys.contains(&key)
    }

    pub fn get_key_down(&self, key: VirtualKeyCode) -> bool {
        self.new_keys.contains(&key)
    }

    pub fn get_key_up(&self, key: VirtualKeyCode) -> bool {
        self.up_keys.contains(&key)
    }
}

pub fn str_to_key(key: &str) -> Option<VirtualKeyCode> {
    match key {
        "Key0" => Some(VirtualKeyCode::Key0),
        "Key1" => Some(VirtualKeyCode::Key1),
        "Key2" => Some(VirtualKeyCode::Key2),
        "Key3" => Some(VirtualKeyCode::Key3),
        "Key4" => Some(VirtualKeyCode::Key4),
        "Key5" => Some(VirtualKeyCode::Key5),
        "Key6" => Some(VirtualKeyCode::Key6),
        "Key7" => Some(VirtualKeyCode::Key7),
        "Key8" => Some(VirtualKeyCode::Key8),
        "Key9" => Some(VirtualKeyCode::Key9),
        "A" => Some(VirtualKeyCode::A),
        "B" => Some(VirtualKeyCode::B),
        "C" => Some(VirtualKeyCode::C),
        "D" => Some(VirtualKeyCode::D),
        "E" => Some(VirtualKeyCode::E),
        "F" => Some(VirtualKeyCode::F),
        "G" => Some(VirtualKeyCode::G),
        "H" => Some(VirtualKeyCode::H),
        "I" => Some(VirtualKeyCode::I),
        "J" => Some(VirtualKeyCode::J),
        "K" => Some(VirtualKeyCode::K),
        "L" => Some(VirtualKeyCode::L),
        "M" => Some(VirtualKeyCode::M),
        "N" => Some(VirtualKeyCode::N),
        "O" => Some(VirtualKeyCode::O),
        "P" => Some(VirtualKeyCode::P),
        "Q" => Some(VirtualKeyCode::Q),
        "R" => Some(VirtualKeyCode::R),
        "S" => Some(VirtualKeyCode::S),
        "T" => Some(VirtualKeyCode::T),
        "U" => Some(VirtualKeyCode::U),
        "V" => Some(VirtualKeyCode::V),
        "W" => Some(VirtualKeyCode::W),
        "X" => Some(VirtualKeyCode::X),
        "Y" => Some(VirtualKeyCode::Y),
        "Z" => Some(VirtualKeyCode::Z),
        "Space" => Some(VirtualKeyCode::Space),
        "Escape" => Some(VirtualKeyCode::Escape),
        "Back" => Some(VirtualKeyCode::Back),
        "Return" => Some(VirtualKeyCode::Return),
        "Tab" => Some(VirtualKeyCode::Tab),
        "Left" => Some(VirtualKeyCode::Left),
        "Up" => Some(VirtualKeyCode::Up),
        "Right" => Some(VirtualKeyCode::Right),
        "Down" => Some(VirtualKeyCode::Down),
        _ => {
            error!("Unknown key: {}", key);
            None
        }
    }
}
