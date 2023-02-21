use std::{path::Path, cell::Ref};

use cgmath::{Rotation3, Vector3};
use wgpu::{vertex_attr_array, Texture};

use crate::{Component, Resource, ResourceIndex};


mod context;
pub mod render;
mod pipeline_default;
mod loaders;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
    ambient_color: [f32; 3],
    ambient_strength: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Locals {
    pub position: [f32; 4],
    pub diffuse_light_pos: [f32; 4],
    pub diffuse_light_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normals: [f32; 3],
    pub color: [f32; 3],
}

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3];
impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceRaw {
    pub pr_matrix: [[f32; 4]; 4],
    pub n_matrix: [[f32; 3]; 3],
}

const INSTANCE_ATTRIBUTES: [wgpu::VertexAttribute; 7] = vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x3, 9 => Float32x3, 10 => Float32x3];
impl Instance {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &INSTANCE_ATTRIBUTES,
        }
    }
    pub fn to_raw(&self) -> InstanceRaw {
        let prmat =
            cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
        let nmat = cgmath::Matrix3::from(self.rotation);
        InstanceRaw {
            pr_matrix: prmat.into(),
            n_matrix: nmat.into(),
        }
    }
}
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub name: String,
}
pub struct Mesh {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub num_indices: u32,
    pub name: String,
    pub material_id: Option<usize>,
}

pub struct ModelComponent {
    model_index: ResourceIndex,
}
impl ModelComponent {
    pub fn new(model_index: ResourceIndex) -> ModelComponent {
        ModelComponent {
            model_index,
        }
    }
}
impl Component for ModelComponent {}

pub struct CameraFollowComponent {}
impl Component for CameraFollowComponent {}
pub struct GeometryComponent {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub forward: cgmath::Vector3<f32>,
}
impl GeometryComponent {
    pub fn new(position: Option<cgmath::Vector3<f32>>, rotation: Option<cgmath::Quaternion<f32>>, forward: Option<cgmath::Vector3<f32>>) -> Self {
        let position = match position {
            Some(pos) => pos,
            None => cgmath::Vector3::new(0.0, 0.0, 0.0),
        };
        let rotation = match rotation {
            Some(rot) => rot,
            None => cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0)),
        };
        let forward = match forward {
            Some(vec) => vec,
            None => cgmath::Vector3::unit_x(),
        };
        GeometryComponent {
            position,
            rotation,
            forward,
        }
    }
}
impl Default for GeometryComponent {
    fn default() -> Self {
        Self { position: cgmath::Vector3::new(0.0, 0.0, 0.0), rotation: cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0)), forward: Vector3::unit_x() }
    }
}
impl Component for GeometryComponent {}

pub struct ModelResource {
    model: Model
}
impl ModelResource {
    pub fn new(model: Model) -> ModelResource{
        ModelResource {
            model,
        }
    }
}
impl Resource for ModelResource {

}


pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
}