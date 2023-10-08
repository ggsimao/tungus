use std::f32::consts::PI;

use nalgebra_glm::*;

const ANGLE_LOWER_BOUND: f32 = 0.001;

pub struct Camera {
    pos: Vec3,
    direction: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
    fov: f32,
}

impl Camera {
    pub fn new(initial_pos: Vec3) -> Self {
        let focal_point = -initial_pos;
        let yaw = angle(
            &vec3(focal_point.x, 0.0, focal_point.z),
            &vec3(1.0, 0.0, 0.0),
        );
        let pitch = angle(
            &vec3(focal_point.x, focal_point.y, 0.0),
            &vec3(1.0, 0.0, 0.0),
        );
        Camera {
            pos: initial_pos,
            direction: focal_point,
            pitch,
            yaw,
            roll: 0.0,
            fov: 1.0,
        }
    }

    pub fn look_at(&self) -> Mat4 {
        look_at(
            &self.pos,
            &(self.direction + self.pos),
            &vec3(0.0, 1.0, 0.0),
        )
    }

    pub fn translate(&mut self, offset: Vec3) {
        self.pos -= offset.z * self.direction;

        let global_up = vec3(0.0, 1.0, 0.0);
        let camera_right = normalize(&cross(&global_up, &self.direction));
        self.pos -= offset.x * camera_right;

        let camera_up = cross(&self.direction, &camera_right);
        self.pos -= offset.y * camera_up;
    }
    pub fn translate_longitudinal(&mut self, offset: f32) {
        self.translate(vec3(offset, 0.0, 0.0));
    }
    pub fn translate_axial(&mut self, offset: f32) {
        self.translate(vec3(0.0, offset, 0.0));
    }
    pub fn translate_frontal(&mut self, offset: f32) {
        self.translate(vec3(0.0, 0.0, offset));
    }
    pub fn translate_forward(&mut self, offset: f32) {
        let mut direction = Vec3::zeros();
        direction.x = self.yaw.cos();
        direction.z = self.yaw.sin();
        direction *= offset;
        self.pos -= direction;
    }

    pub fn rotate(&mut self, euler_angles: Vec3) {
        self.pitch = (self.pitch + euler_angles.x.to_radians())
            .max(-PI / 2.0 + ANGLE_LOWER_BOUND)
            .min(PI / 2.0 - ANGLE_LOWER_BOUND);
        self.yaw += euler_angles.y.to_radians();
        self.roll += euler_angles.z.to_radians();

        self.direction.x = self.yaw.cos() * self.pitch.cos();
        self.direction.y = self.pitch.sin();
        self.direction.z = self.yaw.sin() * self.pitch.cos();
    }
    pub fn rotate_pitch(&mut self, rotation: f32) {
        self.rotate(vec3(rotation, 0.0, 0.0));
    }
    pub fn rotate_yaw(&mut self, rotation: f32) {
        self.rotate(vec3(0.0, rotation, 0.0));
    }
    pub fn rotate_roll(&mut self, rotation: f32) {
        self.rotate(vec3(0.0, 0.0, rotation));
    }

    pub fn change_fov(&mut self, offset: f32) {
        self.fov += offset.to_radians();
    }
    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }
}
