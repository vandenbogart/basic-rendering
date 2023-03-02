
use cgmath::prelude::*;

use crate::{Component, World};

pub mod camera;
pub mod input;
pub mod movement;

pub trait System {
    fn run(&mut self, world: &mut World, dt: f32);
}
#[derive(Copy, Clone)]
pub enum ClickMoveState {
    Initial,
    Move,
    Waiting,
}
pub struct ClickMoveComponent {
    pub last_click_pos: Option<cgmath::Point2<f32>>,
    pub move_coord: Option<cgmath::Point3<f32>>,
    pub speed: f32,
    pub state: ClickMoveState,
}
impl Component for ClickMoveComponent {}
impl ClickMoveComponent {
    pub fn new(speed: f32) -> Self {
        Self { last_click_pos: None, move_coord: None, speed, state: ClickMoveState::Initial }
    }
    pub fn move_towards(&self, current_position: cgmath::Point3<f32>, dt: f32) -> Option<cgmath::Point3<f32>> {
        match (self.state, self.move_coord) {
            (ClickMoveState::Waiting, Some(coord)) => {
                let distance = coord - current_position;
                let direction = distance.normalize();
                if distance.magnitude() > 1.0 {
                    Some(current_position + (dt * self.speed * direction))
                }
                else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct WalkableComponent {

}
impl Component for WalkableComponent {}