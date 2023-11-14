use std::rc::Rc;

use wgpu::util::DeviceExt;
use wgpu::RenderPass;
use super::{Drawable, QUAD_INDICES, QUAD_VERTICES};
use crate::{rendering::shader::ShaderManager, asset_management::asset::Asset};
pub struct SpriteDrawable {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    shader: Rc<crate::rendering::shader::Shader>,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    sprite: String,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteUniform {
    color: [f32; 4],
}

impl<'a> SpriteDrawable {
    pub fn new(sprite: &str, shader_manager: &ShaderManager) -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            shader: shader_manager.get_shader("sprite").unwrap(),
            bind_group: None,
            uniform_buffer: None,
            sprite: sprite.to_string(),
        }
    }
}

impl<'b> Drawable<'b> for SpriteDrawable {
    fn init<'a>(&mut self, device: &wgpu::Device) where 'a: 'b {
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
        let asset_manager = crate::asset_management::ASSET_MANAGER.lock().unwrap();
        let asset = asset_manager.get_asset(&self.sprite).expect("Asset not found!");
        let locked_asset = asset.lock().unwrap();
        let sprite_asset = match *locked_asset {
            Asset::Image(ref image_asset) => Some(image_asset),
            _ => None,
        };
        self.uniform_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Uniform Buffer"),
            contents: bytemuck::cast_slice(&[SpriteUniform {
                color: [1.0, 1.0, 1.0, 1.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }));
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &crate::rendering::core::get_sprite_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(sprite_asset.unwrap().get_texture_view().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(crate::rendering::core::get_default_sampler().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
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

    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, global_bind_group: &'a wgpu::BindGroup) {
        pass.set_pipeline(self.shader.get_pipeline());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(0, global_bind_group, &[]);
        pass.set_bind_group(1, self.bind_group.as_ref().unwrap(), &[]);
        pass.draw(0..4, 0..1);
    }
}
