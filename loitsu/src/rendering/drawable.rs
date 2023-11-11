use wgpu::RenderPass;
use wgpu::util::DeviceExt;
use super::shader::ShaderManager;
use super::vertex::Vertex;

const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 1.0] }, // A
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0] }, // B
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 1.0] }, // C
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 0.0] }, // D
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

pub trait Drawable {
    fn init(&mut self, device: &wgpu::Device, shader_manager: &ShaderManager);
    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, shader_manager: &'a ShaderManager<'a>);
}

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
impl Drawable for DebugDrawable {
    fn init(&mut self, device: &wgpu::Device, _shader_manager: &ShaderManager) {
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

    fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, shader_manager: &'a ShaderManager<'a>){
        pass.set_pipeline(shader_manager.get_shader("debug").unwrap().get_pipeline());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        pass.draw(0..3, 0..1);
    }
}
