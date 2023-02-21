use std::{f32::consts::PI, time::Duration};

use cgmath::Rotation3;

use crate::renderer::GeometryComponent;

use super::{System, WASDControllerComponent};

pub struct MovementSystem {}
impl MovementSystem {
    pub fn new() -> Self {
        Self::default()
    }
}
impl System for MovementSystem {
    fn run(&mut self, world: &mut crate::World, dt: Duration) {
        let result = world
            .query()
            .with_component::<WASDControllerComponent>()
            .with_component::<GeometryComponent>()
            .execute();
        result.get_entities().iter().for_each(|ent| {
            let wasd = result.get_component::<WASDControllerComponent>(*ent);
            let mut geo = result.get_component_mut::<GeometryComponent>(*ent);
            if wasd.d == 1 && wasd.a == 0 {
                geo.rotation = geo.rotation
                    * cgmath::Quaternion::from_angle_y(cgmath::Rad(PI * dt.as_secs_f32()));
            }
            if wasd.a == 1 && wasd.d == 0 {
                geo.rotation = geo.rotation
                    * cgmath::Quaternion::from_angle_y(cgmath::Rad(-PI * dt.as_secs_f32()));
            }
            if wasd.w == 1 {
                let direction = geo.rotation * geo.forward;
                geo.position = geo.position + (direction * wasd.speed as f32 * dt.as_secs_f32());
            }
        })
    }
}
impl Default for MovementSystem {
    fn default() -> Self {
        Self {}
    }
}
