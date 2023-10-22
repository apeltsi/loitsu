use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub fn init_window() {
    let event_loop = EventLoop::new();
    println!("Opening window...");
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .build(&event_loop)
        .unwrap();

    println!("Preparing window...");
    env_logger::init();
    pollster::block_on(run(event_loop, window));
}

