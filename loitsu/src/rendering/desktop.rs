use crate::ecs::ECS;
use crate::log_render as log;
use crate::scripting::ScriptingInstance;
use std::sync::{Arc, RwLock};
use winit::event_loop::EventLoop;

pub fn init_window<T>(scripting: T, ecs: Arc<RwLock<ECS<T>>>)
where
    T: ScriptingInstance + 'static,
{
    let event_loop = EventLoop::new().unwrap();
    log!("Opening window...");
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .with_min_inner_size(winit::dpi::LogicalSize::new(20.0, 20.0))
        .build(&event_loop)
        .unwrap();

    log!("Preparing window...");
    env_logger::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::rendering::core::run(
            event_loop, window, scripting, ecs,
        ));
}
