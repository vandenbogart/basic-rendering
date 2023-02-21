use std::{f32::consts::PI, time::Duration};

use winit::event::{ElementState, VirtualKeyCode};

use crate::renderer::{CameraFollowComponent, GeometryComponent};

use super::{System, WASDControllerComponent};

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
impl Camera {
    pub fn new(aspect: f32) -> Camera {
        Camera {
            eye: (0.0, 10.0, 30.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

pub struct CameraSystem {
    camera: Camera,
    speed: f32,
    radius: f32,
    pos_x: f32,
    pos_y: f32,
    up: f32,
    down: f32,
    left: f32,
    right: f32,
}
impl CameraSystem {
    pub fn new(aspect: f32) -> Self {
        Self {
            camera: Camera::new(aspect),
            speed: 1.0,
            radius: 20.0,
            pos_x: 0.0,
            pos_y: 0.0,
            up: 0.0,
            down: 0.0,
            right: 0.0,
            left: 0.0,
        }
    }
    pub fn process_keyboard(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        match (keycode, state) {
            (VirtualKeyCode::D, ElementState::Pressed) => {
                self.right = 1.0;
            }
            (VirtualKeyCode::D, ElementState::Released) => {
                self.right = 0.0;
            }
            (VirtualKeyCode::A, ElementState::Pressed) => {
                self.left = 1.0;
            }
            (VirtualKeyCode::A, ElementState::Released) => {
                self.left = 0.0;
            }
            (VirtualKeyCode::W, ElementState::Pressed) => {
                self.up = 1.0;
            }
            (VirtualKeyCode::W, ElementState::Released) => {
                self.up = 0.0;
            }
            (VirtualKeyCode::S, ElementState::Pressed) => {
                self.down = 1.0;
            }
            (VirtualKeyCode::S, ElementState::Released) => {
                self.down = 0.0;
            }
            _ => (),
        };
    }
    pub fn view_proj(&self) -> cgmath::Matrix4<f32> {
        self.camera.build_view_projection_matrix()
    }
}
impl System for CameraSystem {
    fn run(&mut self, world: &mut crate::World, dt: Duration) {
        let result = world
            .query()
            .with_component::<CameraFollowComponent>()
            .with_component::<GeometryComponent>()
            .execute();
        let entity = result.get_entities()[0];
        let follow_position = result.get_component::<GeometryComponent>(entity).position;
        let dt = dt.as_secs_f32();

        if self.right == 1.0 && self.left == 0.0 {
            self.pos_x += self.speed * dt;
            if self.pos_x > 1.0 {
                self.pos_x = 0.0;
            }
        }
        if self.left == 1.0 && self.right == 0.0 {
            self.pos_x -= self.speed * dt;
            if self.pos_x < 0.0 {
                self.pos_x = 1.0;
            }
        }
        if self.up == 1.0 && self.down == 0.0 {
            self.pos_y += self.speed * dt;
            if self.pos_y > 1.0 {
                self.pos_y = 1.0;
            }
        }
        if self.down == 1.0 && self.up == 0.0 {
            self.pos_y -= self.speed * dt;
            if self.pos_y < 0.0 {
                self.pos_y = 0.0;
            }
        }
        let new_x = follow_position.x + ((self.pos_x * PI * 2.0).cos() * self.radius);
        let new_y = self.pos_y * self.radius;
        let new_z = follow_position.z + ((self.pos_x * PI * 2.0).sin() * self.radius);

        self.camera.target =
            cgmath::point3(follow_position.x, follow_position.y, follow_position.z);
        self.camera.eye = cgmath::point3(new_x, new_y, new_z);
    }
}
