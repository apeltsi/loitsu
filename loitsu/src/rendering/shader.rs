use std::collections::HashMap;

use wgpu::{Device, ShaderModule, RenderPipeline, PrimitiveState};

use crate::log;
#[derive(Debug)]
#[allow(dead_code)]
pub struct Shader {
    shader: ShaderModule,
    pipeline: RenderPipeline
}


pub struct ShaderManager<'a> {
    shaders: HashMap<&'a str, Shader>,
}

impl Shader {
    pub fn get_pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}

impl<'a> ShaderManager<'a> {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new()
        }
    }
    
    pub fn load_default_shaders(&mut self, device: &Device) {
        self.load_shader(device, "debug", include_str!("shaders/debug.wgsl"));
        log!("Default shaders loaded");
    }

    pub fn load_shader(&mut self, device: &Device, name: &'a str, shader: &str) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{}_shader", name).as_str()),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{}_pipeline", name).as_str()),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: unsafe { crate::rendering::core::TARGET_FORMAT },
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: Default::default(),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false
            }
        });
        
        self.shaders.insert(name, Shader {
            shader,
            pipeline,
        });
    }

    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name).map(|s| s)
    }
} 
