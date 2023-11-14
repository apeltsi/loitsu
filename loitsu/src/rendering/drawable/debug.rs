use wgpu::util::DeviceExt;
use wgpu::RenderPass;
use super::{Drawable, QUAD_INDICES, QUAD_VERTICES};
use crate::rendering::shader::ShaderManager;

pub struct DebugDrawable {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}
impl DebugDrawable {
    pub fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
impl<'b> Drawable<'b> for DebugDrawable {
    fn init<'a>(&mut self, device: &wgpu::Device, _shader_manager: &ShaderManager) where 'a: 'b {
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
    }

    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, shader_manager: &ShaderManager<'a>, _global_bind_group: &'a wgpu::BindGroup){
        //pass.set_pipeline(shader_manager.get_shader("debug").unwrap().get_pipeline());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        pass.draw(0..3, 0..1);
    }
}
