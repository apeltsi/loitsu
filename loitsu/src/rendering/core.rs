#[allow(unused_imports)]
use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState, MouseScrollDelta},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
#[allow(unused_imports)]
use crate::{log_render as log, scripting::{ScriptingInstance, EntityUpdate}, scene_management::Scene, rendering::drawable::{sprite::SpriteDrawable, DrawablePrototype}, asset_management::AssetManager, ecs::{Transform, RuntimeEntity}, log_scripting, input::InputState};
#[allow(unused_imports)]
use crate::ecs::{ECS, ComponentFlags};
use std::{cmp::max, cell::RefCell, rc::Rc, sync::{Mutex, Arc}};
use crate::{asset_management::ASSET_MANAGER, scene_management::Entity, util::scaling};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

pub static mut TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

static mut DEFAULT_SAMPLER: Option<wgpu::Sampler> = None;

#[cfg(target_arch = "wasm32")]
static mut WEB_RESIZED: bool = true;

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
        size: (std::mem::size_of::<[[f32; 4]; 4]>() as u64) * 2,
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
                size: wgpu::BufferSize::new((std::mem::size_of::<[[f32; 4]; 4]>() as u64) * 2),
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
            anisotropy_clamp: 1,
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
    let mut frame_count: u64 = 0;
    let mut state = State {
        camera: CameraState::new()
    };
    let input_state = Arc::new(Mutex::new(InputState::new()));
    state.camera.set_scale(1.0);
    state.camera.set_position([0.0, 0.0].into());
    #[cfg(feature = "editor")]
    let mut selected_entity: Option<Rc<RefCell<RuntimeEntity<T>>>> = None;
    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter);
        *control_flow = ControlFlow::Poll;
        #[cfg(target_arch = "wasm32")] {
            if unsafe {WEB_RESIZED} {
                let window_size = {
                    let win = web_sys::window().unwrap();
                    winit::dpi::LogicalSize::new(win.inner_width().unwrap().as_f64().unwrap(), win.inner_height().unwrap().as_f64().unwrap())
                };
                window.set_inner_size(window_size);
                if frame_count == 0 {
                    let max = max(window_size.width as i32, window_size.height as i32);
                    state.camera.view = [
                            [(size.height as f32) / max as f32, 0.0, 0.0, 0.0], 
                            [0.0, (size.width as f32) / max as f32, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0]
                    ];
                    queue.write_buffer(&camera_matrix_buffer, 0, bytemuck::cast_slice(&[CameraMatrix {
                        view: state.camera.view,
                        camera: state.camera.get_transformation_matrix()
                    }]));
                }
                unsafe {WEB_RESIZED = false;}
            }
        }

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

                // okay gamers lets resize the screen & camera matrix buffers
                // from atlas :D
                let max = max(size.width, size.height);
                state.camera.aspect = (size.width as f32 / max as f32, size.height as f32 / max as f32);
                state.camera.view = [
                        [(size.height as f32) / max as f32, 0.0, 0.0, 0.0], 
                        [0.0, (size.width as f32) / max as f32, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0]
                ];
                queue.write_buffer(&camera_matrix_buffer, 0, bytemuck::cast_slice(&[CameraMatrix {
                    view: state.camera.view,
                    camera: state.camera.get_transformation_matrix()
                }]));
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                #[allow(unused_mut)]
                let mut updates = Vec::new();
                #[cfg(feature = "editor")]
                {
                    let client_events = ecs.poll_client_events();
                    for event in client_events {
                        match event {
                            crate::editor::ClientEvent::SelectEntity(id) => {
                                if let Some(entity) = ecs.get_entity(id.as_str()) {
                                    selected_entity = Some(entity.clone());
                                    let entity = (*entity).borrow();
                                    let as_entity = entity.as_entity();
                                    let entity_bounds = get_entity_screen_space_bounds(&state.camera, &as_entity).unwrap();
                                    ecs.emit(crate::editor::Event::EntitySelected(as_entity));
                                    ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                                }
                            },
                            crate::editor::ClientEvent::SetComponentProperty { entity, component, field, property } => {
                                if let Some(entity) = ecs.get_entity(entity.as_str()) {
                                    {
                                        let mut entity = (*entity).borrow_mut();
                                        let component = entity.get_component_mut(component.as_str()).unwrap();
                                        component.set_property(field.as_str(), property);
                                    }
                                    updates.extend(ecs.run_component_methods(&mut scripting, ComponentFlags::EDITOR_UPDATE));
                                }
                            },
                        }
                    }
                }

                #[cfg(not(feature = "direct_asset_management"))]
                if !ecs_initialized {
                    let scene: Option<Scene> = {
                        let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
                        let x = if let Some(static_shard) = &asset_manager.assets.lock().unwrap().static_shard {
                            // init scripts
                            scripting.initialize(static_shard.get_scripts().clone(), input_state.clone()).unwrap();
                            log_scripting!("Scripting initialized");
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
                    if frame_count > 1 && ecs_initialized && asset_manager.pending_tasks.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                        #[cfg(not(feature = "disable_common_ecs_methods"))]
                        {
                            updates.extend(ecs.run_frame(&mut scripting));
                        }
                    }
                }
                #[cfg(feature = "editor")]
                {
                    if !ecs_initialized {
                        updates.extend(ecs.run_component_methods(&mut scripting, crate::ecs::ComponentFlags::EDITOR_START));
                        #[cfg(target_arch = "wasm32")]
                        crate::web::remove_editor_loading_task("Starting render pipeline...");
                        ecs_initialized = true;
                    }
                }
                {
                    let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
                    process_entity_updates(&device, &asset_manager, &shader_manager, &mut drawables, updates);
                }
                {
                    let mut input_state = input_state.lock().unwrap();
                    input_state.new_keys.clear();
                    input_state.up_keys.clear();
                }
                if state.camera.dirty {
                    queue.write_buffer(&camera_matrix_buffer, 0, bytemuck::cast_slice(&[CameraMatrix {
                        view: state.camera.view,
                        camera: state.camera.get_transformation_matrix()
                    }]));
                    state.camera.dirty = false;
                    #[cfg(feature = "editor")]
                    {
                        ecs.emit(crate::editor::Event::CameraChanged(state.camera.position.x, state.camera.position.y, state.camera.scale));
                    }
                }
                render_frame(&surface, &device, &queue, &mut drawables, &global_bind_group, ecs_initialized, frame_count);
                frame_count += 1;
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, ..} => {
                    let mut input_state = input_state.lock().unwrap();
                    input_state.mouse.last_position = Some(input_state.mouse.position);
                    input_state.mouse.position = (position.x as f32 / config.width as f32, position.y as f32 / config.height as f32);
                    #[cfg(feature = "editor")]
                    if input_state.mouse.right_button {
                        let delta = input_state.mouse.get_delta();
                        let world_scale_delta = crate::util::scaling::as_world_scale(&state.camera, (-delta.0, delta.1));
                        state.camera.position.x += world_scale_delta.0;
                        state.camera.position.y += world_scale_delta.1;
                        state.camera.dirty = true;
                        if let Some(entity) = &selected_entity {
                            let as_entity = entity.borrow().as_entity();
                            let entity_bounds = get_entity_screen_space_bounds(&state.camera, &as_entity).unwrap();
                            ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                        }
                    }
                },
                WindowEvent::MouseInput { state: element_state, button, .. } => {
                    let mut input_state = input_state.lock().unwrap();
                    match button {
                        MouseButton::Left => {
                            input_state.mouse.left_button = *element_state == ElementState::Pressed;
                            #[cfg(feature = "editor")]
                            if *element_state == ElementState::Pressed {
                                let click_pos = input_state.mouse.get_world_position(&state.camera);
                                if let Some(entity) = find_overlapping_entity(&ecs, click_pos) {
                                    selected_entity = Some(entity.clone());
                                    let entity = entity.borrow();
                                    let as_entity = entity.as_entity();
                                    let entity_bounds = get_entity_screen_space_bounds(&state.camera, &as_entity).unwrap();
                                    ecs.emit(crate::editor::Event::EntitySelected(as_entity));
                                    ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                                }
                            }
                        },
                        MouseButton::Right => {
                            input_state.mouse.right_button = *element_state == ElementState::Pressed;
                        },
                        MouseButton::Middle => {
                            input_state.mouse.middle_button = *element_state == ElementState::Pressed;
                        },
                        _ => {}
                    }
                },
                #[cfg(feature = "editor")]
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            state.camera.scale += y * 0.001;
                            state.camera.dirty = true;
                        },
                        MouseScrollDelta::PixelDelta(pos) => {
                            state.camera.scale += pos.y as f32 * 0.001;
                            state.camera.dirty = true;
                        }
                    }
                    if state.camera.scale < 0.1 {
                        state.camera.scale = 0.1;
                    }
                    if let Some(entity) = &selected_entity {
                        let as_entity = entity.borrow().as_entity();
                        let entity_bounds = get_entity_screen_space_bounds(&state.camera, &as_entity).unwrap();
                        ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                    }
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        let mut input_state = input_state.lock().unwrap();
                        // check if we should add or remove the key
                        if input.state == ElementState::Released {
                            input_state.down_keys.retain(|&x| x != key);
                            input_state.up_keys.push(key);
                        } else {
                            input_state.down_keys.push(key);
                            input_state.new_keys.push(key);
                        }
                    }
                },
                _ => {}
            },
            _ => {}
        }
    });
}

#[allow(dead_code)]
fn get_entity_screen_space_bounds(camera: &CameraState, entity: &Entity) -> Option<(f32, f32, f32, f32)> {
    match entity.transform {
        Transform::Transform2D {position, scale, ..} => {
            let screen_pos = scaling::as_screen_pos(camera, position);
            let screen_scale = scaling::as_screen_scale(camera, scale);
            return Some((screen_pos.0, screen_pos.1, screen_scale.0, screen_scale.1));
        },
        _ => {}
    };
    None
}

#[allow(dead_code)]
fn find_overlapping_entity<T>(ecs: &ECS<T>, check_position: (f32, f32)) -> Option<Rc<RefCell<RuntimeEntity<T>>>> where T: ScriptingInstance {
    for e in ecs.get_runtime_entities() {
        let entity = e.borrow();
        match *(*entity.transform).borrow() {
            Transform::Transform2D {position, scale, ..} => {
                if position.0 - scale.0 / 2.0 <= check_position.0 && position.0 + scale.0 / 2.0 >= check_position.0 &&
                   position.1 - scale.0 / 2.0 <= check_position.1 && position.1 + scale.1 / 2.0 >= check_position.1 {
                    return Some(e.clone());
                }
            },
            _ => {}
        };
    }
    None
}

fn process_entity_updates(device: &wgpu::Device, 
                          asset_manager: &AssetManager, 
                          shader_manager: &ShaderManager,
                          drawables: &mut Vec<Box<dyn Drawable>>,
                          updates: Vec<(Rc<RefCell<Transform>>, Vec<EntityUpdate>)>) {
    for entity_updates in updates {
        for update in entity_updates.1 {
            match update {
                EntityUpdate::AddDrawable(drawable) => {
                    match drawable {
                        DrawablePrototype::Sprite {sprite, color, id} => {
                            let mut drawable = Box::new(SpriteDrawable::new(sprite.as_str(), color, id, shader_manager));
                            drawable.init(device, asset_manager, entity_updates.0.clone());
                            drawables.push(drawable);
                        }
                    }
                },
                EntityUpdate::RemoveDrawable(id) => {
                    // NOTE: This could be more efficient, maybe use a hashmap?
                    for i in 0..drawables.len() {
                        if drawables[i].get_uuid().to_string() == id {
                            drawables.remove(i);
                            break;
                        }
                    }
                },
                EntityUpdate::SetDrawableProperty(id, field_name, property) => {
                    // NOTE: Same as above, maybe use a hashmap?
                    for drawable in &mut *drawables {
                        if drawable.get_uuid().to_string() == id {
                            drawable.set_property(field_name, property);
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraMatrix {
    view: [[f32; 4]; 4],
    camera: [[f32; 4]; 4],
}

struct State {
    camera: CameraState
}

pub struct CameraState {
    pub position: cgmath::Vector2<f32>,
    pub scale: f32,
    pub aspect: (f32, f32),
    pub dirty: bool,
    pub view: [[f32; 4]; 4]
}

impl CameraState {
    fn new() -> CameraState {
        CameraState {
            position: cgmath::Vector2::<f32> {x: 0.0, y: 0.0},
            scale: 1.0,
            aspect: (1.0, 1.0),
            dirty: false,
            view: [[0.0,0.0,0.0,0.0], [0.0,0.0,0.0,0.0], [0.0,0.0,0.0,0.0], [0.0,0.0,0.0,0.0]]
        }
    }

    fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.position = position;
        self.dirty = true;
    }

    fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.dirty = true;
    }

    fn get_transformation_matrix(&self) -> [[f32; 4]; 4] {
        [
            [self.scale, 0.0, 0.0, -self.position.x * self.scale], // position is inverted because we want to move the world, not the camera
            [0.0, self.scale, 0.0, -self.position.y * self.scale],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

static mut HAS_RENDERED: bool = false;
static mut HAS_LOADED: bool = false;

pub fn render_frame(surface: &wgpu::Surface, device: &wgpu::Device, 
                    queue: &wgpu::Queue, drawables: &mut Vec<Box<dyn Drawable>>, 
                    global_bind_group: &wgpu::BindGroup,
                    ecs_initialized: bool, frame_num: u64) {
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
    let clear_color = wgpu::Color::BLACK;
    {
        let asset_manager = ASSET_MANAGER.lock().unwrap();
        if asset_manager.pending_tasks.load(std::sync::atomic::Ordering::SeqCst) == 0 && ecs_initialized {
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
            drawable.draw(frame_num, &device, &queue, &mut r_pass, global_bind_group);
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
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float {filterable: true}, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }
        ],
    })
}

pub fn get_default_sampler() -> Option<&'static wgpu::Sampler> {
    unsafe { DEFAULT_SAMPLER.as_ref() }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn resize() {
    unsafe { WEB_RESIZED = true; }
}
