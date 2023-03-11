

use wgpu::{
    util::DeviceExt, vertex_attr_array, ColorTargetState, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureView,
};

use super::{::Context};

pub struct DepthPipeline {
    pub render_pipeline: RenderPipeline,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct DepthVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const DEPTH_VERTICES: &[DepthVertex] = &[
    DepthVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    DepthVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    DepthVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    DepthVertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

const DEPTH_INDICES: &[u32] = &[0, 1, 2, 2, 3, 0];

impl DepthPipeline {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let (depth_texture, depth_texture_view) = DepthPipeline::init_depth_texture();
        let shader = 
            device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Depth shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("depth.wgsl").into()),
            });

        let fragment = wgpu::FragmentState {
            entry_point: "fs_main",
            module: &shader,
            targets: &[Some(ColorTargetState::from(surface_config.format))],
        };

        let vertex = wgpu::VertexState {
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<DepthVertex>() as wgpu::BufferAddress,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                step_mode: wgpu::VertexStepMode::Vertex,
            }],
            module: &shader,
            entry_point: "vs_main",
        };

        let bind_group_layout =
            
                device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            count: None,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            visibility: wgpu::ShaderStages::FRAGMENT,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            count: None,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            visibility: wgpu::ShaderStages::FRAGMENT,
                        },
                    ],
                    label: Some("Depth Bind Group"),
                });

        let sampler =device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let bind_group = 
            device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("Depth Bind Group"),
                layout: &bind_group_layout,
            });

        let vertex_buffer = 
            device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                contents: bytemuck::cast_slice(DEPTH_VERTICES),
                label: Some("Depth vt buf"),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = 
            device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                contents: bytemuck::cast_slice(DEPTH_INDICES),
                label: Some("Depth ind buf"),
                usage: wgpu::BufferUsages::INDEX,
            });

        let layout_descriptor = 
            device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Depth Render Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = 
            device
            .create_render_pipeline(&RenderPipelineDescriptor {
                depth_stencil: None,
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



        DepthPipeline {
            render_pipeline,
            depth_texture,
            depth_texture_view,
            bind_group,
            vertex_buffer,
            sampler,
            bind_group_layout,
            index_buffer,
        }
    }

    fn init_depth_texture(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture =device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (depth_texture, depth_texture_view)
    }

    pub fn resize(&mut self, ) {
        let (depth_texture, depth_texture_view) = DepthPipeline::init_depth_texture();
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
        self.bind_group = 
            device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.depth_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("Depth Bind Group"),
                layout: &self.bind_group_layout,
            });
    }

    pub fn make_pass(&self, view: &TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Depth Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..DEPTH_INDICES.len() as u32, 0, 0..1);
        drop(render_pass);
        
    }
}
