use std::slice::Iter;

use cgmath::prelude::*;

use crate::{
    asset_manager::AssetManager,
    component_manager::ComponentManager,
    components::{transform::Transform, walk_to::WalkTo, walkable_surface::WalkableSurface, click_move::ClickMove},
    ray::Ray,
    world::World, EntityHandle,
};

use super::System;

pub struct MovementSystem {

}
impl MovementSystem {
    pub fn new() -> Self {
        Self::default()
    }
}
impl System for MovementSystem {
    fn run(&mut self, world: &mut World, cm: &mut ComponentManager, am: &AssetManager, dt: f32) {
        let entities = cm.get_all_by_type::<WalkTo>();
        entities.iter().for_each(|(ent, walk_to)| {
            let mut transform = cm
                .mut_component::<Transform>(*ent)
                .unwrap_or_else(|| panic!("Entity {:?} does not have a transform component", ent));
            if let Some(target) = walk_to.target {
                let walkable_surfaces: Vec<EntityHandle> = cm.get_all_by_type::<WalkableSurface>().into_iter().map(|(ent, _)| ent).collect();
                let ray = Ray::new(
                    cgmath::point3(transform.position.x, transform.position.y + 20.0, transform.position.z),
                    -1.0 as f32 * cgmath::Vector3::unit_y(),
                );
                let hits = ray.test(walkable_surfaces.as_slice(), cm, am);
                let floor_pos = hits.get(0).unwrap_or_else(|| {
                    panic!("No walkable surfaces found for entity {:?}", ent)
                }).position;

                let new_y = floor_pos.y + 1.0;
                let new_pos = (target - transform.position).normalize() * walk_to.speed * dt;
                transform.position += new_pos;
                transform.position.y = new_y;
            }
        });

        let entities = cm.get_all_by_type::<ClickMove>();
        entities.iter().for_each(|(ent, click_move)| {
            let mut transform = cm
                .mut_component::<Transform>(*ent)
                .unwrap_or_else(|| panic!("Entity {:?} does not have a transform component", ent));
            if let Some(target) = click_move.target {
                let walkable_surfaces: Vec<EntityHandle> = cm.get_all_by_type::<WalkableSurface>().into_iter().map(|(ent, _)| ent).collect();
                let ray = Ray::new(
                    cgmath::point3(transform.position.x, transform.position.y + 20.0, transform.position.z),
                    -1.0 as f32 * cgmath::Vector3::unit_y(),
                );
                let hits = ray.test(walkable_surfaces.as_slice(), cm, am);
                let floor_pos = hits.get(0).unwrap_or_else(|| {
                    panic!("No walkable surfaces found for entity {:?}", ent)
                }).position;

                let new_y = floor_pos.y + 1.0;
                let new_pos = (target - transform.position).normalize() * click_move.speed * dt;
                transform.position += new_pos;
                transform.position.y = new_y;
            }
        });
    }
}
impl Default for MovementSystem {
    fn default() -> Self {
        Self {}
    }
}
