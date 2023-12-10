use std::{borrow::BorrowMut, cell::RefCell, f32::consts::PI, rc::Rc};

use beryllium::Keycode;
use glfw::Key;
use nalgebra_glm::*;

use crate::controls::{Controller, SignalHandler, SignalType, Slot};

const ANGLE_LOWER_BOUND: f32 = 0.001;

#[derive(Clone, Copy)]
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
    pub fn translate_vertical(&mut self, offset: f32) {
        self.pos.y += offset;
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
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }
    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }
    pub fn invert(&self) -> Camera {
        let mut inverted = self.clone();
        inverted.rotate_pitch(inverted.get_pitch().to_degrees() * -2.0);
        inverted.rotate_yaw(180.0);
        inverted
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
    pub fn get_dir(&self) -> Vec3 {
        self.direction
    }
}

pub struct CameraController {
    pub signal_list: Vec<SignalType>,
    pub inv_vertical: bool,
    pub trans_speed: f32,
    pub rot_speed: f32,
    pub zoom_speed: f32,
    pub cycle_time: f32,
    pub positive_delta_mov: Vec3,
    pub negative_delta_mov: Vec3,
    pub delta_rot: Vec3,
    pub delta_zoom: f32,
}

impl<'a> CameraController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            signal_list: vec![],
            inv_vertical: false,
            trans_speed: 0.1,
            rot_speed: 0.1,
            zoom_speed: 0.1,
            cycle_time: 0.0,
            positive_delta_mov: Vec3::zeros(),
            negative_delta_mov: Vec3::zeros(),
            delta_rot: Vec3::zeros(),
            delta_zoom: 0.0,
        }))
    }
    pub fn set_speeds(&mut self, cycle_time: f32) {
        self.trans_speed = cycle_time * 0.002;
        self.rot_speed = cycle_time * 0.01;
        self.zoom_speed = cycle_time * 0.1;
        self.cycle_time = cycle_time;
    }

    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::D => self.positive_delta_mov.x = self.trans_speed,
            Keycode::A => self.negative_delta_mov.x = self.trans_speed,
            Keycode::SPACE => self.positive_delta_mov.y = self.trans_speed,
            Keycode::LCTRL => self.negative_delta_mov.y = self.trans_speed,
            Keycode::S => self.positive_delta_mov.z = self.trans_speed,
            Keycode::W => self.negative_delta_mov.z = self.trans_speed,
            _ => {}
        }
    }
    pub fn on_key_released(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::D => self.positive_delta_mov.x = 0.0,
            Keycode::A => self.negative_delta_mov.x = 0.0,
            Keycode::SPACE => self.positive_delta_mov.y = 0.0,
            Keycode::LCTRL => self.negative_delta_mov.y = 0.0,
            Keycode::S => self.positive_delta_mov.z = 0.0,
            Keycode::W => self.negative_delta_mov.z = 0.0,
            _ => {}
        }
    }
    pub fn on_mouse_moved(&mut self, x: i32, y: i32) {
        self.delta_rot += vec3(-x as f32, y as f32, 0.0) * self.rot_speed;
    }
    pub fn on_mouse_scrolled(&mut self, y: i32) {
        self.delta_zoom += y as f32 * self.zoom_speed;
    }
}

impl<'a> Slot<'a> for CameraController {
    fn on_signal(&mut self, signal: SignalType) {
        self.signal_list.push(signal);
    }
}

impl<'a> Controller<'a, Camera, CameraController> for Rc<RefCell<CameraController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut CameraController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&self, obj: &mut Camera) {
        let mut self_obj = (**self).borrow_mut();
        for signal in self_obj.signal_list.clone() {
            match signal {
                SignalType::KeyPressed(key) => self_obj.on_key_pressed(key),
                SignalType::KeyReleased(key) => self_obj.on_key_released(key),
                SignalType::MouseMoved(x, y) => self_obj.on_mouse_moved(x, y),
                SignalType::MouseScrolled(y) => self_obj.on_mouse_scrolled(y),
                _ => (),
            }
        }
        let positive = self_obj.positive_delta_mov;
        let negative = self_obj.negative_delta_mov;
        let delta_mov = positive - negative;
        obj.translate_longitudinal(delta_mov.x);
        obj.translate_vertical(delta_mov.y);
        obj.translate_forward(delta_mov.z);
        obj.rotate(self_obj.delta_rot);
        obj.change_fov(self_obj.delta_zoom);
        self_obj.delta_rot *= 0.0;
        self_obj.delta_zoom = 0.0;

        self_obj.signal_list.clear();
    }
}
