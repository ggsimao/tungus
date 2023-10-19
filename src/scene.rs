use crate::meshes::{Draw, Mesh};
use crate::models::Model;
use crate::shaders::ShaderProgram;
use gl33::gl_enumerations::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

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
        // shader.set_matrix_3fv("normalMatrix", &self.normal);
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
}
