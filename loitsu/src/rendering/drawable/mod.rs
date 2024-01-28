pub mod sprite;
use std::sync::{Arc, Mutex};

use crate::{asset_management::AssetManager, ecs::RuntimeTransform};
use wgpu::RenderPass;

use super::vertex::Vertex;

pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // C
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // D
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub enum DrawablePrototype {
    Sprite {
        sprite: String,
        color: [f32; 4],
        id: uuid::Uuid,
    },
}

#[derive(Debug, Clone)]
pub enum DrawableProperty {
    Sprite(String),
    Color([f32; 4]),
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    transform: [[f32; 4]; 4],
}
impl TransformUniform {
    pub fn new(transform: &mut RuntimeTransform, frame_num: u64) -> Self {
        Self {
            transform: transform.eval_transform_mat(frame_num),
        }
    }
}
pub trait Drawable<'b> {
    fn init<'a>(
        &mut self,
        device: &wgpu::Device,
        asset_manager: &AssetManager,
        transform: Arc<Mutex<RuntimeTransform>>,
    ) where
        'a: 'b;
    fn draw<'a>(
        &'a mut self,
        frame_num: u64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut RenderPass<'a>,
        global_bind_group: &'a wgpu::BindGroup,
    );
    fn get_uuid(&self) -> uuid::Uuid;
    fn set_property(&mut self, name: String, property: DrawableProperty);
}
