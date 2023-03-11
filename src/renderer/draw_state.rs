use std::{collections::HashMap, cell::Ref};

use wgpu::util::DeviceExt;

use crate::{
    asset_manager::{Asset, AssetHandle},
    components::transform::Transform,
    loaders::Model,
};

use super::{Globals, Instance, InstanceRaw};

pub struct Drawable {
    pub asset_handle: AssetHandle,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub num_indices: u32,
    pub instance_buf: wgpu::Buffer,
    pub num_instances: u32,
}

pub struct DrawStateBuilder<'a> {
    device: &'a wgpu::Device,
    asset_transform_map: HashMap<AssetHandle, Vec<Ref<'a, Transform>>>,
    assets: Vec<&'a Asset<Model>>,
    view_proj: Option<cgmath::Matrix4<f32>>,
}
impl<'a> DrawStateBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            asset_transform_map: HashMap::new(),
            assets: Vec::new(),
            view_proj: None
        }
    }
    pub fn add_model_instance(&mut self, model: &'a Asset<Model>, transform: Ref<'a, Transform>) {
        if let Some(transforms) = self.asset_transform_map.get_mut(&model.asset_handle) {
            transforms.push(transform);
        } else {
            self.asset_transform_map
                .insert(model.asset_handle, vec![transform]);
        }
        self.assets.push(model);
    }
    pub fn set_view_proj(&mut self, view_proj: cgmath::Matrix4<f32>) {
        self.view_proj = Some(view_proj);
    }
    pub fn build(&self) -> DrawState {
        let mut drawables = Vec::new();
        let mut asset_instance_map = HashMap::<AssetHandle, Vec<InstanceRaw>>::new();
        self.assets.iter().for_each(|asset| {
            if let Some(transforms) = self.asset_transform_map.get(&asset.asset_handle) {
                // Collect all instances of this asset
                let instances = transforms
                    .iter()
                    .map(|transform| {
                        Instance {
                            position: transform.position.into(),
                            rotation: transform.rotation.into(),
                        }
                        .to_raw()
                    })
                    .collect();
                asset_instance_map.insert(asset.asset_handle, instances);
                let instances = asset_instance_map.get(&asset.asset_handle).unwrap();

                // Collect all buffers
                let mut mesh_drawables: Vec<Drawable> = asset
                    .asset
                    .meshes
                    .iter()
                    .map(|mesh| {
                        let vertex_buf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(&mesh.vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                });
                        let index_buf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(&mesh.indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                });
                        let num_indices = mesh.num_indices;
                        let instance_buf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(instances),
                                    usage: wgpu::BufferUsages::VERTEX,
                                });
                        let num_instances = instances.len() as u32;
                        let asset_handle = asset.asset_handle;
                        Drawable {
                            asset_handle,
                            vertex_buf,
                            index_buf,
                            num_indices,
                            instance_buf,
                            num_instances,
                        }
                    })
                    .collect();
                drawables.append(&mut mesh_drawables);
            } else {
                panic!("Attempted to draw asset with no instance");
            }
        });
        DrawState {
            drawables,
            asset_instance_map,
            globals: Globals {
                view_proj: self.view_proj.unwrap().into(),
                ambient_color: [1.0, 1.0, 1.0, 1.0],
                ambient_strength: [0.2, 0.0, 0.0, 0.0],
            },
        }
    }
    pub fn reset(&mut self) {
        self.asset_transform_map = HashMap::new();
        self.assets = Vec::new();
    }
}
pub struct DrawState {
    pub drawables: Vec<Drawable>,
    pub asset_instance_map: HashMap<AssetHandle, Vec<InstanceRaw>>,
    pub globals: Globals,
}
