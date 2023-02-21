use std::time::Duration;

use winit::event::{ElementState, VirtualKeyCode};

use super::{System, WASDControllerComponent};

pub struct InputSystem {
    keys: Vec<(VirtualKeyCode, ElementState)>,
}
impl InputSystem {
    pub fn new() -> Self {
        InputSystem::default()
    }
    pub fn process_keyboard(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        self.keys.push((keycode, state));
    }
}
impl Default for InputSystem {
    fn default() -> Self {
        Self {
            keys: Default::default(),
        }
    }
}
impl System for InputSystem {
    fn run(&mut self, world: &mut crate::World, _dt: Duration) {
        for key in &self.keys {
            match key {
                (
                    VirtualKeyCode::W | VirtualKeyCode::A | VirtualKeyCode::S | VirtualKeyCode::D,
                    ElementState::Pressed | ElementState::Released,
                ) => {
                    let result = world
                        .query()
                        .with_component::<WASDControllerComponent>()
                        .execute();
                    result.get_entities().iter().for_each(|ent| {
                        let mut comp = result.get_component_mut::<WASDControllerComponent>(*ent);
                        match key {
                            (VirtualKeyCode::W, ElementState::Pressed) => comp.w = 1,
                            (VirtualKeyCode::W, ElementState::Released) => comp.w = 0,
                            (VirtualKeyCode::A, ElementState::Pressed) => comp.a = 1,
                            (VirtualKeyCode::A, ElementState::Released) => comp.a = 0,
                            (VirtualKeyCode::S, ElementState::Pressed) => comp.s = 1,
                            (VirtualKeyCode::S, ElementState::Released) => comp.s = 0,
                            (VirtualKeyCode::D, ElementState::Pressed) => comp.d = 1,
                            (VirtualKeyCode::D, ElementState::Released) => comp.d = 0,
                            _ => (),
                        }
                    })
                }
                _ => (),
            }
        }
        self.keys = Default::default();
    }
}
