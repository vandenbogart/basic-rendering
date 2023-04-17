use std::collections::HashMap;

use cgmath::{One, VectorSpace};
use wgpu::{util::DeviceExt, vertex_attr_array};

use crate::{components::model::AnimationState, renderer::render::Renderer};

type Signature = String;

pub trait DrawGltf<'a> {
    fn draw_gltf(&mut self, gltf: &'a GltfFrameState);
}
impl<'a> DrawGltf<'a> for wgpu::RenderPass<'a> {
    fn draw_gltf(&mut self, gltf: &'a GltfFrameState) {
        for (mesh_idx, mesh) in gltf.gltf_file.meshes.iter().enumerate() {
            for (prim_idx, primitive) in mesh.primitives.iter().enumerate() {
                let pipeline = gltf
                    .gltf_file
                    .render_pipelines
                    .get(&primitive.signature)
                    .unwrap();
                self.set_pipeline(pipeline);
                self.set_vertex_buffer(
                    0,
                    gltf.meshes[mesh_idx][prim_idx]
                        .instance_buffer
                        .as_ref()
                        .unwrap()
                        .slice(..),
                );
                for (i, buffer) in primitive.vertex_buffers.iter().enumerate() {
                    self.set_vertex_buffer((i + 1) as u32, buffer.slice(..));
                }
                self.set_index_buffer(primitive.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                self.draw_indexed(
                    0..primitive.num_indices,
                    0,
                    0..primitive.instances.len() as u32,
                );
            }
        }
    }
}
pub struct GltfFramePrimitiveState {
    node_instance_map: HashMap<usize, [[f32; 4]; 4]>,
    instances: Vec<[[f32; 4]; 4]>,
    instance_buffer: Option<wgpu::Buffer>,
    num_instances: u32,
}

pub struct GltfFrameState<'a> {
    pub meshes: Vec<Vec<GltfFramePrimitiveState>>,
    pub gltf_file: &'a GltfFile,
    global_transform: cgmath::Matrix4<f32>,
}

impl<'a> GltfFrameState<'a> {
    pub fn new(gltf_file: &'a GltfFile) -> Self {
        let mut meshes = Vec::new();
        for mesh in gltf_file.meshes.iter() {
            let mut primitives = Vec::new();
            for primitive in mesh.primitives.iter() {
                let mut node_instance_map = HashMap::new();
                for instance in primitive.instances.iter() {
                    let node_idx = instance.parent_node;
                    node_instance_map.insert(node_idx, instance.transform.into());
                }
                let frame_state = GltfFramePrimitiveState {
                    node_instance_map,
                    instances: Vec::new(),
                    instance_buffer: None,
                    num_instances: 0,
                };
                primitives.push(frame_state);
            }
            meshes.push(primitives);
        }
        Self {
            meshes,
            gltf_file,
            global_transform: cgmath::Matrix4::one(),
        }
    }
    fn update_instance(
        &mut self,
        transform: cgmath::Matrix4<f32>,
        mesh_idx: usize,
        node_idx: usize,
    ) {
        for primitive in self.meshes[mesh_idx].iter_mut() {
            let prev_transform = primitive
                .node_instance_map
                .get(&node_idx)
                .expect("Unable to apply transform: Node does not exist");
            let new_transform = transform * cgmath::Matrix4::from(*prev_transform);
            primitive
                .node_instance_map
                .insert(node_idx, new_transform.into());
        }
    }

    fn add_node_transform(&mut self, transform: cgmath::Matrix4<f32>, node_idx: usize) {
        let node = &self.gltf_file.document.nodes().collect::<Vec<_>>()[node_idx];
        if let Some(mesh) = node.mesh() {
            self.update_instance(transform, mesh.index(), node.index());
        } else {
            println!("Transform does not affect mesh");
        }
    }

    pub fn set_animation(&mut self, animation: &AnimationState) {
        let animations = self.gltf_file.document.animations().collect::<Vec<_>>();
        let gltf_animation = animations
            .get(animation.index)
            .expect("Invalid animation index");
        for channel in gltf_animation.channels() {
            let sampler = channel.sampler();
            let target = channel.target();
            let node_idx = target.node().index();
            let reader = channel.reader(|buffer| Some(&self.gltf_file.buffers[buffer.index()]));
            let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>();
            let rotations = match reader.read_outputs().unwrap() {
                gltf::animation::util::ReadOutputs::Rotations(rots) => {
                 rots
                        .into_f32()
                        .map(|r| cgmath::Quaternion::from(r))
                        .collect::<Vec<_>>()
                },
                _ => vec![],
            };
            let translations = match reader.read_outputs().unwrap() {
                gltf::animation::util::ReadOutputs::Translations(trans) => {
                    trans
                        .map(|t| cgmath::Vector3::from(t))
                        .collect::<Vec<_>>()
                }
                _ => vec![],
            };

            if inputs.iter().len() > 0 {
                let elapsed = (animation.current_time - animation.start_time).as_secs_f32() % inputs.last().unwrap();
                let rhs_index = inputs.iter().position(|&el| el >= elapsed).unwrap();
                let lhs_index = if rhs_index > 0 { rhs_index - 1 } else { 0 };
                let rhs_timestep = inputs[rhs_index];
                let lhs_timestep = inputs[lhs_index];
                let slerp_factor = (elapsed - lhs_timestep) / (rhs_timestep - lhs_timestep);

                if target.property() == gltf::animation::Property::Rotation {
                    let transform = cgmath::Matrix4::from(rotations[lhs_index].slerp(rotations[rhs_index], slerp_factor));
                    self.add_node_transform(transform, node_idx)
                }
                if target.property() == gltf::animation::Property::Translation {
                    let lhs = translations[lhs_index];
                    let rhs = translations[rhs_index];
                    let transform = lhs.lerp(rhs, slerp_factor);
                    self.add_node_transform(cgmath::Matrix4::from_translation(transform), node_idx)
                }


            }



        }
    }

    pub fn set_global_transform(&mut self, transform: cgmath::Matrix4<f32>) {
        self.global_transform = transform;
    }

    pub fn init_buffers(&mut self, device: &wgpu::Device) {
        for mesh in self.meshes.iter_mut() {
            for primitive in mesh.iter_mut() {
                primitive.instances = primitive
                    .node_instance_map
                    .iter()
                    .map(|(_, p)| (self.global_transform * cgmath::Matrix4::from(p.clone())).into())
                    .collect();
                primitive.instance_buffer = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&primitive.instances),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }
        }
    }
}

pub struct GltfMesh {
    pub primitives: Vec<GltfPrimitive>,
}

#[repr(C)]
#[derive(bytemuck::Pod, Copy, Clone, bytemuck::Zeroable)]
pub struct Instance {
    parent_node: usize,
    transform: [[f32; 4]; 4],
}

pub struct GltfPrimitive {
    pub signature: Signature,
    pub vertex_buffers: Vec<wgpu::Buffer>,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub instances: Vec<Instance>,
}

pub struct GltfFile {
    pub path: String,
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
    pub render_pipelines: HashMap<Signature, wgpu::RenderPipeline>,
    pub meshes: Vec<GltfMesh>,
}
impl GltfFile {
    pub fn new(path: &str, renderer: &Renderer) -> Self {
        
        let file = std::fs::File::open(path);
        let (document, buffers, images) = gltf::import(path).unwrap();

        Self {
            path: String::from(path),
            render_pipelines: Self::build_pipelines(&document, renderer),
            meshes: Self::build_meshes(&document, &buffers, renderer),
            document,
            buffers,
            images,
        }
    }
    fn sign_primitive(primitive: &gltf::Primitive) -> Signature {
        let mut signature = String::new();
        primitive.attributes().for_each(|(name, _)| {
            signature.push_str(&name.to_string());
        });
        signature
    }
    fn build_meshes(
        document: &gltf::Document,
        buffers: &[gltf::buffer::Data],
        renderer: &Renderer,
    ) -> Vec<GltfMesh> {
        let mut meshes = Vec::new();
        for mesh in document.meshes() {
            let mut primitives = Vec::new();
            for primitive in mesh.primitives() {
                let primitive = Self::build_primitive(primitive, buffers, renderer);
                primitives.push(primitive);
            }
            meshes.push(GltfMesh { primitives });
        }
        let mut nodes = Vec::new();
        let mut transforms = Vec::new();
        for node in document.scenes().next().unwrap().nodes() {
            transforms.push(cgmath::Matrix4::from(node.transform().matrix()));
            nodes.push(node);
        }
        // Traverse scene graph in order to populate transforms
        while !nodes.is_empty() {
            let node = nodes.pop().unwrap();
            let transform = transforms.pop().unwrap();
            if let Some(mesh) = node.mesh() {
                let mesh = &mut meshes[mesh.index()];
                for primitive in mesh.primitives.iter_mut() {
                    primitive.instances.push(Instance {
                        parent_node: node.index(),
                        transform: transform.into(),
                    });
                }
            }
            for child in node.children() {
                transforms.push(cgmath::Matrix4::from(child.transform().matrix()) * transform);
                nodes.push(child);
            }
        }
        meshes
    }
    fn build_primitive(
        primitive: gltf::Primitive,
        buffers: &[gltf::buffer::Data],
        renderer: &Renderer,
    ) -> GltfPrimitive {
        let mut vertex_buffers = Vec::new();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions = reader.read_positions().unwrap().collect::<Vec<[f32; 3]>>();
        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&positions),
                usage: wgpu::BufferUsages::VERTEX,
            });
        vertex_buffers.push(buffer);
        let normals = match reader.read_normals() {
            Some(normals) => normals.collect::<Vec<[f32; 3]>>(),
            None => vec!([0.0, 0.0, 0.0])
        };
        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&normals),
                usage: wgpu::BufferUsages::VERTEX,
            });
        vertex_buffers.push(buffer);
        let (num_indices, index_buffer) = if let Some(indices) = reader.read_indices() {
            let indices: Vec<u32> = indices.into_u32().collect();
            (
                indices.len() as u32,
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            )
        } else {
            panic!("No indices");
        };
        GltfPrimitive {
            vertex_buffers,
            index_buffer,
            signature: Self::sign_primitive(&primitive),
            num_indices,
            instances: Vec::new(),
        }
    }
    fn build_pipelines(
        document: &gltf::Document,
        renderer: &Renderer,
    ) -> HashMap<Signature, wgpu::RenderPipeline> {
        let mut render_pipelines = HashMap::new();
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let signature = Self::sign_primitive(&primitive);
                if !render_pipelines.contains_key(&signature) {
                    let pipeline = Self::build_pipeline(primitive, renderer);
                    render_pipelines.insert(signature, pipeline);
                }
            }
        }
        render_pipelines
    }
    fn build_pipeline(primitive: gltf::Primitive, renderer: &Renderer) -> wgpu::RenderPipeline {
        let positions = primitive
            .get(&gltf::Semantic::Positions)
            .unwrap_or_else(|| {
                panic!("No positions for primitive");
            });
        let positions_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: positions.size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![4 => Float32x3],
        };
        let normals = match primitive.get(&gltf::Semantic::Normals) {
            Some(accessor) => accessor.size(),
            None => {
                println!("Using default normals");
                std::mem::size_of::<[f32;3]>()
            }
        };
        let normals_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: normals as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![5 => Float32x3],
        };
        let instances_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
        };
        let targets = &[Some(wgpu::ColorTargetState::from(
            renderer.surface_config.format,
        ))];
        let fragment = wgpu::FragmentState {
            module: &renderer.default_pipeline.pn_shader,
            entry_point: "fs_main",
            targets,
        };
        let vertex = wgpu::VertexState {
            module: &renderer.default_pipeline.pn_shader,
            entry_point: "vs_main",
            buffers: &[
                instances_buffer_layout,
                positions_buffer_layout,
                normals_buffer_layout,
            ],
        };
        renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&renderer.default_pipeline.layout),
                vertex,
                fragment: Some(fragment),
                primitive: renderer.default_pipeline.primitive,
                depth_stencil: Some(renderer.default_pipeline.depth_stencil.clone()),
                multisample: renderer.default_pipeline.multisample,
                multiview: renderer.default_pipeline.multiview,
            })
    }
}
