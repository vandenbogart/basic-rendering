use std::{f32::consts::PI, time::Duration};

use cgmath::Rotation3;

use crate::renderer::GeometryComponent;

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
                    geo.position = pos
                }
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
