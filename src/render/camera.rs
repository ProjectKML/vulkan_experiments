use crate::render::math_util::direction_from_rotation;
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashSet;
use std::f32::consts::PI;
use winit::event::VirtualKeyCode;

pub const SENSITIVITY_X: f32 = 0.165f32;
pub const SENSITIVITY_Y: f32 = 0.165f32;
pub const SPEED: f32 = 12.5f32;

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Vec3,

    pub view_projection_matrix: Mat4,
}

impl Camera {
    pub fn new(position: Vec3, rotation: Vec3) -> Self {
        Self {
            position,
            rotation,
            view_projection_matrix: Mat4::default(),
        }
    }

    pub fn update(&mut self, pressed_keys: &HashSet<VirtualKeyCode>, delta: f32) {
        let mut invert_y_matrix = Mat4::default();
        invert_y_matrix.col_mut(1).y = -1.0f32;

        let projection_matrix =
            Mat4::perspective_lh(90f32.to_radians(), 16f32 / 9f32, 0.1f32, 10000f32)
                * invert_y_matrix;

        let look_at = self.position + direction_from_rotation(&self.rotation);
        self.view_projection_matrix = projection_matrix
            * Mat4::look_at_lh(self.position, look_at, Vec3::new(0f32, 1f32, 0f32));

        if pressed_keys.contains(&VirtualKeyCode::W) {
            self.move_forward(delta);
        }
        if pressed_keys.contains(&VirtualKeyCode::S) {
            self.move_back(delta);
        }
        if pressed_keys.contains(&VirtualKeyCode::A) {
            self.move_left(delta);
        }
        if pressed_keys.contains(&VirtualKeyCode::D) {
            self.move_right(delta);
        }
        if pressed_keys.contains(&VirtualKeyCode::Space) {
            self.move_up(delta);
        }
        if pressed_keys.contains(&VirtualKeyCode::LShift) {
            self.move_down(delta);
        }
    }

    pub fn move_mouse(&mut self, delta_x: f64, delta_y: f64) {
        let delta_position = Vec2::new(delta_x as f32, delta_y as f32);

        let move_position = delta_position * Vec2::new(SENSITIVITY_X, SENSITIVITY_Y) * 0.0075f32;

        self.rotation.x += move_position.x;
        if self.rotation.x > PI * 2f32 {
            self.rotation.x = 0f32;
        } else if self.rotation.x < 0f32 {
            self.rotation.x = PI * 2f32;
        }

        self.rotation.y -= move_position.y;
        if self.rotation.y > PI / 2f32 - 0.15f32 {
            self.rotation.x = PI / 2f32 - 0.15f32;
        } else if self.rotation.y < 0.15f32 - PI / 2f32 {
            self.rotation.y = 0.15f32 - PI / 2f32;
        }
    }

    pub fn move_forward(&mut self, delta: f32) {
        self.position.x += self.rotation.x.sin() * SPEED * delta;
        self.position.z += self.rotation.x.cos() * SPEED * delta;
    }

    pub fn move_right(&mut self, delta: f32) {
        self.position.x += self.rotation.x.cos() * SPEED * delta;
        self.position.z -= self.rotation.x.sin() * SPEED * delta;
    }

    pub fn move_back(&mut self, delta: f32) {
        self.position.x -= self.rotation.x.sin() * SPEED * delta;
        self.position.z -= self.rotation.x.cos() * SPEED * delta;
    }

    pub fn move_left(&mut self, delta: f32) {
        self.position.x -= self.rotation.x.cos() * SPEED * delta;
        self.position.z += self.rotation.x.sin() * SPEED * delta;
    }

    pub fn move_down(&mut self, delta: f32) {
        self.position.y -= SPEED * delta;
    }

    pub fn move_up(&mut self, delta: f32) {
        self.position.y += SPEED * delta;
    }
}
