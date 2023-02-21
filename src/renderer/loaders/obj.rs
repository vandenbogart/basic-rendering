use std::fs::File;
use std::io;
use std::path::Path;
use std::{fs, io::BufReader};

use wgpu::{Device, Queue, Texture};

use super::super::{Material, Mesh, Model, Vertex};

pub async fn load_texture(
    queue: &Queue,
    device: &Device,
    filename: &str,
) -> anyhow::Result<Texture> {
    let path = Path::new("./assets/textures/").join(filename);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let image = image::load(reader, image::ImageFormat::Png)?; // support other types
    let rgba = image.to_rgba8();

    use image::GenericImageView;
    let dimensions = image.dimensions();
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        // All textures are stored as 3D, we represent our 2D texture
        // by setting depth to 1.
        size: texture_size,
        mip_level_count: 1, // We'll talk about this a little later
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Most images are stored using sRGB so we need to reflect that here.
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        // COPY_DST means that we want to copy data to this texture
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some(filename),
        // This is the same as with the SurfaceConfig. It
        // specifies what texture formats can be used to
        // create TextureViews for this texture. The base
        // texture format (Rgba8UnormSrgb in this case) is
        // always supported. Note that using a different
        // texture format is not supported on the WebGL2
        // backend.
        view_formats: &[],
    });
    queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &rgba,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
            rows_per_image: std::num::NonZeroU32::new(dimensions.1),
        },
        texture_size,
    );
    Ok(texture)
}

pub async fn load_model(
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    path: &std::path::Path,
) -> anyhow::Result<Model> {
    let file = fs::File::open(&path)?;
    let mut model_file = io::BufReader::new(file);
    let (models, obj_mats) = tobj::load_obj_buf(
        &mut model_file,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let p2 = std::path::Path::new("./assets/").join(p);
            let mut mat = io::BufReader::new(fs::File::open(p2).unwrap());
            tobj::load_mtl_buf(&mut mat)
        },
    )?;
    let mut materials = Vec::new();
    for mat in &obj_mats? {
        let mut new_mat = Material {
            name: mat.name.to_string(),
            diffuse_texture: None,
        };
        dbg!(&mat.diffuse_texture);
        if mat.diffuse_texture.len() > 0 {
            let texture = load_texture(queue, device, &mat.diffuse_texture).await?;
            new_mat.diffuse_texture = Some(texture);
        }
        materials.push(new_mat);
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| Vertex {
                    position: [
                        m.mesh.positions[i * 3 + 0],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: if m.mesh.texcoords.len() > 0 {
                        [
                            m.mesh.texcoords[i * 2 + 0],
                            1.0 - m.mesh.texcoords[i * 2 + 1],
                        ]
                    } else {
                        [0.0, 0.0]
                    },
                    normals: [
                        m.mesh.normals[i * 3 + 0],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    color: [0.0, 0.0, 0.0],
                })
                .collect::<Vec<_>>();
            use wgpu::util::DeviceExt;

            let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            Mesh {
                vertex_buf,
                index_buf,
                name: m.name,
                num_indices: m.mesh.indices.len() as u32,
                material_id: m.mesh.material_id,
            }
        })
        .collect::<Vec<_>>();

    Ok(Model {
        meshes,
        materials,
        name: String::from(path.to_str().unwrap()),
    })
}
