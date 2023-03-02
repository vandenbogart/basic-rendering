use cgmath::prelude::*;

use crate::{renderer::{GeometryComponent, ModelComponent, ModelResource}, ray::Ray};

use super::{ClickMoveComponent, System};

pub struct MovementSystem {}
impl MovementSystem {
    pub fn new() -> Self {
        Self::default()
    }
}
impl System for MovementSystem {
    fn run(&mut self, world: &mut crate::World, dt: f32) {
        let result = world
            .query()
            .with_component::<ClickMoveComponent>()
            .with_component::<GeometryComponent>()
            .execute();
        result.get_entities().iter().for_each(|ent| {
            let move_comp = result.get_component::<ClickMoveComponent>(*ent);
            let mut geo = result.get_component_mut::<GeometryComponent>(*ent);
            match move_comp.move_towards(geo.position, dt) {
                Some(pos) => {
                    drop(geo);
                    let mut new_pos = pos.clone();
                    new_pos.y += 10.0;
                    let ray = Ray::new(new_pos, -1.0 * cgmath::Vector3::<f32>::unit_y());
                    let hits = ray.test(world);
                    if let Some(hit) = hits.get(0) {
                        new_pos.y = hit.position.y + 1.0
                    }
                    let mut geo = result.get_component_mut::<GeometryComponent>(*ent);
                    geo.position = new_pos;
                },
                None => (),
            }
        })
    }
}
impl Default for MovementSystem {
    fn default() -> Self {
        Self {}
    }
}
