use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::time::SystemTime;

use crate::camera::Camera;
use crate::controls::{Controller, SignalType, Slot};
use crate::data::UniformBuffer;
use crate::lighting::Lighting;
use crate::meshes::{BasicMesh, Draw};
use crate::models::Model;
use crate::shaders::ShaderProgram;
use beryllium::Keycode;
use gl33::gl_enumerations::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

#[derive(Clone)]
pub struct SceneObject {
    drawable: Box<dyn Draw>,
    position: Vec3,
    scale: Vec3,
    orientation: Vec3, // currently unused
    model: Mat4,
    normal: Mat3,
    outline: Vec4, // last element indicates whether the object should be outlined
}

impl SceneObject {
    pub fn from<T: Draw + 'static>(object: T) -> Self {
        SceneObject {
            drawable: Box::new(object),
            position: Vec3::zeros(),
            scale: vec3(1.0, 1.0, 1.0),
            orientation: Vec3::zeros(),
            model: Mat4::identity(),
            normal: Mat3::identity(),
            outline: Vec4::zeros(),
        }
    }

    pub fn get_pos(&self) -> Vec3 {
        self.position
    }

    pub fn get_model(&self) -> Mat4 {
        self.model
    }

    pub fn get_normal(&self) -> Mat3 {
        self.normal
    }

    pub fn set_outline(&mut self, color: Vec3) {
        self.outline.x = color.x;
        self.outline.y = color.y;
        self.outline.z = color.z;
    }

    pub fn enable_outline(&mut self, enable: bool) {
        self.outline.w = enable as i32 as f32;
    }

    pub fn scale(&mut self, factor: f32) {
        self.scale *= factor;
        self.model = translation(&self.position) * scaling(&self.scale);
        self.normal = mat4_to_mat3(&self.model.try_inverse().unwrap().transpose());
    }

    pub fn translate(&mut self, offset: &Vec3) {
        self.position += offset;
        self.model = translation(&self.position) * scaling(&self.scale);
        self.normal = mat4_to_mat3(&self.model.try_inverse().unwrap().transpose());
    }

    pub fn draw_outline(&self, shader: &ShaderProgram) {
        unsafe {
            glStencilFunc(GL_NOTEQUAL, 1, 0xFF);
            glStencilMask(0x00);
            glDisable(GL_DEPTH_TEST);
        }

        shader.set_3f("outlineColor", &self.outline.xyz());
        self.drawable.draw(shader);

        unsafe {
            glStencilMask(0xFF);
            glStencilFunc(GL_ALWAYS, 1, 0xFF);
            glEnable(GL_DEPTH_TEST);
        }
    }
    pub fn has_outline(&self) -> bool {
        self.outline.w > 0.0
    }
}

impl Draw for SceneObject {
    fn draw(&self, shader: &ShaderProgram) {
        // println!("{:?}", (self.position.x, self.position.y, self.position.z));
        self.drawable.draw(shader);
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(self.clone())
    }
}

pub struct SceneParameters {
    pub visualize_normals: bool,
    pub start: SystemTime,
}

impl SceneParameters {
    pub fn init() -> Self {
        Self {
            visualize_normals: false,
            start: SystemTime::now(),
        }
    }
}

pub struct SceneController {
    pub signal_list: Vec<SignalType>,
    visualize_normals: bool,
}

impl SceneController {
    pub fn new() -> Rc<RefCell<SceneController>> {
        Rc::new(RefCell::new(Self {
            signal_list: vec![],
            visualize_normals: false,
        }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::N => self.visualize_normals = !self.visualize_normals,
            _ => (),
        }
    }
}

impl<'a> Slot<'a> for SceneController {
    fn on_signal(&mut self, signal: SignalType) {
        self.signal_list.push(signal);
    }
}

impl<'a> Controller<'a, SceneParameters, SceneController> for Rc<RefCell<SceneController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut SceneController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut SceneParameters) {
        let mut self_obj = (**self).borrow_mut();
        for signal in self_obj.signal_list.clone() {
            match signal {
                SignalType::KeyPressed(key) => self_obj.on_key_pressed(key),
                _ => (),
            }
        }
        obj.visualize_normals = self_obj.visualize_normals;
        self_obj.signal_list.clear();
    }
}

pub struct Scene<'a> {
    pub objects: &'a Vec<&'a SceneObject>,
    pub skyboxes: &'a Vec<&'a SceneObject>,
    pub object_shader: &'a ShaderProgram,
    pub skybox_shader: &'a ShaderProgram,
    pub outline_shader: &'a ShaderProgram,
    pub debug_shader: &'a ShaderProgram,
    pub camera: &'a Camera,
    pub lighting: &'a Lighting,
    pub params: &'a SceneParameters,
}

impl<'a> Scene<'a> {
    pub fn compose(&self, ubo: &UniformBuffer) {
        let mut ordered_objects = self.objects.clone();

        // Doesn't take into account different faces of the same object
        ordered_objects.sort_by(|a, b| self.distance_compare(a, b));

        let projection = perspective(1.0, self.camera.get_fov(), 0.1, 100.0);
        let view = self.camera.look_at();

        ubo.set_view_mat(&view);
        ubo.set_projection_mat(&projection);

        self.object_shader.use_program();
        self.set_lighting_uniforms();
        for object in ordered_objects {
            ubo.set_model_mat(&object.get_model());
            ubo.set_normal_mat(&object.get_normal());
            object.draw(&self.object_shader);
            // if self.params.visualize_normals {
            //     self.debug_shader.use_program();
            //     object.draw(&self.debug_shader);
            //     self.object_shader.use_program();
            // }
            if object.has_outline() {
                self.outline_shader.use_program();
                let outline_scale = scale(&object.get_model(), &vec3(1.1, 1.1, 1.1));
                ubo.set_model_mat(&outline_scale);
                object.draw_outline(&self.outline_shader);
                self.object_shader.use_program();
            }
        }

        unsafe {
            glDisable(GL_CULL_FACE);
            glDepthFunc(GL_LEQUAL);
        }

        let view = mat3_to_mat4(&mat4_to_mat3(&self.camera.look_at()));
        ubo.set_view_mat(&view);

        self.skybox_shader.use_program();

        for skybox in self.skyboxes {
            skybox.draw(self.skybox_shader);
        }

        unsafe {
            glEnable(GL_CULL_FACE);
            glDepthFunc(GL_LESS);
        }
    }

    fn distance_compare(&self, a: &SceneObject, b: &SceneObject) -> Ordering {
        let distance_a = length(&(self.camera.get_pos() - a.get_pos()));
        let distance_b = length(&(self.camera.get_pos() - b.get_pos()));
        distance_b.partial_cmp(&distance_a).unwrap()
    }

    fn set_lighting_uniforms(&self) {
        self.object_shader
            .set_directional_light("dirLight", &self.lighting.dir);
        for (i, point) in self.lighting.point.iter().enumerate() {
            self.object_shader
                .set_point_light(format!("pointLights[{}]", i).as_str(), &point);
        }
        self.object_shader
            .set_spotlight("spotlight", &self.lighting.spot);
    }
}
