use wgpu::{ColorTargetState, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor};

use super::{Instance, Vertex};

pub struct DefaultPipeline {
    pub render_pipeline: RenderPipeline,
}

impl DefaultPipeline {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic PipelineShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let depth_stencil = wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState::default(),
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_write_enabled: true,
            format: wgpu::TextureFormat::Depth32Float,
            stencil: wgpu::StencilState::default(),
        };

        let fragment = wgpu::FragmentState {
            entry_point: "fs_main",
            module: &shader,
            targets: &[Some(ColorTargetState::from(surface_config.format))],
        };

        let vertex = wgpu::VertexState {
            buffers: &[Vertex::layout(), Instance::layout()],
            module: &shader,
            entry_point: "vs_main",
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

        let layout_descriptor = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Basic Render Layout"),
            bind_group_layouts: &[&globals_bind_group_layout, &locals_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            depth_stencil: Some(depth_stencil),
            fragment: Some(fragment),
            label: Some("Basic Render Pipeline"),
            layout: Some(&layout_descriptor),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            vertex,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
        });
        DefaultPipeline { render_pipeline }
    }
}
