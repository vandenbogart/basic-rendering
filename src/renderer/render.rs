use std::{cell::Ref, collections::HashMap, path::Path};

use cgmath::One;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferUsages, Texture, TextureView,
};

use crate::{window::Window, EntityId, ResourceIndex};

use super::{
    context::Context, loaders::obj::load_model, pipeline_default::DefaultPipeline,
    GeometryComponent, Globals, Instance, Locals, ModelComponent, ModelResource,
};

pub struct Renderer {
    pub context: Context,
    globals: Globals,
    globals_buffer: Buffer,
    globals_bind_group: BindGroup,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    default_pipeline: DefaultPipeline,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let context = Context::new(window)
            .await
            .expect("Failed to initialize context");
        let globals = Globals {
            view_proj: cgmath::Matrix4::one().into(),
            ambient_strength: 0.1,
            ambient_color: [0.0, 1.0, 0.0],
        };
        let globals_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals buffer"),
            size: std::mem::size_of::<Globals>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let default_pipeline = DefaultPipeline::new(&context);

        let globals_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("globals bind_group"),
                layout: &default_pipeline.render_pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                }],
            });

        let (depth_texture, depth_texture_view) = Renderer::init_depth_texture(&context);

        Renderer {
            context,
            globals,
            globals_buffer,
            globals_bind_group,
            depth_texture,
            depth_texture_view,
            default_pipeline,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
        let (depth_texture, depth_texture_view) = Renderer::init_depth_texture(&self.context);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }
    fn init_depth_texture(context: &Context) -> (Texture, TextureView) {
        let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: context.surface_config.width,
                height: context.surface_config.height,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (depth_texture, depth_texture_view)
    }
    pub async fn create_model_resource(&self, filepath: String) -> ModelResource {
        dbg!(&filepath);
        let model = load_model(
            &self.context.queue,
            &self.context.device,
            Path::new(&filepath),
        )
        .await
        .unwrap();
        ModelResource::new(model)
    }
    pub fn draw(&mut self, world: &mut crate::World, view_proj: cgmath::Matrix4<f32>) {
        self.globals.view_proj = view_proj.into();
        self.context.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::cast_slice(&[self.globals]),
        );
        let locals = Locals {
            position: [0.0, 0.0, 0.0, 0.0],
            diffuse_light_color: [1.0, 1.0, 1.0, 0.0],
            diffuse_light_pos: [0.0, 5.0, 0.0, 0.0],
        };
        let locals_buffer = self
            .context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Locals buffer"),
                contents: bytemuck::cast_slice(&[locals]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
        let locals_bind_group = self
            .context
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
        let output = self.context.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Pass Encoder"),
                });

        let result = world
            .query()
            .with_component::<ModelComponent>()
            .with_component::<GeometryComponent>()
            .execute();
        let entities = result.get_entities();

        let entities_model_index = entities
            .iter()
            .map(|e| result.get_component::<ModelComponent>(*e).model_index);
        let mut model_index_entity_map: HashMap<ResourceIndex, Vec<EntityId>> = HashMap::new();
        for (entity_dx, resource) in entities_model_index.enumerate() {
            if model_index_entity_map.contains_key(&resource) {
                model_index_entity_map
                    .get_mut(&resource)
                    .unwrap()
                    .push(entities[entity_dx])
            } else {
                model_index_entity_map.insert(resource, vec![entities[entity_dx]]);
            }
        }

        let instance_buffers = model_index_entity_map
            .iter()
            .map(|(resource_idx, ent_ids)| {
                let mut pre_buf = Vec::new();
                for ent_id in ent_ids {
                    let geo = result.get_component::<GeometryComponent>(*ent_id);
                    let instance = Instance {
                        position: geo.position,
                        rotation: geo.rotation,
                    };
                    pre_buf.push(instance.to_raw());
                }
                (
                    *resource_idx,
                    self.context
                        .device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("int buf"),
                            contents: bytemuck::cast_slice(&pre_buf),
                            usage: BufferUsages::VERTEX,
                        }),
                )
            })
            .collect::<HashMap<ResourceIndex, Buffer>>();

        let models = instance_buffers
            .iter()
            .map(|(resource_index, _)| {
                (
                    *resource_index,
                    world.get_resource::<ModelResource>(*resource_index),
                )
            })
            .collect::<HashMap<ResourceIndex, Ref<ModelResource>>>();

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
            for (resource_index, instance_buffer) in instance_buffers.iter() {
                let model = models.get(resource_index).unwrap();
                let num_instances = model_index_entity_map.get(&resource_index).unwrap().len();
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                for mesh in model.model.meshes.iter() {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..(num_instances as u32));
                }
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
