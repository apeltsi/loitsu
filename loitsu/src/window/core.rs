pub fn render_frame(surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
    let frame = surface
    .get_current_texture()
    .expect("Failed to acquire next swap chain texture");
let view = frame
    .texture
    .create_view(&wgpu::TextureViewDescriptor::default());

let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

// Lets clear the main texture
{
    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Clear Texture"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: false
            }
        })],
        depth_stencil_attachment: None,
    });
}
// TODO: Here well loop through our drawables :DDD

queue.submit(Some(encoder.finish()));
frame.present();
}