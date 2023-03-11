use std::{cell::Ref, collections::HashMap, path::Path};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferUsages,
};

use crate::window::Window;

use super::{pipeline_default::DefaultPipeline, Globals, Instance, Locals, draw_state::{DrawState, Drawable, DrawStateBuilder}};

pub struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    globals_buffer: Buffer,
    globals_bind_group: BindGroup,
    default_pipeline: DefaultPipeline,
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
        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals buffer"),
            size: std::mem::size_of::<Globals>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let default_pipeline = DefaultPipeline::new(&device, &surface_config);

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals bind_group"),
            layout: &default_pipeline.render_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let (depth_texture, depth_texture_view) = Renderer::init_depth_texture(&device, &surface_config);

        Renderer {
            globals_buffer,
            globals_bind_group,
            default_pipeline,
            queue,
            device,
            surface,
            surface_config,
            depth_texture,
            depth_texture_view,
        }
    }
    fn init_depth_texture(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView) {
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
    pub fn get_draw_state_builder(&self) -> DrawStateBuilder {
        DrawStateBuilder::new(&self.device)
    }
    pub fn draw(&self, draw_state: DrawState) {
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&draw_state.globals));
        let locals = Locals {
            diffuse_light_color: [1.0, 1.0, 1.0, 1.0],
            diffuse_light_pos: [30.0, 100.0, 0.0, 0.0],
        };
        let locals_buffer = self
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Locals buffer"),
                contents: bytemuck::cast_slice(&[locals]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
        let locals_bind_group = self
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Locals Bind group"),
                layout: &self
                    .default_pipeline
                    .render_pipeline
                    .get_bind_group_layout(1),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: locals_buffer.as_entire_binding(),
                }],
            });
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Pass Encoder"),
                });

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

            render_pass.set_pipeline(&self.default_pipeline.render_pipeline);
            render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
            render_pass.set_bind_group(1, &locals_bind_group, &[]);
            for drawable in draw_state.drawables.iter() {
                render_pass.set_vertex_buffer(0, drawable.vertex_buf.slice(..));
                render_pass.set_index_buffer(drawable.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(1, drawable.instance_buf.slice(..));
                render_pass.draw_indexed(0..drawable.num_indices, 0, 0..drawable.num_instances);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
