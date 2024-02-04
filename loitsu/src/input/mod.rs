use winit::keyboard::{Key, NamedKey};

pub mod mouse;

pub struct InputState {
    pub mouse: mouse::MouseState,
    pub down_keys: Vec<Key>,
    pub new_keys: Vec<Key>,
    pub up_keys: Vec<Key>,
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

    pub fn get_key(&self, key: Key) -> bool {
        self.down_keys.contains(&key)
    }

    pub fn get_key_down(&self, key: Key) -> bool {
        self.new_keys.contains(&key)
    }

    pub fn get_key_up(&self, key: Key) -> bool {
        self.up_keys.contains(&key)
    }
}

pub fn str_to_key(key: &str) -> Option<Key> {
    match key {
        "Space" => Some(Key::Named(NamedKey::Space)),
        "Escape" => Some(Key::Named(NamedKey::Escape)),
        "Backspace" => Some(Key::Named(NamedKey::Backspace)),
        "Enter" => Some(Key::Named(NamedKey::Enter)),
        "Tab" => Some(Key::Named(NamedKey::Tab)),
        "Left" => Some(Key::Named(NamedKey::ArrowLeft)),
        "Up" => Some(Key::Named(NamedKey::ArrowUp)),
        "Right" => Some(Key::Named(NamedKey::ArrowRight)),
        "Down" => Some(Key::Named(NamedKey::ArrowDown)),
        _ => {
            let smol_str = key.chars().next().unwrap().to_string();
            Some(Key::Character(smol_str.to_ascii_lowercase().into()))
        }
    }
}
