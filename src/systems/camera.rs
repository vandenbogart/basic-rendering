use cgmath::prelude::*;
use std::f32::consts::PI;

use winit::event::{ElementState, VirtualKeyCode};

use crate::{asset_manager::AssetManager, component_manager::ComponentManager, world::World};

use super::System;

pub struct Camera {
    eye: cgmath::Point3<f32>,
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
            eye: (0.0, 100.0, 300.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * self.proj() * self.view()
    }
    pub fn view(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
    pub fn proj(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
    pub fn unproject_click(&self, norm_x: f32, norm_y: f32) -> cgmath::Point3<f32> {
        let ndc = cgmath::vec4(2.0 * norm_x, 2.0 * (1.0 - norm_y), 0.99, 1.0);
        let ndc = ndc + cgmath::vec4(-1.0, -1.0, 0.0, 0.0);
        let proj_i = self.proj().invert().expect("Unable to invert proj matrix");
        let view_c = proj_i * ndc;
        let view_c = view_c / view_c.w;
        let view_i = self.view().invert().expect("Unable to invert view matrix");
        let world_c = view_i * view_c;
        cgmath::point3(world_c.x, world_c.y, world_c.z)
    }
    pub fn get_position(&self) -> cgmath::Point3<f32> {
        self.eye
    }
}

pub struct CameraSystem {
    pub camera: Camera,
    speed: f32,
    radius: f32,
    pos_x: f32,
    pos_y: f32,
    up: f32,
    down: f32,
    left: f32,
    right: f32,
    width: f32,
    height: f32,
}
impl CameraSystem {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            camera: Camera::new(width / height),
            speed: 1.0,
            radius: 200.0,
            pos_x: 0.0,
            pos_y: 0.0,
            up: 0.0,
            down: 0.0,
            right: 0.0,
            left: 0.0,
            width,
            height,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect = width as f32 / height as f32;
        self.width = width as f32;
        self.height = height as f32;
        self.camera = Camera::new(aspect);
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
    fn run(&mut self, world: &mut World, cm: &mut ComponentManager, am: &AssetManager, dt: f32) {
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
        let follow_position = cgmath::point3(0.0, 0.0, 0.0);
        let new_x = follow_position.x + ((self.pos_x * PI * 2.0).cos() * self.radius);
        let new_y = self.pos_y * self.radius;
        let new_z = follow_position.z + ((self.pos_x * PI * 2.0).sin() * self.radius);

        self.camera.target =
            cgmath::point3(follow_position.x, follow_position.y, follow_position.z);
        self.camera.eye = cgmath::point3(new_x, new_y, new_z);
    }
}
