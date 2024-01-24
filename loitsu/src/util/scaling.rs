use crate::rendering::core::CameraState;

/// Expects a vector, representing a scale, in screen space and returns a vector in world space
/// This is the inverse of `as_screen_scale`
pub fn as_world_scale(camera: &CameraState, pos: (f32, f32)) -> (f32, f32) {
    let (x, y) = (pos.0, pos.1);
    let (x, y) = (x / camera.scale, y / camera.scale); // Scale the mouse position by the camera zoom
    let (x, y) = (x / camera.aspect.1, y / camera.aspect.0);
    (x * 2.0, y * 2.0)
}

/// Expects a vector, representing a scale, in world space and returns a vector in screen space
/// This is the inverse of `as_world_scale`
pub fn as_screen_scale(camera: &CameraState, pos: (f32, f32)) -> (f32, f32) {
    let (x, y) = (pos.0 * camera.aspect.1, pos.1 * camera.aspect.0);
    let (x, y) = (x * camera.scale, y * camera.scale);
    (x / 2.0, y / 2.0)
}

/// Expects a vector, representing a position, in screen space and returns a vector in world space
/// Here, (0, 0) represents the bottom-left corner of the screen.
/// This is the inverse of `as_screen_pos`
pub fn as_world_pos(camera: &CameraState, pos: (f32, f32)) -> (f32, f32) {
    let aspect = camera.aspect;
    let (x, y) = (pos.0 * 2.0, pos.1 * 2.0); // Scale the mouse position to the range -1.0 to 1.0
    let (x, y) = (x - 1.0, -y + 1.0); // Center the mouse position
    let (x, y) = (x / camera.scale, y / camera.scale); // Scale the mouse position by the camera zoom
    let (x, y) = (x + camera.position.x, y + camera.position.y); // Move the mouse position by the camera position
    (x / aspect.1, y / aspect.0)
}

/// Expects a vector, representing a position, in world space and returns a vector in screen space
/// Here, (0, 0) represents the bottom-left corner of the screen.
/// This is the inverse of `as_world_pos`
pub fn as_screen_pos(camera: &CameraState, pos: (f32, f32)) -> (f32, f32) {
    let aspect = camera.aspect;
    let (x, y) = (
        pos.0 * aspect.1 - camera.position.x,
        pos.1 * aspect.0 - camera.position.y,
    ); // Move the mouse position by the camera position
    let (x, y) = (x * camera.scale, y * camera.scale); // Scale the mouse position by the camera zoom
    let (x, y) = (x, y); // Scale the mouse position to the range -1.0 to 1.0
    let (x, y) = (x + 1.0, -y + 1.0); // Center the mouse position
    (x / 2.0, y / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_world_scale() {
        let camera = CameraState {
            position: (0.0, 0.0).into(),
            scale: 2.0,
            aspect: (1.0, 2.0),
            dirty: false,
            view: [[0.0; 4]; 4],
        };
        // float eq is bad, but this works sooooo... ¯\_(ツ)_/¯
        assert_eq!(
            as_screen_scale(&camera, as_world_scale(&camera, (0.0, 0.0))),
            (0.0, 0.0)
        );
        assert_eq!(
            as_screen_scale(&camera, as_world_scale(&camera, (0.5, 0.5))),
            (0.5, 0.5)
        );
        assert_eq!(
            as_screen_scale(&camera, as_world_scale(&camera, (1.0, 1.0))),
            (1.0, 1.0)
        );
    }

    #[test]
    fn test_as_world_pos() {
        let camera = CameraState {
            position: (0.0, 0.0).into(),
            scale: 3.0,
            aspect: (1.0, 2.0),
            dirty: false,
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        assert_eq!(
            as_screen_pos(&camera, as_world_pos(&camera, (0.0, 0.0))),
            (0.0, 0.0)
        );
        assert_eq!(
            as_screen_pos(&camera, as_world_pos(&camera, (0.5, 0.5))),
            (0.5, 0.5)
        );
        assert_eq!(
            as_screen_pos(&camera, as_world_pos(&camera, (1.0, 1.0))),
            (1.0, 1.0)
        );
    }
}
