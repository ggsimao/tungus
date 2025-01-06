use std::alloc::{alloc, Layout};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::path::Path;
use std::ptr::null_mut;
use std::rc::Rc;

use crate::controls::{Controller, SignalType, Slot};
use crate::data::{Framebuffer, ShadowFramebuffer, UniformBuffer};
use crate::meshes::{BasicMesh, Draw};
use crate::scene::{Scene, SceneObject};
use crate::shaders::ShaderProgram;
use crate::spatial::Spatial;
use crate::utils::constrained_step;
use beryllium::Keycode;
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

const GAMMA: f32 = 2.2;

const SHADOW_RESOLUTION: (u32, u32) = (1024, 1024);

struct ScreenParameters {
    clear_color: Vec4,
    pub sobel_on: bool,
    pub msaa_on: bool,
    pub gamma: f32,
    pub window_size: (u32, u32),
}

pub struct Screen {
    canvas: SceneObject,
    fbo: Framebuffer,
    sfbo: ShadowFramebuffer,
    shader: ShaderProgram,
    ubo: UniformBuffer,
    params: ScreenParameters,
}

impl<'a> Screen {
    pub fn new(
        canvas: SceneObject,
        clear_color: Vec4,
        window_size: (u32, u32),
        shader: ShaderProgram,
        ubo: UniformBuffer,
    ) -> Self {
        let fbo = Framebuffer::new(window_size).unwrap();
        fbo.setup();
        let sfbo = ShadowFramebuffer::new(SHADOW_RESOLUTION).unwrap();
        sfbo.setup();
        let params = ScreenParameters {
            clear_color,
            sobel_on: false,
            msaa_on: false,
            gamma: GAMMA,
            window_size,
        };

        Self {
            canvas,
            fbo,
            sfbo,
            shader,
            ubo,
            params,
        }
    }

    pub fn clear_buffers(&self) {
        // TODO: maybe make more generic
        unsafe {
            glClearColor(
                self.params.clear_color.x,
                self.params.clear_color.y,
                self.params.clear_color.z,
                self.params.clear_color.w,
            );
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
        }
    }

    pub fn draw_on_framebuffer(&mut self, scene: &mut Scene) {
        ShaderProgram::reset_tex_count();
        self.generate_shadow_maps(scene);
        self.fbo.bind();
        self.clear_buffers();
        scene.compose(&self.ubo);
        Framebuffer::clear_binding();
    }

    fn generate_shadow_maps(&self, scene: &mut Scene) {
        self.sfbo.bind();

        let mut m_viewport = [0; 4];
        unsafe {
            glGetIntegerv(GL_VIEWPORT, m_viewport.as_mut_ptr());
        }

        self.clear_buffers();
        scene.set_shadow_maps(&self.ubo, &self.sfbo);
        unsafe {
            glViewport(m_viewport[0], m_viewport[1], m_viewport[2], m_viewport[3]);
        }

        ShadowFramebuffer::clear_binding();
    }

    pub fn draw_on_another(&mut self, other: &Screen, scaling: f32, offset: Vec2) {
        other.fbo.bind();
        self.ubo.bind_base();

        let mut transformed_canvas = self.canvas.clone();
        transformed_canvas.scale(&vec3(scaling, scaling, scaling));
        transformed_canvas.translate(&vec3(offset.x, offset.y, 0.0));

        unsafe {
            glDisable(GL_DEPTH_TEST);
        }

        ShaderProgram::reset_tex_count();
        self.shader.use_program();
        self.shader.set_1f("gamma", 1.0);
        self.shader
            .set_texture2D_multisample("screenTexture", self.fbo.get_texture());
        self.ubo.set_model_mat(&transformed_canvas.get_model());
        transformed_canvas.draw(&self.shader);

        unsafe {
            glEnable(GL_DEPTH_TEST);
        }
    }

    pub fn draw_on_screen(&mut self) {
        Framebuffer::clear_binding();
        self.ubo.bind_base();

        unsafe {
            glClearColor(1.0, 1.0, 1.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDisable(GL_DEPTH_TEST);
        }

        ShaderProgram::reset_tex_count();
        self.shader.use_program();
        self.shader.set_1f("gamma", self.params.gamma);
        self.shader
            .set_texture2D_multisample("screenTexture", self.fbo.get_texture());
        self.shader
            .set_1i("sampleCount", self.fbo.get_texture().get_samples() as i32);
        self.shader.set_1b("applySobel", self.params.sobel_on);
        self.shader.set_1b("applyMSAA", self.params.msaa_on);
        self.ubo.set_model_mat(&identity());
        self.canvas.draw(&self.shader);

        unsafe {
            glEnable(GL_DEPTH_TEST);
        }
    }
}

pub struct ScreenController {
    sobel_on: bool,
    msaa_on: bool,
    gamma: f32,
}

impl ScreenController {
    pub fn new() -> Rc<RefCell<ScreenController>> {
        Rc::new(RefCell::new(Self {
            sobel_on: false,
            msaa_on: true,
            gamma: GAMMA,
        }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::E => self.sobel_on = !self.sobel_on,
            Keycode::M => self.msaa_on = !self.msaa_on,
            Keycode::EQUALS => self.gamma = (self.gamma + 0.2).min(3.0),
            Keycode::MINUS => self.gamma = (self.gamma - 0.2).max(1.0),
            _ => (),
        }
    }
}

impl<'a> Slot for ScreenController {
    fn on_signal(&mut self, signal: SignalType) {
        match signal {
            SignalType::KeyPressed(key) => self.on_key_pressed(key),
            _ => (),
        }
    }
}

impl<'a> Controller<'a, Screen, ScreenController> for Rc<RefCell<ScreenController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut ScreenController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Screen) {
        let self_obj = (**self).borrow();
        obj.params.sobel_on = self_obj.sobel_on;
        obj.params.msaa_on = self_obj.msaa_on;
        obj.params.gamma = self_obj.gamma;
    }
}
