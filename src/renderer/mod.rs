pub mod draw_state;
mod pipeline_default;
pub mod render;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
    pub ambient_color: [f32; 4],
    pub ambient_strength: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct Locals {
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
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3];
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
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceRaw {
    pub pr_matrix: [[f32; 4]; 4],
    pub n_matrix: [[f32; 3]; 3],
}

const INSTANCE_ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x3, 9 => Float32x3, 10 => Float32x3];
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
            cgmath::Matrix4::from_translation(self.position - cgmath::point3(0.0, 0.0, 0.0))
                * cgmath::Matrix4::from(self.rotation);

        let nmat = cgmath::Matrix3::from(self.rotation);
        InstanceRaw {
            pr_matrix: prmat.into(),
            n_matrix: nmat.into(),
        }
    }
}

