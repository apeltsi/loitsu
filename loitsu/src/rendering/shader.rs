use super::vertex::Vertex;
use std::{collections::HashMap, rc::Rc};

use wgpu::{Device, PrimitiveState, RenderPipeline, ShaderModule};

use crate::log;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Shader {
    shader: ShaderModule,
    pipeline: RenderPipeline,
    bindings: Vec<wgpu::BindGroupLayout>,
}

pub struct ShaderManager<'a> {
    shaders: HashMap<&'a str, Rc<Shader>>,
}

impl Shader {
    pub fn get_pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}

impl<'a> ShaderManager<'a> {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    pub fn load_default_shaders(&mut self, device: &Device) {
        {
            // spriterenderer
            let mut bindings = Vec::new();
            bindings.push(crate::rendering::core::get_global_bind_group_layout(device));
            bindings.push(crate::rendering::core::get_sprite_bind_group_layout(device));
            self.load_shader(
                device,
                "sprite",
                include_str!("shaders/sprite.wgsl"),
                bindings,
            );
        }
        log!("Default shaders loaded");
    }

    pub fn load_shader(
        &mut self,
        device: &Device,
        name: &'a str,
        shader: &str,
        bindings: Vec<wgpu::BindGroupLayout>,
    ) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{}_shader", name).as_str()),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });

        let mut render_pipeline_layout = None;
        if bindings.len() > 0 {
            let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = bindings.iter().collect();
            render_pipeline_layout = Some(device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some(format!("{}_pipeline_layout", name).as_str()),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                },
            ));
        }

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{}_pipeline", name).as_str()),
            layout: render_pipeline_layout.as_ref(),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: unsafe { crate::rendering::core::TARGET_FORMAT },
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: Default::default(),
            primitive: PrimitiveState {
                // we have a list of indices and vertices
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
        });
        let shader = Shader {
            shader,
            pipeline,
            bindings,
        };
        self.shaders.insert(name, Rc::new(shader));
    }

    pub fn get_shader(&self, name: &str) -> Option<Rc<Shader>> {
        self.shaders.get(name).cloned()
    }
}
