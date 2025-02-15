use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::rc::Rc;
use std::time::SystemTime;

use crate::camera::Camera;
use crate::controls::{Controller, SignalType, Slot};
use crate::data::{buffer_data, Buffer, BufferType, ShadowFramebuffer, UniformBuffer, VertexArray};
use crate::lighting::Lighting;
use crate::meshes::{BasicMesh, Draw, Skybox, Vertex};
use crate::models::Model;
use crate::shaders::ShaderProgram;
use crate::spatial::Spatial;
use beryllium::Keycode;
use bytemuck::{Pod, Zeroable};
use gl33::gl_enumerations::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

#[derive(Clone)]
#[repr(C)]
pub struct Instance {
    pub model: Mat4,
    pub normal: Mat3,
    pub trans: Mat4,
    pub rot: Mat4,
}

impl Copy for Instance {}
unsafe impl Zeroable for Instance {}
unsafe impl Pod for Instance {}

impl Instance {
    pub fn new() -> Self {
        Instance {
            model: Mat4::identity(),
            normal: Mat3::identity(),
            trans: Mat4::identity(),
            rot: Mat4::identity(),
        }
    }
}

impl Spatial for Instance {
    fn get_model(&self) -> &Mat4 {
        &self.model
    }
    fn get_normal(&mut self) -> &Mat3 {
        self.normal = mat4_to_mat3(&self.get_model().try_inverse().unwrap().transpose());
        &self.normal
    }
    fn set_model(&mut self, model: &Mat4) {
        self.model = *model;
    }
}

pub struct SceneObject {
    drawable: Box<dyn Draw>,
    instances: Vec<Instance>,
    ibo: Buffer,
    model: Mat4,
    normal: Mat3,
    outline: Vec4, // last element indicates whether the object should be outlined
    dirty_instances: bool,
    dirty_normal: bool,
}

impl Clone for SceneObject {
    fn clone(&self) -> Self {
        SceneObject {
            drawable: self.drawable.clone(),
            instances: self.instances.clone(),
            ibo: self.ibo,
            model: self.model.clone(),
            normal: self.normal.clone(),
            outline: self.outline.clone(),
            dirty_instances: self.dirty_instances,
            dirty_normal: self.dirty_normal,
        }
    }
}

impl SceneObject {
    pub fn from<T: Draw + 'static>(object: T) -> Self {
        let obj = SceneObject {
            drawable: Box::new(object),
            instances: vec![Instance::new()],
            ibo: Buffer::new().expect("Couldn't make the instance buffer!"),
            model: Mat4::identity(),
            normal: Mat3::identity(),
            outline: Vec4::zeros(),
            dirty_instances: false,
            dirty_normal: false,
        };
        obj.setup_object();
        obj
    }

    fn setup_object(&self) {
        self.ibo.bind(BufferType::Array);

        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(&self.instances),
            GL_STATIC_DRAW,
        );

        self.drawable.setup_inst_attr();
        Buffer::clear_binding(BufferType::Array);
    }

    pub fn add_instance(&mut self) {
        self.instances.push(Instance::new());
    }

    pub fn add_instances(&mut self, instances: usize) {
        for _ in 0..instances {
            self.instances.push(Instance::new());
        }
    }

    pub fn get_instances(&self) -> usize {
        self.instances.len()
    }

    pub fn get_instance(&self, instance: isize) -> &Instance {
        if instance < 0 {
            let index = self.instances.len() - (-instance as usize);
            &self.instances[index]
        } else {
            &self.instances[instance as usize]
        }
    }

    pub fn get_instance_mut(&mut self, instance: isize) -> &mut Instance {
        self.dirty_instances = true;
        if instance < 0 {
            let index = self.instances.len() - (-instance as usize);
            &mut self.instances[index]
        } else {
            &mut self.instances[instance as usize]
        }
    }

    pub fn get_outline(&self) -> Vec4 {
        self.outline
    }

    pub fn set_outline(&mut self, color: Vec4) {
        self.outline.x = color.x;
        self.outline.y = color.y;
        self.outline.z = color.z;
        self.outline.w = color.w;
    }

    pub fn enable_outline(&mut self, enable: bool) {
        self.outline.w = enable as i32 as f32;
    }

    pub fn has_outline(&self) -> bool {
        self.outline.w > 0.0
    }

    pub fn draw_outline(&self, shader: &ShaderProgram, drawable: &dyn Draw) {
        unsafe {
            glStencilFunc(GL_NOTEQUAL, 1, 0xFF);
            glStencilMask(0x00);
            glDisable(GL_DEPTH_TEST);
        }

        shader.set_3f("outlineColor", &self.outline.xyz());
        drawable.draw(shader);

        unsafe {
            glStencilMask(0xFF);
            glStencilFunc(GL_ALWAYS, 1, 0xFF);
            glEnable(GL_DEPTH_TEST);
        }
    }

    pub fn draw(&self, shader: &ShaderProgram) {
        if self.dirty_instances == true {
            self.ibo.bind(BufferType::Array);
            buffer_data(
                BufferType::Array,
                bytemuck::cast_slice(&self.instances),
                GL_STATIC_DRAW,
            );
        }
        self.drawable.instanced_draw(shader, self.instances.len());
        Buffer::clear_binding(BufferType::Array);
    }
}

impl Spatial for SceneObject {
    fn get_model(&self) -> &Mat4 {
        &self.model
    }
    fn get_normal(&mut self) -> &Mat3 {
        if self.dirty_normal {
            self.normal = mat4_to_mat3(&self.get_model().try_inverse().unwrap().transpose());
        }
        &self.normal
    }
    fn set_model(&mut self, model: &Mat4) {
        self.model = *model;
    }
}

#[derive(Clone, Copy)]
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
    visualize_normals: bool,
}

impl SceneController {
    pub fn new() -> Rc<RefCell<SceneController>> {
        Rc::new(RefCell::new(Self {
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

impl Slot for SceneController {
    fn on_signal(&mut self, signal: SignalType) {
        match signal {
            SignalType::KeyPressed(key) => self.on_key_pressed(key),
            _ => (),
        }
    }
}

impl<'a> Controller<'a, SceneParameters, SceneController> for Rc<RefCell<SceneController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut SceneController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut SceneParameters) {
        let self_obj = (**self).borrow_mut();
        obj.visualize_normals = self_obj.visualize_normals;
    }
}

pub struct Scene<'a> {
    pub objects: Vec<SceneObject>,
    pub skyboxes: &'a Vec<&'a Skybox>,
    pub object_shader: ShaderProgram,
    pub skybox_shader: ShaderProgram,
    pub outline_shader: ShaderProgram,
    pub shadow_shader: ShaderProgram,
    pub debug_shader: ShaderProgram,
    pub camera: Camera,
    pub lighting: &'a Lighting,
    pub params: SceneParameters,
}

impl<'a> Scene<'a> {
    pub fn mirrored(&'a self) -> Self {
        Scene {
            objects: self.objects.clone(),
            skyboxes: &self.skyboxes,
            object_shader: self.object_shader,
            skybox_shader: self.skybox_shader,
            outline_shader: self.outline_shader,
            shadow_shader: self.shadow_shader,
            debug_shader: self.debug_shader,
            camera: self.camera.invert(),
            lighting: &self.lighting,
            params: self.params,
        }
    }
    pub fn compose(&mut self, ubo: &UniformBuffer) {
        let projection = perspective(1.0, self.camera.get_fov(), 0.1, 100.0);
        ubo.set_projection_mat(&projection);

        let skybox_view = mat3_to_mat4(&mat4_to_mat3(&self.camera.look_at()));
        ubo.set_view_mat(&skybox_view);
        self.skybox_shader.use_program();
        self.draw_skybox();

        let view = self.camera.look_at();
        ubo.set_view_mat(&view);

        self.object_shader.use_program();
        self.set_lighting_uniforms();
        self.object_shader.set_3f("viewPos", &self.camera.get_pos());

        self.draw_objects(ubo);
    }

    fn draw_objects(&mut self, ubo: &UniformBuffer) {
        let distance_compare = |a: &SceneObject, b: &SceneObject| {
            let a_pos = a.get_model().column(3).xyz();
            let b_pos = b.get_model().column(3).xyz();
            let distance_a = length(&(self.camera.get_pos() - a_pos));
            let distance_b = length(&(self.camera.get_pos() - b_pos));
            distance_b.partial_cmp(&distance_a).unwrap()
        };
        self.objects.sort_by(distance_compare);
        for object in &self.objects {
            ubo.set_model_mat(&object.get_model());
            object.draw(&self.object_shader);
            if self.params.visualize_normals {
                self.debug_shader.use_program();
                object.draw(&self.debug_shader);
                self.object_shader.use_program();
            }
            if object.has_outline() {
                self.outline_shader.use_program();
                let outline_scale = scale(&object.get_model(), &vec3(1.1, 1.1, 1.1));
                ubo.set_model_mat(&outline_scale);
                object.draw_outline(self.outline_shader.borrow_mut(), object.drawable.as_ref());
                self.object_shader.use_program();
            }
        }
    }

    fn draw_skybox(&mut self) {
        unsafe {
            glDisable(GL_STENCIL_TEST);
            glDisable(GL_CULL_FACE);
            glDepthFunc(GL_LEQUAL);
        }

        for skybox in self.skyboxes {
            skybox.draw(&self.skybox_shader);
        }

        unsafe {
            glEnable(GL_STENCIL_TEST);
            glEnable(GL_CULL_FACE);
            glDepthFunc(GL_LESS);
        }
    }

    pub fn set_shadow_maps(&mut self, ubo: &UniformBuffer, sfbo: &ShadowFramebuffer) {
        unsafe {
            glViewport(
                0,
                0,
                sfbo.get_window_size().0 as i32,
                sfbo.get_window_size().1 as i32,
            );
            glCullFace(GL_FRONT);
        }
        // directional
        // unsafe {
        //     glClear(GL_DEPTH_BUFFER_BIT);
        // }
        let (near_plane, far_plane): (f32, f32) = (-2.0, 10.0);
        let dir_projection = ortho(-10.0, 10.0, -10.0, 10.0, near_plane, far_plane);
        let directional_pos = -self.lighting.dir.dir;
        let dir_view = look_at(&directional_pos, &Vec3::zeros(), &vec3(0.0, 1.0, 0.0));
        self.set_shadow_map("dirLight", &dir_projection, &dir_view, ubo, sfbo);

        // spotlight
        // unsafe {
        //     glClear(GL_DEPTH_BUFFER_BIT);
        // }
        // let spot_projection =
        //     perspective(1.0, self.lighting.spot.phi.to_radians() / 2.0, 0.1, 100.0);
        // let spot_pos = self.lighting.spot.pos;
        // let spot_dir = self.lighting.spot.pos + self.lighting.spot.dir;
        // let spot_view = look_at(&spot_pos, &spot_dir, &vec3(0.0, 1.0, 0.0));
        // self.set_shadow_map("spotlight", &spot_projection, &spot_view, ubo, sfbo);

        unsafe {
            glCullFace(GL_BACK);
        }
    }

    fn set_shadow_map(
        &mut self,
        light_name: &str,
        projection: &Mat4,
        view: &Mat4,
        ubo: &UniformBuffer,
        sfbo: &ShadowFramebuffer,
    ) {
        ubo.set_projection_mat(&projection);
        ubo.set_view_mat(&view);

        self.shadow_shader.use_program();
        self.draw_shadows(ubo);

        self.object_shader.use_program();
        self.object_shader
            .set_matrix_4fv(&format!("{}SpaceMatrix", light_name), &(projection * view));
        self.object_shader
            .set_texture2D(&format!("{}.shadow_map", light_name), sfbo.get_texture());
    }

    fn draw_shadows(&mut self, ubo: &UniformBuffer) {
        for object in &self.objects {
            ubo.set_model_mat(&object.get_model());
            object.draw(&self.shadow_shader);
        }
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
