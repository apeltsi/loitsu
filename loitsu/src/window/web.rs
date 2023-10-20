use khronos_egl;

pub fn init_view() {
    env_logger::init();

    println!("Initializing view...");
    let egl = khronos_egl::Instance::new(khronos_egl::Static);
    let display = unsafe { egl.get_display(khronos_egl::DEFAULT_DISPLAY) }.unwrap();
    egl.initialize(display)
        .expect("unable to initialize display");
}
