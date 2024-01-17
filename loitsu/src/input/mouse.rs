use crate::{rendering::core::CameraState, util::scaling};

pub struct MouseState {
    pub position: (f32, f32),
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub drag_start: Option<(f32, f32)>,
    pub last_position: Option<(f32, f32)>,
}

impl MouseState {
    pub fn get_world_position(&self, camera: &CameraState) -> (f32, f32) {
        scaling::as_world_pos(camera, (self.position.0, self.position.1))
    }

    pub fn get_delta(&self) -> (f32, f32) {
        match self.last_position {
            Some(last_position) => (
                self.position.0 - last_position.0,
                self.position.1 - last_position.1,
            ),
            None => (0.0, 0.0),
        }
    }
}
