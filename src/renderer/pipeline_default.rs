use std::num::NonZeroU32;

use wgpu::MultisampleState;

pub struct DefaultPipeline {
    pub shader: wgpu::ShaderModule,
    pub pn_shader: wgpu::ShaderModule,
    pub depth_stencil: wgpu::DepthStencilState,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,
    pub locals_bind_group_layout: wgpu::BindGroupLayout,
    pub layout: wgpu::PipelineLayout,
    pub multisample: wgpu::MultisampleState,
    pub multiview: Option<NonZeroU32>,
    pub primitive: wgpu::PrimitiveState,
}

impl DefaultPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic PipelineShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pnc.wgsl").into()),
        });
        let pn_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic PipelineShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pn.wgsl").into()),
        });

        let depth_stencil = wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState::default(),
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_write_enabled: true,
            format: wgpu::TextureFormat::Depth32Float,
            stencil: wgpu::StencilState::default(),
        };
        
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: wgpu::ShaderStages::all(),
                }],
                label: Some("Globals Bind Group"),
            });

        let locals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: wgpu::ShaderStages::all(),
                }],
                label: Some("Locals Bind Group"),
            });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Basic Render Layout"),
            bind_group_layouts: &[&globals_bind_group_layout, &locals_bind_group_layout],
            push_constant_ranges: &[],
        });

        DefaultPipeline {
            shader,
            depth_stencil,
            globals_bind_group_layout,
            locals_bind_group_layout,
            layout, 
            multisample: MultisampleState::default(),
            multiview: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            pn_shader,
        }
    }
}
