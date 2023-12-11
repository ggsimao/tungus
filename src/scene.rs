use std::cmp::Ordering;

use crate::camera::Camera;
use crate::lighting::Lighting;
use crate::meshes::{BasicMesh, Draw};
use crate::models::Model;
use crate::shaders::ShaderProgram;
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

        shader.set_matrix_4fv("modelMatrix", &scale(&self.model, &vec3(1.1, 1.1, 1.1)));
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
        shader.set_matrix_4fv("modelMatrix", &self.model);
        shader.set_matrix_3fv("normalMatrix", &self.normal);
        self.drawable.draw(shader);
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(self.clone())
    }
}

pub struct Scene<'a> {
    pub objects: &'a Vec<&'a SceneObject>,
    pub skyboxes: &'a Vec<&'a SceneObject>,
    pub object_shader: &'a ShaderProgram,
    pub skybox_shader: &'a ShaderProgram,
    pub outline_shader: &'a ShaderProgram,
    pub camera: &'a Camera,
    pub lighting: &'a Lighting,
}

impl<'a> Scene<'a> {
    // It might seem strange for this not to be from the trait Draw, but
    // this wouldn't work well with other things that accept Draw. Maybe choose a better name?
    pub fn draw(&self) {
        let mut ordered_objects = self.objects.clone();
        
        // Doesn't take into account different faces of the same object
        ordered_objects.sort_by(|a, b| self.distance_compare(a, b));
        
        self.reinitialize_object_shader();
        for object in ordered_objects {
            object.draw(&self.object_shader);
            if object.has_outline() {
                self.reinitialize_outline_shader();
                object.draw_outline(&self.outline_shader);
                self.reinitialize_object_shader();
            }
        }

        unsafe {
            glDisable(GL_CULL_FACE);
            glDepthFunc(GL_LEQUAL);
        }

        self.reinitialize_skybox_shader();

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

    fn reinitialize_object_shader(&self) {
        self.object_shader.use_program();
        self.object_shader.set_view(&self.camera);

        // TODO: take this out of the method so it doesn't have to be called more than necessary
        let projection = &perspective(1.0, self.camera.get_fov(), 0.1, 100.0);

        // TODO: change this for a more general set_lighting method
        self.object_shader
            .set_directional_light("dirLight", &self.lighting.dir);
        self.object_shader
            .set_matrix_4fv("projectionMatrix", projection);
        for (i, point) in self.lighting.point.iter().enumerate() {
            self.object_shader
                .set_point_light(format!("pointLights[{}]", i).as_str(), &point);
        }
        self.object_shader
            .set_spotlight("spotlight", &self.lighting.spot);
    }

    fn reinitialize_skybox_shader(&self) {
        self.skybox_shader.use_program();

        let view = &mat3_to_mat4(&mat4_to_mat3(&self.camera.look_at()));
        let projection = &perspective(1.0, self.camera.get_fov(), 0.1, 100.0);

        self.skybox_shader.set_matrix_4fv("viewMatrix", view);
        self.skybox_shader
            .set_matrix_4fv("projectionMatrix", projection);
    }

    fn reinitialize_outline_shader(&self) {
        // TODO: take this out of the method so it doesn't have to be called more than necessary
        let projection = &perspective(1.0, self.camera.get_fov(), 0.1, 100.0);

        self.outline_shader.use_program();
        self.outline_shader.set_view(&self.camera);
        self.outline_shader
            .set_matrix_4fv("projectionMatrix", projection);
    }
}
