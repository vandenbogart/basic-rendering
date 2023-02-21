use std::time::Duration;

use crate::{Component, World};

pub mod camera;
pub mod input;
pub mod movement;

pub trait System {
    fn run(&mut self, world: &mut World, dt: Duration);
}

pub struct VelocityComponent {
    velocity: f32,
}
impl Component for VelocityComponent {}

pub struct WASDControllerComponent {
    pub w: i32,
    pub a: i32,
    pub s: i32,
    pub d: i32,
    pub speed: i32,
}
impl Default for WASDControllerComponent {
    fn default() -> Self {
        Self {
            w: Default::default(),
            a: Default::default(),
            s: Default::default(),
            d: Default::default(),
            speed: 10,
        }
    }
}
impl Component for WASDControllerComponent {}
