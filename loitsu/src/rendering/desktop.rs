use winit::event_loop::EventLoop;
use crate::log;
use crate::scripting::ScriptingInstance;
use crate::ecs::ECS;

pub fn init_window<T>(scripting: T, ecs: ECS<T>) where T: ScriptingInstance + 'static {
    let event_loop = EventLoop::new();
    log!("Opening window...");
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .build(&event_loop)
        .unwrap();

    log!("Preparing window...");
    env_logger::init();
    pollster::block_on(crate::rendering::core::run(event_loop, window, scripting, ecs));
}

