//pub mod debug;
pub mod sprite;
use wgpu::RenderPass;
use super::vertex::Vertex;

pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 1.0] }, // A
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0] }, // B
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 1.0] }, // C
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 0.0] }, // D
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub trait Drawable<'b> {
    fn init<'a>(&mut self, device: &wgpu::Device) where 'a: 'b; 
    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, global_bind_group: &'a wgpu::BindGroup);
}
