use khronos_egl;
use wgpu_hal::{Adapter, Device};
use wgpu::{Limits, ImageSubresourceRange, TextureViewDimension, TextureFormat, Features};

static mut LOADING_STATUS: i32 = 0;

#[no_mangle]
pub extern "C" fn check_status() -> i32 {
    unsafe { LOADING_STATUS }
}

pub fn init_view() {
    env_logger::init();

    unsafe { LOADING_STATUS = 1 };
    println!("Initializing view...");
    let egl = khronos_egl::Instance::new(khronos_egl::Static);
    let display = unsafe { egl.get_display(khronos_egl::DEFAULT_DISPLAY) }.unwrap();
    egl.initialize(display)
        .expect("unable to initialize display");

    let attributes = [
        khronos_egl::RED_SIZE,
        8,
        khronos_egl::GREEN_SIZE,
        8,
        khronos_egl::BLUE_SIZE,
        8,
        khronos_egl::NONE,
    ];

    let config = egl
        .choose_first_config(display, &attributes)
        .unwrap()
        .expect("unable to choose config");

    let surface = unsafe {
        let window = std::ptr::null_mut::<std::ffi::c_void>();
        egl.create_window_surface(display, config, window, None)
    }.expect("unable to create surface");

    let context_attributes = [khronos_egl::CONTEXT_CLIENT_VERSION, 3, khronos_egl::NONE];

    let gl_context = egl
        .create_context(display, config, None, &context_attributes)
        .expect("unable to create context");
    egl.make_current(display, Some(surface), Some(surface), Some(gl_context))
        .expect("can't make context current");

    println!("Hooking up to wgpu-hal");
    let exposed = unsafe {
        <wgpu_hal::api::Gles as wgpu_hal::Api>::Adapter::new_external(|name| {
            egl.get_proc_address(name)
                .map_or(std::ptr::null(), |p| p as *const _)
        })
    }
    .expect("GL adapter can't be initialized");

    unsafe { LOADING_STATUS = 2 };

    let mut od = unsafe {
        exposed.adapter.open(
            Features::empty(),
            &Limits::default()
        ).unwrap()
    };

    let format = TextureFormat::Rgba8UnormSrgb;
    let texture = <wgpu_hal::api::Gles as wgpu_hal::Api>::Texture::default_framebuffer(format);
    let view = unsafe {
        od.device
            .create_texture_view(
                &texture,
                &wgpu_hal::TextureViewDescriptor {
                    label: None,
                    format,
                    dimension: TextureViewDimension::D2,
                    usage: wgpu_hal::TextureUses::COLOR_TARGET,
                    range: ImageSubresourceRange::default()
                },
            ).unwrap();

    };
    println!("View initialized!");
    unsafe { LOADING_STATUS = 3 };
}
