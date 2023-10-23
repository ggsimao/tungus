use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::vec3;
use nalgebra_glm::*;
use std::ffi::CString;
use std::path::Path;

use crate::camera::Camera;
use crate::helpers;
use crate::lighting::DirectionalLight;
use crate::lighting::PointLight;
use crate::lighting::Spotlight;
use crate::textures::{Material, Texture};

pub struct Shader(pub u32);

impl Shader {
    pub fn new(ty: ShaderType) -> Option<Self> {
        let shader = glCreateShader(GLenum(ty as u32));
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            glShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self) {
        glCompileShader(self.0);
    }

    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { glGetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == GL_TRUE.0 as i32
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn delete(self) {
        glDeleteShader(self.0);
    }

    pub fn from_source(ty: ShaderType, path: &Path) -> Result<Self, String> {
        let source = helpers::read_from_file(path);
        let id = Self::new(ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(&source[..]);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}

pub enum ShaderType {
    VertexShader = GL_VERTEX_SHADER.0 as isize,
    FragmentShader = GL_FRAGMENT_SHADER.0 as isize,
}

pub struct ShaderProgram(pub u32);
impl ShaderProgram {
    pub fn new() -> Option<Self> {
        let prog = glCreateProgram();
        if prog != 0 {
            Some(Self(prog))
        } else {
            None
        }
    }

    pub fn attach_shader(&self, shader: &Shader) {
        glAttachShader(self.0, shader.0);
    }

    pub fn link_program(&self) {
        glLinkProgram(self.0);
    }

    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { glGetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == GL_TRUE.0 as i32
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetProgramInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn use_program(&self) {
        glUseProgram(self.0);
    }

    pub fn delete(self) {
        glDeleteProgram(self.0);
    }

    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(ShaderType::VertexShader, &Path::new(vert))
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::FragmentShader, &Path::new(frag))
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&f);
        p.link_program();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }

    fn get_uniform_location(&self, name: &str) -> i32 {
        unsafe {
            let uniform_name = CString::new(name.as_bytes()).unwrap().into_raw() as *const u8;
            let location: i32 = glGetUniformLocation(self.0, uniform_name);
            location
        }
    }

    pub fn set_1b(&self, name: &str, value: bool) {
        let location = self.get_uniform_location(name);
        unsafe { glUniform1i(location, value.into()) }
    }
    pub fn set_1i(&self, name: &str, value: i32) {
        let location = self.get_uniform_location(name);
        unsafe { glUniform1i(location, value) }
    }
    pub fn set_1f(&self, name: &str, value: f32) {
        let location = self.get_uniform_location(name);
        unsafe { glUniform1f(location, value) }
    }
    pub fn set_4f(&self, name: &str, value: &Vec4) {
        let location = self.get_uniform_location(name);
        unsafe { glUniform4f(location, value.x, value.y, value.z, value.w) }
    }
    pub fn set_3f(&self, name: &str, value: &Vec3) {
        let location = self.get_uniform_location(name);
        unsafe { glUniform3f(location, value.x, value.y, value.z) }
    }
    pub fn set_matrix_4fv(&self, name: &str, value: &Mat4) {
        let location = self.get_uniform_location(name);
        unsafe { glUniformMatrix4fv(location, 1, 0, value.as_ptr()) }
    }
    pub fn set_matrix_3fv(&self, name: &str, value: &Mat3) {
        let location = self.get_uniform_location(name);
        unsafe { glUniformMatrix3fv(location, 1, 0, value.as_ptr()) }
    }
    pub fn set_texture(&self, texture_name: &str, value: &Texture) {
        unsafe {
            glActiveTexture(GLenum(GL_TEXTURE0.0 as u32));
        }
        value.bind();
        self.set_1i(texture_name, 0 as i32);
    }
    pub fn set_material(&self, material_name: &str, value: &Material) {
        let diffuse_vector = value.get_diffuse_maps();
        let specular_vector = value.get_specular_maps();
        let mut tex_count = 0;
        for (i, diffuse) in diffuse_vector.iter().enumerate() {
            unsafe {
                glActiveTexture(GLenum(GL_TEXTURE0.0 + tex_count as u32));
            }
            diffuse.bind();
            let name = format!("{}.Diffuse[{}]", material_name, i);
            self.set_1i(&name, i as i32);
            tex_count += 1;
        }
        for (i, specular) in specular_vector.iter().enumerate() {
            unsafe {
                glActiveTexture(GLenum(GL_TEXTURE0.0 + tex_count as u32));
            }
            specular.bind();
            let name = format!("{}.Specular[{}]", material_name, i);
            self.set_1i(&name, i as i32);
            tex_count += 1;
        }
        self.set_1f(
            &format!("{}.shininess", material_name),
            value.get_shininess(),
        );
        self.set_1i(
            &format!("{}.loadedDiffuse", material_name),
            diffuse_vector.len().max(1) as i32,
        );
        self.set_1i(
            &format!("{}.loadedSpecular", material_name),
            specular_vector.len().max(1) as i32,
        );
    }
    pub fn set_directional_light(&self, name: &str, value: &DirectionalLight) {
        self.set_3f(format!("{}.direction", name).as_str(), &value.dir);
        self.set_3f(format!("{}.ambient", name).as_str(), &value.amb);
        self.set_3f(format!("{}.diffuse", name).as_str(), &value.diff);
        self.set_3f(format!("{}.specular", name).as_str(), &value.spec);
    }
    pub fn set_point_light(&self, name: &str, value: &PointLight) {
        self.set_3f(format!("{}.position", name).as_str(), &value.pos);
        self.set_1f(format!("{}.constant", name).as_str(), value.att.x);
        self.set_1f(format!("{}.linear", name).as_str(), value.att.y);
        self.set_1f(format!("{}.quadratic", name).as_str(), value.att.z);
        self.set_3f(format!("{}.ambient", name).as_str(), &value.amb);
        self.set_3f(format!("{}.diffuse", name).as_str(), &value.diff);
        self.set_3f(format!("{}.specular", name).as_str(), &value.spec);
    }
    pub fn set_spotlight(&self, name: &str, value: &Spotlight) {
        self.set_3f(format!("{}.position", name).as_str(), &value.pos);
        self.set_3f(format!("{}.direction", name).as_str(), &value.dir);
        self.set_1f(format!("{}.constant", name).as_str(), value.att.x);
        self.set_1f(format!("{}.linear", name).as_str(), value.att.y);
        self.set_1f(format!("{}.quadratic", name).as_str(), value.att.z);
        self.set_3f(format!("{}.ambient", name).as_str(), &value.amb);
        self.set_3f(format!("{}.diffuse", name).as_str(), &value.diff);
        self.set_3f(format!("{}.specular", name).as_str(), &value.spec);
        self.set_1f(format!("{}.phiCos", name).as_str(), value.phi.cos());
        self.set_1f(format!("{}.gammaCos", name).as_str(), value.gamma.cos());
    }
    pub fn set_view(&self, camera: &Camera) {
        self.set_matrix_4fv("viewMatrix", &camera.look_at());
        self.set_3f("viewPos", &camera.get_pos());
    }
}
