use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use crate::{log, scripting::ScriptingInstance, scene_management::Scene};
use crate::ecs::ECS;

pub static mut TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

static mut DEFAULT_SAMPLER: Option<wgpu::Sampler> = None;

#[cfg(target_arch = "wasm32")]
use crate::web::update_loading_status;

use super::{drawable::Drawable, shader::ShaderManager};

pub async fn run<T>(event_loop: EventLoop<()>, window: Window, mut scripting: T, mut ecs: ECS<T>) where T: ScriptingInstance + 'static {
    unsafe { HAS_RENDERED = false; }
    unsafe { HAS_LOADED = false; }
    #[cfg(target_arch = "wasm32")]
    update_loading_status(2);
    let size = window.inner_size();

    let instance = wgpu::Instance::default();
    let surface = unsafe {instance.create_surface(&window).unwrap()};
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.expect("No suitable adapter available");

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    }, None).await.expect("Unable to create device");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    unsafe { TARGET_FORMAT = swapchain_format; }

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    
    // lets load our default shaders
    let mut shader_manager: ShaderManager = crate::rendering::shader::ShaderManager::new();
    shader_manager.load_default_shaders(&device);
    // lets init the global bind group
    let camera_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Camera Matrix Buffer"),
        size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let global_bind_group_layout = get_global_bind_group_layout(&device);
    let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &global_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &camera_matrix_buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<[[f32; 4]; 4]>() as u64),
            }),
        }],
        label: None,
    });

    // lets create the default sampler

    unsafe { 
        DEFAULT_SAMPLER = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            anisotropy_clamp: 0,
            mipmap_filter: wgpu::FilterMode::Nearest,
            border_color: None,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
        }));
    }
    log!("Running event loop...");
    let mut ecs_initialized = false;
    let mut drawables = Vec::<Box<dyn Drawable>>::new();
    let mut frame_count = 0;
    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter);
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if !ecs_initialized {
                    let scene: Option<Scene> = {
                        let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
                        let x = if let Some(static_shard) = &asset_manager.assets.lock().unwrap().static_shard {
                            // init scripts
                            scripting.initialize(static_shard.get_scripts().clone()).unwrap();
                            log!("Scripting initialized");
                            let default_scene_name = static_shard.get_preferences().default_scene.as_str();
                            let scene = static_shard.get_scene(default_scene_name);
                            Some(scene.expect(
                            format!("Default scene wasn't included in the static shard! Expected to find scene '{}'. Available scenes are '{}'", 
                                    default_scene_name,
                                    static_shard.get_available_scene_names().join("', '")
                            ).as_str()).clone())
                        } else {
                            None
                        }; x
                    };
                    if let Some(scene) = scene {
                        ecs.load_scene(
                            scene, &mut scripting);
                        ecs_initialized = true;
                        log!("ECS initialized");
                    }
                } else {
                    let mut asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
                    asset_manager.initialize_shards(&device, &queue);
                    ecs.run_frame(&mut scripting);
                }
                if frame_count == 100 {
                    let mut debug = Box::new(crate::rendering::drawable::sprite::SpriteDrawable::new("sprites/test.png"));
                    debug.init(&device, &shader_manager);
                    //drawables.push(debug); uncommenting this will lead to the compiler complaining :sob:
                }
                render_frame(&surface, &device, &queue, &drawables, &shader_manager, &global_bind_group,ecs_initialized);
                frame_count += 1;
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }
    });
}

static mut HAS_RENDERED: bool = false;
static mut HAS_LOADED: bool = false;

pub fn render_frame(surface: &wgpu::Surface, device: &wgpu::Device, 
                    queue: &wgpu::Queue, drawables: &Vec<Box<dyn Drawable>>, 
                    shader_manager: &ShaderManager, global_bind_group: &wgpu::BindGroup,
                    ecs_initialized: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if !unsafe { HAS_RENDERED } { 
            update_loading_status(3);
            unsafe { HAS_RENDERED = true; }
        }
    }
    let frame = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let mut clear_color = wgpu::Color::RED;
    {
        let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
        if asset_manager.pending_tasks.load(std::sync::atomic::Ordering::SeqCst) == 0 && ecs_initialized {
            clear_color = wgpu::Color::BLUE;
            #[cfg(target_arch = "wasm32")]
            {
                if !unsafe { HAS_LOADED } { 
                    update_loading_status(4);
                    unsafe { HAS_LOADED = true; }
                }
            }
        }
    }
    {
        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Primary Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Lets clear the main texture
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store 
                }
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None
        });

        for drawable in drawables {
            drawable.draw(&mut r_pass, &shader_manager, global_bind_group);
        }
    }

    queue.submit(Some(encoder.finish()));
    frame.present();
}

pub fn get_global_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            },
        ],
    })
}

pub fn get_sprite_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sprite Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float {filterable: true}, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }
        ],
    })
}

pub fn get_default_sampler(device: &wgpu::Device) -> Option<&wgpu::Sampler> {
    unsafe { DEFAULT_SAMPLER.as_ref() }
}
