use wgpu::RenderPass;

use super::shader::ShaderManager;

pub trait Drawable {
    fn init(&self, device: &wgpu::Device, shader_manager: &ShaderManager);
    fn draw<'a>(&self, pass: &mut RenderPass<'a>, shader_manager: &'a ShaderManager<'a>);
}

pub struct DebugDrawable {}

impl Drawable for DebugDrawable {

    fn init(&self, _device: &wgpu::Device, _shader_manager: &ShaderManager) {
    }

    fn draw<'a>(&self, pass: &mut RenderPass<'a>, shader_manager: &'a ShaderManager<'a>){
        pass.set_pipeline(shader_manager.get_shader("debug").unwrap().get_pipeline());
        pass.draw(0..3, 0..1);
    }
}
