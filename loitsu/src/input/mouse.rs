use crate::rendering::core::CameraState;

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
        let (x, y) = (self.position.0 - 0.5, -self.position.1 + 0.5); // Center the mouse position
        let (x, y) = (x * 2.0, y * 2.0); // Scale the mouse position to the range -1.0 to 1.0
        let (x, y) = (x / camera.aspect.1, y / camera.aspect.0);
        let (x, y) = (x / (camera.scale * 2.0), y / (camera.scale * 2.0)); // Scale the mouse position by the camera zoom
        let (x, y) = (x + camera.position.x, y + camera.position.y); // Move the mouse position by the camera position
        (x, y)
    }

    pub fn as_world_scale(&self, camera: &CameraState, pos: (f32, f32)) -> (f32, f32) {
        let (x, y) = (pos.0 * 2.0, pos.1 * 2.0); // Scale the mouse position to the range -1.0 to 1.0
        let (x, y) = (x / camera.scale, y / camera.scale); // Scale the mouse position by the camera zoom
        (x, y)
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
