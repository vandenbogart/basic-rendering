use crate::renderer::Vertex;

use self::gltf::GltfFile;

pub mod obj;
pub mod gltf;

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

impl From<GltfFile> for Model {
    fn from(gltf: GltfFile) -> Self {
        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        for mesh in gltf.document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&gltf.buffers[buffer.index()]));

                let positions = reader.read_positions().unwrap();
                let normals = reader.read_normals().unwrap();
                let colors = if let Some(colors) = reader.read_colors(0) { colors.into_rgb_f32().collect() } else {
                    vec![[0.0, 0.0, 0.0]; positions.len()]
                };
                let vertices: Vec<Vertex> = positions
                    .zip(normals)
                    .zip(colors)
                    .map(|((position, normal), color)| Vertex {
                        position: position.into(),
                        normals: normal.into(),
                        color: color.into(),
                    })
                    .collect();
                let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
            }
        }
        Self {
            meshes,
            materials,
            name: gltf.path,
        }
    }


}
