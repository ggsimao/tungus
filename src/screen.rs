use std::cell::RefCell;
use std::rc::Rc;

use crate::controls::{Controller, SignalType, Slot};
use crate::meshes::{BasicMesh, Draw};
use crate::rendering::Framebuffer;
use crate::scene::{Scene, SceneObject};
use crate::shaders::ShaderProgram;
use beryllium::Keycode;
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

pub struct Screen<'a> {
    canvas: SceneObject,
    clear_color: Vec4,
    fbo: Framebuffer,
    shader: &'a ShaderProgram,
    filter: bool,
}

impl<'a> Screen<'a> {
    pub fn new(
        canvas: SceneObject,
        clear_color: Vec4,
        fbo: Framebuffer,
        shader: &'a ShaderProgram,
    ) -> Self {
        Self {
            canvas,
            clear_color,
            fbo,
            shader,
            filter: false,
        }
    }
    pub fn clear_color(&self) {
        unsafe {
            glClearColor(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                self.clear_color.w,
            );
        }
    }

    pub fn clear_buffers(&self) {
        // TODO: maybe make more generic
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
        }
    }

    pub fn draw_on_framebuffer(&self, scene: &Scene) {
        self.fbo.bind();
        self.clear_color();
        self.clear_buffers();
        unsafe {
            glEnable(GL_DEPTH_TEST);
        }
        scene.draw();
    }

    pub fn bind(&self) {
        self.fbo.bind();
    }

    pub fn draw_on_another(&self, other: &Screen, scaling: f32, offset: Vec2) {
        other.fbo.bind();

        let mut transformed_canvas = self.canvas.clone();
        transformed_canvas.scale(scaling);
        transformed_canvas.translate(&vec3(offset.x, offset.y, 0.0));

        unsafe {
            glDisable(GL_DEPTH_TEST);
        }

        self.shader.use_program();
        self.shader
            .set_texture2D("screenTexture", self.fbo.get_texture());
        transformed_canvas.draw(&self.shader);
    }

    pub fn draw_on_screen(&self) {
        Framebuffer::clear_binding();

        unsafe {
            glClearColor(1.0, 1.0, 1.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDisable(GL_DEPTH_TEST);
        }

        self.shader.use_program();
        self.shader
            .set_texture2D("screenTexture", self.fbo.get_texture());
        self.shader.set_1b("applyFilter", self.filter);
        self.canvas.draw(&self.shader);
    }
}

pub struct ScreenController {
    pub signal_list: Vec<SignalType>,
    filter: bool,
}

impl ScreenController {
    pub fn new() -> Rc<RefCell<ScreenController>> {
        Rc::new(RefCell::new(Self {
            signal_list: vec![],
            filter: false,
        }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::E => self.filter = !self.filter,
            _ => (),
        }
    }
}

impl<'a> Slot<'a> for ScreenController {
    fn on_signal(&mut self, signal: SignalType) {
        self.signal_list.push(signal);
    }
}

impl<'a> Controller<'a, Screen<'a>, ScreenController> for Rc<RefCell<ScreenController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut ScreenController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Screen) {
        let mut self_obj = (**self).borrow_mut();
        for signal in self_obj.signal_list.clone() {
            match signal {
                SignalType::KeyPressed(key) => self_obj.on_key_pressed(key),
                _ => (),
            }
        }
        obj.filter = self_obj.filter;
        self_obj.signal_list.clear();
    }
}
