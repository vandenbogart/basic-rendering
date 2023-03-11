use crate::renderer::Vertex;

pub mod obj;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub name: String,
}
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
    pub name: String,
    pub material_id: Option<usize>,
}
pub struct Material {
    pub name: String,
}
