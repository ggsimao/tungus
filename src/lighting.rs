use std::{cell::RefCell, rc::Rc};

use beryllium::Keycode;
use nalgebra_glm::*;

use crate::controls::{Controller, SignalHandler, SignalType, Slot};

pub struct DirectionalLight {
    pub dir: Vec3,
    pub amb: Vec3,
    pub diff: Vec3,
    pub spec: Vec3,
    pub on: bool,
}

impl DirectionalLight {
    pub fn new(dir: Vec3, amb: Vec3, diff: Vec3, spec: Vec3) -> Self {
        DirectionalLight {
            dir,
            amb,
            diff,
            spec,
            on: true,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PointLight {
    pub pos: Vec3,
    pub amb: Vec3,
    pub diff: Vec3,
    pub spec: Vec3,
    pub att: Vec3,
    pub on: bool,
}

impl PointLight {
    pub fn new(pos: Vec3, amb: Vec3, diff: Vec3, spec: Vec3, att: Vec3) -> Self {
        PointLight {
            pos,
            amb,
            diff,
            spec,
            att,
            on: true,
        }
    }
}

// phi: angle of the inner cone
// gamma: angle of the outer cone
pub struct Spotlight {
    pub pos: Vec3,
    pub dir: Vec3,
    amb: Vec3,
    diff: Vec3,
    spec: Vec3,
    pub att: Vec3,
    pub phi: f32,
    pub gamma: f32,
    pub on: bool,
}

impl Spotlight {
    pub fn new(
        pos: Vec3,
        dir: Vec3,
        amb: Vec3,
        diff: Vec3,
        spec: Vec3,
        att: Vec3,
        phi: f32,
        gamma: f32,
    ) -> Self {
        Spotlight {
            pos,
            dir,
            amb,
            diff,
            spec,
            att,
            phi,
            gamma,
            on: true,
        }
    }

    pub fn get_amb(&self) -> Vec3 {
        self.amb * (self.on as i32 as f32)
    }
    pub fn get_diff(&self) -> Vec3 {
        self.diff * (self.on as i32 as f32)
    }
    pub fn get_spec(&self) -> Vec3 {
        self.spec * (self.on as i32 as f32)
    }
}

pub struct FlashlightController {
    on: bool,
}

impl FlashlightController {
    pub fn new() -> Rc<RefCell<FlashlightController>> {
        Rc::new(RefCell::new(Self { on: false }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::F => self.on = !self.on,
            _ => (),
        }
    }
}

impl<'a> Slot for FlashlightController {
    fn on_signal(&mut self, signal: SignalType) {
        match signal {
            SignalType::KeyPressed(key) => self.on_key_pressed(key),
            _ => (),
        }
    }
}

impl<'a> Controller<'a, Spotlight, FlashlightController> for Rc<RefCell<FlashlightController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut FlashlightController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Spotlight) {
        let self_obj = (**self).borrow_mut();
        obj.on = self_obj.on;
    }
}

pub struct Lighting {
    pub dir: DirectionalLight,
    pub point: Vec<PointLight>,
    pub spot: Spotlight,
}
