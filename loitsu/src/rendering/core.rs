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
use std::{cmp::max, sync::{Mutex, Arc, RwLock}, rc::Rc, cell::RefCell};
use crate::{asset_management::ASSET_MANAGER, util::scaling, ecs::RuntimeTransform};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

pub static mut TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

static mut DEFAULT_SAMPLER: Option<wgpu::Sampler> = None;

#[cfg(target_arch = "wasm32")]
static mut WEB_RESIZED: bool = true;

#[cfg(target_arch = "wasm32")]
use crate::web::update_loading_status;

use super::{drawable::Drawable, shader::ShaderManager};

pub async fn run<T>(event_loop: EventLoop<()>, window: Window, mut scripting: T, ecs: Arc<RwLock<ECS<T>>>) where T: ScriptingInstance + 'static {
    unsafe { HAS_RENDERED = false; }
    unsafe { HAS_LOADED = false; }
    #[cfg(target_arch = "wasm32")]
    update_loading_status(2);
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let window = Rc::new(RefCell::new(window));
    let window_clone = window.clone();
    let win = &*window_clone.borrow();
    let surface = instance.create_surface(win).unwrap();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.expect("No suitable adapter available");

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    }, None).await.expect("Unable to create device");
    let adapter_info = adapter.get_info();
    log!("Backend: {:?} | Adapter: {:?} | Driver: {:?}", adapter_info.backend, adapter_info.name, adapter_info.driver_info);

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    #[cfg(target_arch = "wasm32")]
    unsafe { TARGET_FORMAT = swapchain_format; }
    #[cfg(not(target_arch = "wasm32"))]
    unsafe { TARGET_FORMAT = swapchain_format.add_srgb_suffix(); }

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        #[cfg(target_arch = "wasm32")]
        format: swapchain_format,
        #[cfg(not(target_arch = "wasm32"))]
        format: swapchain_format.remove_srgb_suffix(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![unsafe { TARGET_FORMAT }],
        desired_maximum_frame_latency: 2
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
    let mut selected_entity: Option<Arc<Mutex<RuntimeEntity<T>>>> = None;
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused)]
    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, window_target| {
        let _ = (&instance, &adapter);
        if frame_count == 0 {
            window.borrow().request_redraw();
        }
        #[cfg(not(target_arch = "wasm32"))]
        window_target.set_control_flow(ControlFlow::Poll);
        #[cfg(target_arch = "wasm32")]
        window_target.set_control_flow(ControlFlow::Wait);
        #[cfg(target_arch = "wasm32")] {
            if unsafe {WEB_RESIZED} {
                let mut window_size = {
                    let win = web_sys::window().unwrap();
                    winit::dpi::LogicalSize::new(win.inner_width().unwrap().as_f64().unwrap(), win.inner_height().unwrap().as_f64().unwrap())
                };
                window_size.width = window_size.width.max(1.0);
                window_size.height = window_size.height.max(1.0);
                let _ = window.borrow().request_inner_size(window_size);
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
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if frame_count == 0 && size.height != 1 && size.width != 1 && size.width != new_size.width && size.height != new_size.height {
                    window.borrow().request_redraw();
                    return;
                }
                // Reconfigure the surface with the new size
                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(&device, &config);

                // okay gamers lets resize the screen & camera matrix buffers
                // from atlas :D
                let max = max(new_size.width, new_size.height);
                state.camera.aspect = (new_size.width as f32 / max as f32, new_size.height as f32 / max as f32);
                state.camera.view = [
                        [(new_size.height as f32) / max as f32, 0.0, 0.0, 0.0], 
                        [0.0, (new_size.width as f32) / max as f32, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0]
                ];
                queue.write_buffer(&camera_matrix_buffer, 0, bytemuck::cast_slice(&[CameraMatrix {
                    view: state.camera.view,
                    camera: state.camera.get_transformation_matrix()
                }]));
                // On macos the window needs to be redrawn manually after resizing
                window.borrow().request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                /*
                // Some scheduler code, it helps with GPU and CPU usage but leads to sudden FPS drops
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Attempt to sync with display refresh rate
                    let target_fps = calculate_target_fps(&window.borrow());
                    let target_frame_time = 1.0 / target_fps;

                    let current_time = std::time::Instant::now();
                    let elapsed_frame_time = current_time.duration_since(last_frame_time).as_secs_f32();

                    if elapsed_frame_time < target_frame_time {
                        let sleep_time = target_frame_time - elapsed_frame_time;
                        let sleep_until = std::time::Instant::now() + std::time::Duration::from_secs_f32(sleep_time);
                        std::thread::sleep(sleep_until - std::time::Instant::now());
                    }

                    last_frame_time = std::time::Instant::now();
                }*/
                // okay lets start
                #[allow(unused_mut)]
                let mut updates = Vec::new();
                #[cfg(feature = "editor")]
                {
                    let mut ecs = ecs.write().unwrap();
                    let client_events = ecs.poll_client_events();
                    for event in client_events {
                        match event {
                            crate::editor::ClientEvent::SelectEntity(id) => {
                                if let Some(entity) = ecs.get_entity(id) {
                                    selected_entity = Some(entity.clone());
                                    let entity = (*entity).lock().unwrap();
                                    let as_entity = entity.as_entity();
                                    let entity_bounds = get_entity_screen_space_bounds(&state.camera, &mut entity.transform.lock().unwrap(), frame_count - 1).unwrap();
                                    ecs.emit(crate::editor::Event::EntitySelected(as_entity));
                                    ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                                }
                            },
                            crate::editor::ClientEvent::SetComponentProperty { entity, component, field, property } => {
                                if let Some(entity) = ecs.get_entity(entity) {
                                    {
                                        let mut entity = (*entity).lock().unwrap();
                                        let component = entity.get_component_mut(component).unwrap();
                                        component.set_property(field.as_str(), property);
                                    }
                                    updates.extend(ecs.run_component_methods(&mut scripting, ComponentFlags::EDITOR_UPDATE));
                                }
                            },
                            crate::editor::ClientEvent::MoveSelected(x, y) => {
                                if let Some(entity) = &selected_entity {
                                    let entity = (*entity).lock().unwrap();
                                    let (x, y) = crate::util::scaling::as_world_scale(&state.camera, (x, y));
                                    {
                                        let mut rtransform = entity.transform.lock().unwrap();
                                        match rtransform.transform {
                                            Transform::Transform2D {ref mut position, ..} => {
                                                position.0 += x;
                                                position.1 += y;
                                            },
                                            Transform::RectTransform { .. } => {

                                            }
                                        }
                                        rtransform.has_changed = true;
                                    }
                                    {
                                        let mut rtransform = entity.transform.lock().unwrap();
                                        let entity_bounds = get_entity_screen_space_bounds(&state.camera, &mut rtransform, frame_count - 1).unwrap();
                                        ecs.emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                                    }
                                }
                            },
                            crate::editor::ClientEvent::SaveScene => {
                                #[cfg(target_arch = "wasm32")]
                                crate::editor::save_scene(ecs.as_scene().to_json());
                            }
                        }
                    }
                }

                #[cfg(not(feature = "direct_asset_management"))]
                if !ecs_initialized {
                    let scene: Option<Scene> = {
                        let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
                        let x = if let Some(static_shard) = &asset_manager.assets.lock().unwrap().static_shard {
                            // init scripts
                            scripting.initialize(static_shard.get_scripts().clone(), input_state.clone(), ecs.clone()).unwrap();
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
                        let mut ecs = ecs.write().unwrap();
                        ecs.load_scene(
                            scene, &mut scripting);
                        ecs_initialized = true;
                        log!("ECS initialized");
                    }
                } else {
                    #[allow(unused)]
                    let ecs = ecs.read().unwrap();
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
                        let ecs = ecs.read().unwrap();
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
                        let mut ecs = ecs.write().unwrap();
                        ecs.emit(crate::editor::Event::CameraChanged(state.camera.position.x, state.camera.position.y, state.camera.scale));
                    }
                }
                render_frame(&surface, &device, &queue, &mut drawables, &global_bind_group, ecs_initialized, frame_count);
                frame_count += 1;
                window.borrow().request_redraw();
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.borrow().id() => match event {
                WindowEvent::CloseRequested => window_target.exit(),
                WindowEvent::CursorMoved { position, ..} => {
                    let mut input_state = input_state.lock().unwrap();
                    input_state.mouse.last_position = Some(input_state.mouse.position);
                    input_state.mouse.position = (position.x as f32 / config.width as f32, position.y as f32 / config.height as f32);
                    #[cfg(feature = "editor")]
                    if input_state.mouse.right_button {
                        let delta = input_state.mouse.get_delta();
                        let world_scale_delta = crate::util::scaling::as_world_scale(&state.camera, (-delta.0 * state.camera.aspect.1, delta.1 * state.camera.aspect.0));
                        state.camera.position.x += world_scale_delta.0;
                        state.camera.position.y += world_scale_delta.1;
                        state.camera.dirty = true;
                        if let Some(entity) = &selected_entity {
                            let rentity = entity.lock().unwrap();
                            let entity_bounds = get_entity_screen_space_bounds(&state.camera, &mut rentity.transform.lock().unwrap(), frame_count - 1).unwrap();
                            ecs.write().unwrap().emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
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

                                let mut ecs = ecs.write().unwrap();
                                if let Some(entity) = find_overlapping_entity(&ecs, click_pos, frame_count - 1) {
                                    selected_entity = Some(entity.clone());
                                    let entity = entity.lock().unwrap();
                                    let as_entity = entity.as_entity();
                                    let entity_bounds = get_entity_screen_space_bounds(&state.camera, &mut entity.transform.lock().unwrap(), frame_count - 1).unwrap();
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
                        let rentity = entity.lock().unwrap();
                        let entity_bounds = get_entity_screen_space_bounds(&state.camera, &mut rentity.transform.lock().unwrap(), frame_count - 1).unwrap();
                        ecs.write().unwrap().emit(crate::editor::Event::SelectedEntityPosition(entity_bounds.0, entity_bounds.1, entity_bounds.2, entity_bounds.3));
                    }
                },
                WindowEvent::KeyboardInput { event, .. } => {
                    let mut input_state = input_state.lock().unwrap();
                    // check if we should add or remove the key
                    if event.state == ElementState::Released {
                        input_state.down_keys.retain(|x| *x != event.logical_key);
                        input_state.up_keys.push(event.logical_key.clone());
                    } else {
                        if !input_state.down_keys.contains(&event.logical_key) {
                            input_state.down_keys.push(event.logical_key.clone());
                            input_state.new_keys.push(event.logical_key.clone());
                        }
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }).unwrap();
}
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn calculate_target_fps(window: &winit::window::Window) -> f32 {
    let monitor = window.current_monitor().unwrap();
    let video_mode = monitor.video_modes().next().unwrap();
    let refresh_rate = video_mode.refresh_rate_millihertz() as f32 / 1000.0;
    refresh_rate
}

#[allow(dead_code)]
fn get_entity_screen_space_bounds(camera: &CameraState, rtransform: &mut RuntimeTransform, frame_num: u64) -> Option<(f32, f32, f32, f32)> {
    let (position, _rotation, scale) = rtransform.eval_transform(frame_num);
    let screen_pos = scaling::as_screen_pos(camera, position);
    let screen_scale = scaling::as_screen_scale(camera, scale);
    Some((screen_pos.0, screen_pos.1, screen_scale.0, screen_scale.1))
}

#[allow(dead_code)]
fn find_overlapping_entity<T>(ecs: &ECS<T>, check_position: (f32, f32), frame_num: u64) -> Option<Arc<Mutex<RuntimeEntity<T>>>> where T: ScriptingInstance {
    for e in ecs.get_all_runtime_entities_flat() {
        let entity = e.lock().unwrap();
        let mut rtransform = entity.transform.lock().unwrap();
        let (position, _rotation, scale) = rtransform.eval_transform(frame_num);
        if position.0 - scale.0 / 2.0 <= check_position.0 && position.0 + scale.0 / 2.0 >= check_position.0 &&
            position.1 - scale.0 / 2.0 <= check_position.1 && position.1 + scale.1 / 2.0 >= check_position.1 {
                return Some(e.clone());
            }
    }
    None
}

fn process_entity_updates(device: &wgpu::Device, 
                          asset_manager: &AssetManager, 
                          shader_manager: &ShaderManager,
                          drawables: &mut Vec<Box<dyn Drawable>>,
                          updates: Vec<(Arc<Mutex<RuntimeTransform>>, Vec<EntityUpdate>)>) {
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
                        if drawables[i].get_id() == id {
                            drawables.remove(i);
                            break;
                        }
                    }
                },
                EntityUpdate::SetDrawableProperty(id, field_name, property) => {
                    // NOTE: Same as above, maybe use a hashmap?
                    for drawable in &mut *drawables {
                        if drawable.get_id() == id {
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
        .create_view(&wgpu::TextureViewDescriptor {
            format: Some(unsafe { TARGET_FORMAT }),
            ..Default::default()
        });

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
