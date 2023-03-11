use::cgmath::prelude::*;

use super::Component;

pub struct Transform {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub forward: cgmath::Vector3<f32>,
}
impl Transform {
    pub fn new(
        position: Option<cgmath::Point3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        forward: Option<cgmath::Vector3<f32>>,
    ) -> Self {
        let position = match position {
            Some(pos) => pos,
            None => cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let rotation = match rotation {
            Some(rot) => rot,
            None => cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0)),
        };
        let forward = match forward {
            Some(vec) => vec,
            None => cgmath::Vector3::unit_x(),
        };
        Transform {
            position,
            rotation,
            forward,
        }
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0)),
            forward: cgmath::Vector3::unit_x(),
        }
    }
}
impl Component for Transform {}
