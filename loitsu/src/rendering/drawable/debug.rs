use wgpu::util::DeviceExt;
use wgpu::RenderPass;
use super::{Drawable, QUAD_INDICES, QUAD_VERTICES};
use crate::rendering::shader::ShaderManager;

pub struct DebugDrawable<'a> {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    shader: Option<&'a crate::rendering::shader::Shader>
}
impl DebugDrawable<'_> {
    pub fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            shader: None
        }
    }
}
impl Drawable for DebugDrawable<'_> {
    fn init(&mut self, device: &wgpu::Device, shader_manager: &ShaderManager) {
        // init vertex buffer
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        self.shader = shader_manager.get_shader("sprite").as_deref();
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }));
    }

    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, shader_manager: &'a ShaderManager<'a>, _global_bind_group: &'a wgpu::BindGroup){
        pass.set_pipeline(self.shader.unwrap().get_pipeline());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        pass.draw(0..3, 0..1);
    }
}
