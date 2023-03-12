use cgmath::InnerSpace;
use winit::event::{ElementState, MouseButton};

use crate::{ray::Ray, components::{click::Click, click_move::ClickMove, walkable_surface::WalkableSurface, walk_to::WalkTo}};

use super::{System, camera::Camera};

pub struct ClickSystem {
    mouse_x: f32,
    mouse_y: f32,
    state: ElementState,
    button: MouseButton,
    world_pos: Option<cgmath::Point3<f32>>,
    ray: Option<Ray>,
}
impl ClickSystem {
    pub fn new() -> Self {
        ClickSystem {
            mouse_x: 0.0,
            mouse_y: 0.0,
            state: ElementState::Released,
            button: MouseButton::Left,
            world_pos: None,
            ray: None,
        }
    }
    pub fn process_mousemove(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }
    pub fn process_click(&mut self, state: ElementState, button: MouseButton, camera: &Camera) {
        self.state = state;
        self.button = button;
        if state == ElementState::Pressed {
            let unprojected_pos = camera.unproject_click(self.mouse_x, self.mouse_y);
            self.ray = Some(Ray::new(camera.get_position(), (unprojected_pos - camera.get_position()).normalize()));
        }
    }
}

impl System for ClickSystem {
    fn run(
        &mut self,
        world: &mut crate::world::World,
        cm: &mut crate::component_manager::ComponentManager,
        am: &crate::asset_manager::AssetManager,
        dt: f32,
    ) {
        if self.state == ElementState::Pressed {
            if let Some(ray) = &self.ray {
                let hits = ray.test(world.get_entities().as_slice(), cm, am);
                if let Some(hit) = hits.get(0) {
                    cm.mut_all_by_type::<Click>().iter_mut().for_each(|(_, click)| {
                        click.target = Some(hit.entity);
                        click.screen_y = self.mouse_y;
                        click.screen_x = self.mouse_x;
                        click.world_pos = hit.position;
                    });

                    if let Some(_) = cm.get_component::<WalkableSurface>(hit.entity) {
                        cm.mut_all_by_type::<ClickMove>().iter_mut().for_each(|(_, click_move)| {
                            click_move.target = Some(hit.position);
                        });
                    }
                }
            }
        }
    }
}















