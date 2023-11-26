use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;
use crate::log;
use crate::web::update_loading_status;
use crate::scripting::ScriptingInstance;
use crate::ecs::ECS;

pub fn init_view<T>(scripting: T, ecs: ECS<T>) where T: ScriptingInstance + 'static {
    log!("Initializing web...");
    update_loading_status(1);

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .with_inner_size(winit::dpi::LogicalSize::new(20.0, 20.0))
        .build(&event_loop)
        .unwrap();
    // On wasm, append the canvas to the document body
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
    .expect("couldn't append canvas to document body");
    wasm_bindgen_futures::spawn_local(crate::rendering::core::run(event_loop, window, scripting, ecs));
}
