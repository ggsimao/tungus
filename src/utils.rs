use beryllium::Keycode;
use rand::Rng;
use std::ops::{Add, Rem, Sub};
use std::rc::Rc;
use std::{cell::RefCell, fs};

use nalgebra_glm::{rotation, vec3, Mat4, Vec3};

use crate::{
    controls::{Controller, SignalType, Slot},
    scene::{Instance, SceneObject},
    spatial::Spatial,
};

pub struct RandomTransform {
    axis: Vec3,
    dir: Vec3,
    ang_step: f32,
    lin_step: f32,
    ang_upd_rate: u32,
    lin_upd_rate: u32,
    rotation: Mat4,
    translation: Vec3,
}

impl RandomTransform {
    pub fn continuous(ang_step: f32, lin_step: f32, ang_rate: u32, lin_rate: u32) -> Self {
        let mut rng = rand::thread_rng();
        let axis = vec3(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        )
        .normalize();
        let dir = vec3(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        )
        .normalize();
        RandomTransform {
            axis,
            dir,
            ang_step,
            lin_step,
            ang_upd_rate: ang_rate,
            lin_upd_rate: lin_rate,
            rotation: rotation(ang_step, &axis),
            translation: lin_step * dir,
        }
    }
    pub fn position(
        obj: &mut impl Spatial,
        range_x: (f32, f32),
        range_y: (f32, f32),
        range_z: (f32, f32),
    ) {
        let mut rng = rand::thread_rng();
        let offset_x = rng.gen_range(range_x.0..=range_x.1);
        let offset_y = rng.gen_range(range_y.0..=range_y.1);
        let offset_z = rng.gen_range(range_z.0..=range_z.1);
        obj.translate(&vec3(offset_x, offset_y, offset_z));
    }
    #[inline(always)]
    pub fn rotate(&self, obj: &mut impl Spatial) {
        obj.apply_rotation(&self.rotation);
        // obj.rotate(self.ang_step, &self.axis);
    }
    #[inline(always)]
    pub fn translate(&self, obj: &mut impl Spatial) {
        obj.translate(&self.translation);
    }
    pub fn update_axis(&mut self) {
        let mut rng = rand::thread_rng();
        self.axis = vec3(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        )
        .normalize();
        self.rotation = rotation(self.ang_step, &self.axis);
    }
    pub fn update_dir(&mut self) {
        let mut rng = rand::thread_rng();
        self.dir = vec3(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        )
        .normalize();
        self.translation = self.lin_step * self.dir;
    }
}

pub struct RTController {
    tick_list: Vec<(u32, u32)>, // ang, lin
}

impl<'a> Slot for RTController {
    fn on_signal(&mut self, _: SignalType) {}
}

impl RTController {
    pub fn new() -> Rc<RefCell<RTController>> {
        Rc::new(RefCell::new(Self { tick_list: vec![] }))
    }

    pub fn add_rts(&mut self, rts: &Vec<RandomTransform>) {
        for rt in rts {
            self.tick_list.push((rt.ang_upd_rate, rt.lin_upd_rate));
        }
    }
}

impl<'a> Controller<'a, Vec<RandomTransform>, RTController> for Rc<RefCell<RTController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut RTController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Vec<RandomTransform>) {
        let mut self_obj = (**self).borrow_mut();
        for i in 0..self_obj.tick_list.len() {
            if self_obj.tick_list[i].0 == 1 {
                obj[i].update_axis();
                self_obj.tick_list[i].0 = obj[i].ang_upd_rate;
            } else if self_obj.tick_list[i].0 > 1 {
                self_obj.tick_list[i].0 -= 1;
            }

            if self_obj.tick_list[i].1 == 1 {
                obj[i].update_dir();
                self_obj.tick_list[i].1 = obj[i].lin_upd_rate;
            } else if self_obj.tick_list[i].1 > 1 {
                self_obj.tick_list[i].1 -= 1;
            }
        }
    }
}

pub fn constrained_step<T: Sub<Output = T> + Rem<Output = T> + Add<Output = T> + Copy>(
    curr_value: T,
    min: T,
    step: T,
    modulus: T,
) -> T {
    (curr_value - min + (step % modulus) + modulus) % modulus + min
}
