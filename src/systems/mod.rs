use crate::world::World;


pub mod camera;

pub trait System {
    fn run(&mut self, world: &mut World, dt: f32);
}
