use crate::{
    asset_manager::AssetManager,
    component_manager::ComponentManager,
    components::{model::Model, transform::Transform},
    loaders, EntityHandle,
};
use cgmath::prelude::*;

pub struct RayHit {
    pub entity: EntityHandle,
    pub position: cgmath::Point3<f32>,
}
impl RayHit {
    pub fn new(entity: EntityHandle, position: cgmath::Point3<f32>) -> Self {
        Self { entity, position }
    }
}
struct Triangle(
    cgmath::Point3<f32>,
    cgmath::Point3<f32>,
    cgmath::Point3<f32>,
);
pub struct Ray {
    position: cgmath::Point3<f32>,
    direction: cgmath::Vector3<f32>,
}
impl Ray {
    pub fn new(position: cgmath::Point3<f32>, direction: cgmath::Vector3<f32>) -> Self {
        Self {
            position,
            direction,
        }
    }
    pub fn test(
        &self,
        entities: &[EntityHandle],
        cm: &ComponentManager,
        am: &AssetManager,
    ) -> Vec<RayHit> {
        let mut intersection_points = Vec::new();
        for ent in entities {
            let model = cm
                .get_component::<Model>(*ent)
                .unwrap_or_else(|| panic!("Entity {:?} does not have a model component", ent));
            let transform = cm
                .get_component::<Transform>(*ent)
                .unwrap_or_else(|| panic!("Entity {:?} does not have a transform component", ent));
            let asset = am
                .get_asset::<loaders::Model>(model.asset_handle)
                .unwrap_or_else(|| panic!("Asset {:?} does not exist", model.asset_handle));

            for mesh in asset.asset.meshes.iter() {
                let vertices = &mesh.vertices;
                for chunk in mesh.indices.chunks(3) {
                    let transform = transform.to_matrix();
                    let (v1, v2, v3) = (chunk[0], chunk[1], chunk[2]);
                    let (v1, v2, v3) = (
                        vertices[v1 as usize],
                        vertices[v2 as usize],
                        vertices[v3 as usize],
                    );
                    let triangle = Triangle(
                        Ray::apply_transform(v1.position.into(), transform),
                        Ray::apply_transform(v2.position.into(), transform),
                        Ray::apply_transform(v3.position.into(), transform),
                    );
                    if let Some(p) = self.intersect(&triangle) {
                        intersection_points.push(RayHit::new(*ent, p));
                    }
                }
            }
        }
        intersection_points.sort_unstable_by(|a, b| {
            let a = self.position.distance(a.position);
            let b = self.position.distance(b.position);
            a.partial_cmp(&b).expect("Failed to order")
        });
        intersection_points
    }
    fn apply_transform(v: cgmath::Point3<f32>, t: cgmath::Matrix4<f32>) -> cgmath::Point3<f32> {
        let v = cgmath::vec4(v.x, v.y, v.z, 1.0);
        let v = t * v;
        cgmath::point3(v.x, v.y, v.z)
    }
    fn intersect(&self, t: &Triangle) -> Option<cgmath::Point3<f32>> {
        // Calcuate plane intersection point
        let face_norm = (t.1 - t.0).cross(t.2 - t.1);
        let L0 = self.position;
        let L = self.direction.normalize();
        let n = face_norm;
        let n0 = t.0;
        // Cull backface triangles
        if L.dot(n) >= 0.0 {
            return None;
        }
        // Cull behind plane
        if (n0 - L0).dot(L) <= 0.0 {
            return None;
        }
        // Find intersection, may be undefined
        let x = cgmath::dot(n0 - L0, n) / cgmath::dot(L, n);
        if x.is_finite() {
            let x = L0 + x * L;
            // edge 1
            let e1 = t.1 - t.0;
            let x1 = x - t.0;
            if e1.cross(x1).dot(n) < 0.0 {
                return None;
            }
            // edge 2
            let e2 = t.2 - t.1;
            let x2 = x - t.1;
            if e2.cross(x2).dot(n) < 0.0 {
                return None;
            }
            // edge 3
            let e3 = t.0 - t.2;
            let x3 = x - t.2;
            if e3.cross(x3).dot(n) < 0.0 {
                return None;
            }
            return Some(x);
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_hit() {
        let ray = Ray::new(cgmath::point3(0.0, 5.0, 0.0), cgmath::vec3(0.0, 0.0, 1.0));
        let triangle = Triangle(
            cgmath::point3(0.0, 6.0, 1.0),
            cgmath::point3(1.0, 4.0, 1.0),
            cgmath::point3(-1.0, 4.0, 1.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, Some(cgmath::point3(0.0, 5.0, 1.0)))
    }
    #[test]
    fn test_ray_miss() {
        let ray = Ray::new(cgmath::point3(0.0, 10.0, 0.0), cgmath::vec3(0.0, 0.0, 1.0));
        let triangle = Triangle(
            cgmath::point3(0.0, 6.0, 1.0),
            cgmath::point3(1.0, 4.0, 1.0),
            cgmath::point3(-1.0, 4.0, 1.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, None)
    }

    #[test]
    fn test_ray_miss_2() {
        let ray = Ray::new(cgmath::point3(0.0, 0.0, 0.0), cgmath::vec3(-1.0, 0.0, 0.0));
        let triangle = Triangle(
            cgmath::point3(-2.0, 6.0, 1.0),
            cgmath::point3(-1.5, -2.0, 1.5),
            cgmath::point3(-1.0, 4.0, 1.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, None)
    }

    #[test]
    fn test_ray_miss_3() {
        let ray = Ray::new(cgmath::point3(0.0, 0.0, 0.0), cgmath::vec3(1.0, 0.0, 0.0));
        let triangle = Triangle(
            cgmath::point3(-2.0, 6.0, 1.0),
            cgmath::point3(-1.5, -2.0, -1.5),
            cgmath::point3(-1.0, 4.0, 4.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, None)
    }

    #[test]
    fn test_ray_cull_backface() {
        let ray = Ray::new(cgmath::point3(0.0, 0.0, 0.0), cgmath::vec3(0.0, 1.0, 0.0));
        let triangle = Triangle(
            cgmath::point3(0.0, 2.0, 1.0),
            cgmath::point3(1.0, 2.0, 0.0),
            cgmath::point3(-1.0, 2.0, -1.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, None)
    }

    #[test]
    fn test_ray_cull_behind() {
        let ray = Ray::new(cgmath::point3(0.0, 0.0, 0.0), cgmath::vec3(0.0, 1.0, 0.0));
        let triangle = Triangle(
            cgmath::point3(1.0, -2.0, 0.0),
            cgmath::point3(0.0, -2.0, 1.0),
            cgmath::point3(-1.0, -2.0, -1.0),
        );
        let result = ray.intersect(&triangle);
        assert_eq!(result, None)
    }

    #[test]
    fn test_transform() {
        let p = cgmath::point3(0.0, 1.0, 0.0);
        let geo = Transform::new(
            Some(cgmath::point3(5.0, 5.0, 5.0)),
            Some(cgmath::Quaternion::from_angle_x(cgmath::Deg(90.0))),
            None,
        );

        let result = Ray::apply_transform(p, geo.to_matrix());
        assert_eq!(result.distance(cgmath::point3(5.0, 5.0, 6.0)) < 0.1, true)
    }
}
