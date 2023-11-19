pub mod sprite;
use wgpu::RenderPass;
use crate::asset_management::AssetManager;

use super::vertex::Vertex;

pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // A
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // B
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // C
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // D
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub enum DrawablePrototype {
    Sprite {sprite: String, color: [f32; 4]}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    transform: [[f32; 4]; 4],
}
impl TransformUniform {
    pub fn new() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }
}
pub trait Drawable<'b> {
    fn init<'a>(&mut self, device: &wgpu::Device, asset_manager: &AssetManager) where 'a: 'b; 
    fn draw<'a>(&'a self, queue: &wgpu::Queue, pass: &mut RenderPass<'a>, global_bind_group: &'a wgpu::BindGroup);
}
