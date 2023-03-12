use crate::EntityHandle;

use super::Component;

pub struct Click {
    pub screen_y: f32,
    pub screen_x: f32,
    pub world_pos: cgmath::Point3<f32>,
    pub target: Option<EntityHandle>,
}
impl Component for Click {}

