use glam::{Mat4, Vec3};
use std::f32::consts::PI;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vec3::new(0.0, 8.0, 35.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect: width as f32 / height as f32,
            fovy: PI / 3.0,
            znear: 0.1,
            zfar: 200.0,
            yaw: -PI / 2.0,
            pitch: -0.2,
            speed: 15.0,
            sensitivity: 0.1,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.position + self.get_forward(), self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn get_forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_forward().cross(self.up).normalize()
    }

    pub fn move_forward(&mut self, dt: f32) {
        self.position += self.get_forward() * self.speed * dt;
    }

    pub fn move_backward(&mut self, dt: f32) {
        self.position -= self.get_forward() * self.speed * dt;
    }

    pub fn move_left(&mut self, dt: f32) {
        self.position -= self.get_right() * self.speed * dt;
    }

    pub fn move_right(&mut self, dt: f32) {
        self.position += self.get_right() * self.speed * dt;
    }

    pub fn move_up(&mut self, dt: f32) {
        self.position += self.up * self.speed * dt;
    }

    pub fn move_down(&mut self, dt: f32) {
        self.position -= self.up * self.speed * dt;
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw * self.sensitivity;
        self.pitch += delta_pitch * self.sensitivity;
        
        // Limitar el pitch para evitar gimbal lock
        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}