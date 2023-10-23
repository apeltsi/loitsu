use winit::event_loop::EventLoop;
use crate::log;
pub fn init_window() {
    let event_loop = EventLoop::new();
    log!("Opening window...");
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .build(&event_loop)
        .unwrap();

    log!("Preparing window...");
    env_logger::init();
    pollster::block_on(crate::window::core::run(event_loop, window));
}

