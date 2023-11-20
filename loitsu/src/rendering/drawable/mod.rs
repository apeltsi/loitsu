pub mod sprite;
use std::{rc::Rc, cell::RefCell};

use wgpu::RenderPass;
use crate::{asset_management::AssetManager, ecs::Transform};

use super::vertex::Vertex;

pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // A
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // B
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // C
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // D
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub enum DrawablePrototype {
    Sprite {sprite: String, color: [f32; 4], id: uuid::Uuid},
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    transform: [[f32; 4]; 4],
}
impl TransformUniform {
    pub fn new(transform: Transform) -> Self {
        match transform {
            Transform::Transform2D { position, scale, .. } => {
                Self {
                    transform: [ // TODO: Rotation here
                        [scale.0, 0.0, 0.0, position.0],
                        [0.0, scale.1, 0.0, position.1],
                        [0.0, 0.0, 1.0 , 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ]
                }
            },
            Transform::RectTransform { position } => {
                Self {
                    // TODO: THIS
                    transform: [
                        [1.0, 0.0, 0.0, position.0],
                        [0.0, 1.0, 0.0, position.1],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ]
                }
            }
        }
    }
}
pub trait Drawable<'b> {
    fn init<'a>(&mut self, device: &wgpu::Device, asset_manager: &AssetManager, transform: Rc<RefCell<Transform>>) where 'a: 'b; 
    fn draw<'a>(&'a self, queue: &wgpu::Queue, pass: &mut RenderPass<'a>, global_bind_group: &'a wgpu::BindGroup);
    fn get_uuid(&self) -> uuid::Uuid;
}
