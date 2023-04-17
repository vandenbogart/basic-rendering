use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::{loaders::gltf::{GltfFile, DrawGltf, GltfFrameState}, window::Window};

use super::{
    pipeline_default::DefaultPipeline,
    Globals, Locals,
};

pub struct Renderer {
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub default_pipeline: DefaultPipeline,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let size = window.window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window.window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let default_pipeline = DefaultPipeline::new(&device);

        let (depth_texture, depth_texture_view) =
            Renderer::init_depth_texture(&device, &surface_config);

        Renderer {
            default_pipeline,
            queue,
            device,
            surface,
            surface_config,
            depth_texture,
            depth_texture_view,
        }
    }
    fn init_depth_texture(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
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

    pub fn draw(&self, gltfs: &mut Vec<GltfFrameState>, view_proj: cgmath::Matrix4<f32>) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let globals = Globals {
            view_proj: view_proj.into(),
            ambient_color: [1.0, 1.0, 1.0, 1.0],
            ambient_strength: [0.2, 0.0, 0.0, 0.0],
        };
        let globals_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Globals buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[globals]),
            });
        let globals_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Locals Bind group"),
            layout: &self.default_pipeline.globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let locals = Locals {
            diffuse_light_color: [1.0, 1.0, 1.0, 1.0],
            diffuse_light_pos: [30.0, 100.0, 0.0, 0.0],
        };
        let locals_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Locals buffer"),
                contents: bytemuck::cast_slice(&[locals]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let locals_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Locals Bind group"),
            layout: &self.default_pipeline.locals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: locals_buffer.as_entire_binding(),
            }],
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Pass Encoder"),
            });
        for frame_state in gltfs.iter_mut() {
            frame_state.init_buffers(&self.device);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(0, &globals_bind_group, &[]);
            render_pass.set_bind_group(1, &locals_bind_group, &[]);

            for gltf in gltfs.iter() {
                render_pass.draw_gltf(gltf);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
