

use winit::event::{ElementState, ModifiersState, MouseButton, VirtualKeyCode};

use super::{ClickMoveComponent, ClickMoveState, System};

pub struct InputSystem {
    keys: Vec<(VirtualKeyCode, ElementState)>,
    mouse_pos: cgmath::Point2<f32>,
    last_click_pos: Option<cgmath::Point2<f32>>,
}
impl InputSystem {
    pub fn new() -> Self {
        InputSystem::default()
    }
    pub fn process_keyboard(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        self.keys.push((keycode, state));
    }
    pub fn process_mouse_move(&mut self, x: f32, y: f32, _modifiers: ModifiersState) {
        self.mouse_pos.x = x;
        self.mouse_pos.y = y;
    }
    pub fn process_mouse_click(&mut self, state: ElementState, _button: MouseButton) {
        if state == ElementState::Pressed {
            self.last_click_pos = Some(cgmath::point2(self.mouse_pos.x, self.mouse_pos.y));
        }
    }
}
impl Default for InputSystem {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            mouse_pos: cgmath::point2(0.0, 0.0),
            last_click_pos: None,
        }
    }
}
impl System for InputSystem {
    fn run(&mut self, world: &mut crate::World, _dt: f32) {
        let result = world
            .query()
            .with_component::<ClickMoveComponent>()
            .execute();
        result.get_entities().iter().for_each(|ent| {
            let mut comp = result.get_component_mut::<ClickMoveComponent>(*ent);
            match (self.last_click_pos, comp.state) {
                (Some(pos), ClickMoveState::Initial) => {
                    comp.last_click_pos = Some(pos);
                    comp.state = ClickMoveState::Move;
                }
                (Some(pos), ClickMoveState::Waiting) => {
                    if comp.last_click_pos.unwrap() != pos {
                        comp.last_click_pos = Some(pos);
                        comp.state = ClickMoveState::Move;
                    }
                }
                _ => ()
            }
        });

        self.keys = Default::default();
    }
}
