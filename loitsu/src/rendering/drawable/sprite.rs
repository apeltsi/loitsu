use std::{rc::Rc, cell::RefCell};
use wgpu::util::DeviceExt;
use wgpu::RenderPass;
use super::{Drawable, QUAD_INDICES, QUAD_VERTICES, TransformUniform};
use crate::{rendering::shader::ShaderManager, asset_management::{asset::Asset, AssetManager}, ecs::Transform};
pub struct SpriteDrawable {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    shader: Rc<crate::rendering::shader::Shader>,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    transform_buffer: Option<wgpu::Buffer>,
    initial_uniform: SpriteUniform,
    sprite: String,
    transform: Option<Rc<RefCell<Transform>>>,
    uuid: uuid::Uuid,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteUniform {
    color: [f32; 4],
}

impl<'a> SpriteDrawable {
    pub fn new(sprite: &str, color: [f32; 4], uuid: uuid::Uuid, shader_manager: &ShaderManager) -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            shader: shader_manager.get_shader("sprite").unwrap(),
            bind_group: None,
            uniform_buffer: None,
            transform_buffer: None,
            initial_uniform: SpriteUniform {
                color,
            },
            sprite: sprite.to_string(),
            transform: None,
            uuid
        }
    }
}

impl<'b> Drawable<'b> for SpriteDrawable {
    fn init<'a>(&mut self, device: &wgpu::Device, asset_manager: &AssetManager, transform: Rc<RefCell<Transform>>) where 'a: 'b {
        // init vertex buffer
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }));
        let asset = asset_manager.get_asset(&self.sprite).expect("Asset not found!");
        let locked_asset = asset.lock().unwrap();
        let sprite_asset = match *locked_asset {
            Asset::Image(ref image_asset) => Some(image_asset),
        };
        self.uniform_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Uniform Buffer"),
            contents: bytemuck::cast_slice(&[self.initial_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }));
        let initial_transform = TransformUniform::new(transform.borrow().clone());
        self.transform = Some(transform.clone());
        self.transform_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Transform Buffer"),
            contents: bytemuck::cast_slice(&[initial_transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }));
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &crate::rendering::core::get_sprite_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.transform_buffer.as_ref().unwrap(),
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<[[f32; 4]; 4]>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(sprite_asset.unwrap().get_texture_view().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(crate::rendering::core::get_default_sampler().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.uniform_buffer.as_ref().unwrap(),
                        offset: 0,
                        size: None,
                    })
                }
            ],
            label: Some("sprite_bind_group"),
        }));
    }

    fn draw<'a>(&'a self, frame_num: u64, queue: &wgpu::Queue, pass: &mut RenderPass<'a>, global_bind_group: &'a wgpu::BindGroup) {
        if self.transform.is_none() {
            return;
        }
        if self.transform.clone().unwrap().borrow_mut().check_changed(frame_num) {
            let transform = TransformUniform::new(self.transform.clone().unwrap().borrow().clone());
            queue.write_buffer(self.transform_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&[transform]));
        }
        pass.set_pipeline(self.shader.get_pipeline());
        pass.set_bind_group(0, global_bind_group, &[]);
        pass.set_bind_group(1, self.bind_group.as_ref().unwrap(), &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
    }

    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}
