use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit::platform::web::WindowExtWebSys;
use wasm_bindgen::closure::Closure;
use crate::log;
use crate::web::update_loading_status;

pub fn init_view() {
    log!("Initializing web...");
    update_loading_status(1);

    let window_size = || {
        let win = web_sys::window().unwrap();
        winit::dpi::LogicalSize::new(win.inner_width().unwrap().as_f64().unwrap(), win.inner_height().unwrap().as_f64().unwrap())
    };

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("loitsu")
        .with_inner_size(window_size())
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
    wasm_bindgen_futures::spawn_local(crate::rendering::core::run(event_loop, window));
}
