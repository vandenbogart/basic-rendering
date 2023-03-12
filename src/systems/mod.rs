use crate::{asset_manager::AssetManager, component_manager::ComponentManager, world::World};

pub mod camera;
pub mod movement;
pub mod click;

pub trait System {
    fn run(&mut self, world: &mut World, cm: &mut ComponentManager, am: &AssetManager, dt: f32);
}
