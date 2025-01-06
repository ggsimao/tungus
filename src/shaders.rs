use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::vec3;
use nalgebra_glm::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::path::Path;

use crate::camera::Camera;
use crate::data::UniformBuffer;
use crate::helpers;
use crate::lighting::DirectionalLight;
use crate::lighting::PointLight;
use crate::lighting::Spotlight;
use crate::textures::CubeMap;
use crate::textures::Texture2DMultisample;
use crate::textures::{Material, Texture2D};
use crate::utils;

#[derive(Clone, Copy)]
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
        let obj = Self::new(ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        obj.set_source(&source[..]);
        obj.compile();
        if obj.compile_success() {
            Ok(obj)
        } else {
            let out = obj.info_log();
            obj.delete();
            Err(out)
        }
    }
}

pub enum ShaderType {
    VertexShader = GL_VERTEX_SHADER.0 as isize,
    GeometryShader = GL_GEOMETRY_SHADER.0 as isize,
    FragmentShader = GL_FRAGMENT_SHADER.0 as isize,
}

static mut TEX_COUNT: u32 = 0;

#[derive(Clone, Copy)]
pub struct ShaderProgram(u32);
impl ShaderProgram {
    pub fn new() -> Option<Self> {
        let prog = glCreateProgram();
        if prog != 0 {
            Some(Self(prog))
        } else {
            None
        }
    }

    #[inline]
    pub fn tex_count() -> u32 {
        unsafe { TEX_COUNT }
    }

    #[inline]
    pub fn increment_tex_count() -> () {
        unsafe {
            TEX_COUNT += 1;
        }
    }

    #[inline]
    pub fn reset_tex_count() -> () {
        unsafe {
            TEX_COUNT = 0;
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

    pub fn from_vert_geo_frag(vert: &str, geo: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(ShaderType::VertexShader, &Path::new(vert))
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let g = Shader::from_source(ShaderType::GeometryShader, &Path::new(geo))
            .map_err(|e| format!("Geometry Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::FragmentShader, &Path::new(frag))
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&g);
        p.attach_shader(&f);
        p.link_program();
        v.delete();
        g.delete();
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
        let uniform_name = CString::new(name.as_bytes()).unwrap().into_raw() as *const u8;
        let location: i32;
        unsafe {
            location = glGetUniformLocation(self.0, uniform_name);
        }
        // if location == -1 {
        //     println!("Uniform {} not found for shader program {}", name, self.0);
        // }
        location
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
    #[allow(non_snake_case)]
    pub fn set_texture2D(&self, texture_name: &str, value: &Texture2D) {
        unsafe {
            glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
        }
        value.bind();
        self.set_1i(texture_name, Self::tex_count() as i32);
        Self::increment_tex_count();
    }
    #[allow(non_snake_case)]
    pub fn set_texture2D_multisample(&self, texture_name: &str, value: &Texture2DMultisample) {
        unsafe {
            glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
        }
        value.bind();
        self.set_1i(texture_name, Self::tex_count() as i32);
        Self::increment_tex_count();
    }
    pub fn set_cubemap(&self, texture_name: &str, value: &CubeMap) {
        unsafe {
            glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
        }
        value.bind();
        self.set_1i(texture_name, Self::tex_count() as i32);
        Self::increment_tex_count();
    }
    pub fn set_material(&self, material_name: &str, value: &Material) {
        let diffuse_vector = value.get_diffuse_maps();
        let specular_vector = value.get_specular_maps();
        let loaded_diffuse = diffuse_vector.len().max(1) as i32;
        let loaded_specular = specular_vector.len().max(1) as i32;

        if diffuse_vector.len() == 0 {
            unsafe {
                glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
            }
            let diff = Texture2D::new(crate::textures::TextureType::Diffuse);
            diff.empty_texture();
            diff.bind();
            let name = format!("{}.diffuseTextures[0]", material_name);
            self.set_1i(&name, Self::tex_count() as i32);
            Self::increment_tex_count();
        } else {
            for (i, diffuse) in diffuse_vector.iter().enumerate() {
                unsafe {
                    glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
                }
                diffuse.bind();
                let name = format!("{}.diffuseTextures[{}]", material_name, i);
                self.set_1i(&name, Self::tex_count() as i32);
                Self::increment_tex_count();
            }
        }
        if specular_vector.len() == 0 {
            unsafe {
                glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
            }
            let spec = Texture2D::new(crate::textures::TextureType::Specular);
            spec.empty_texture();
            spec.bind();
            let name = format!("{}.specularTextures[0]", material_name);
            self.set_1i(&name, Self::tex_count() as i32);
            Self::increment_tex_count();
        } else {
            for (i, specular) in specular_vector.iter().enumerate() {
                unsafe {
                    glActiveTexture(GLenum(GL_TEXTURE0.0 + Self::tex_count()));
                }
                specular.bind();
                let name = format!("{}.specularTextures[{}]", material_name, i);
                self.set_1i(&name, Self::tex_count() as i32);
                Self::increment_tex_count();
            }
        }

        self.set_1f(
            &format!("{}.shininess", material_name),
            value.get_shininess(),
        );
        self.set_1i(&format!("{}.loadedDiffuse", material_name), loaded_diffuse);
        self.set_1i(
            &format!("{}.loadedSpecular", material_name),
            loaded_specular,
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
        self.set_3f(format!("{}.ambient", name).as_str(), &value.get_amb());
        self.set_3f(format!("{}.diffuse", name).as_str(), &value.get_diff());
        self.set_3f(format!("{}.specular", name).as_str(), &value.get_spec());
        self.set_1f(format!("{}.phiCos", name).as_str(), value.phi.cos());
        self.set_1f(format!("{}.gammaCos", name).as_str(), value.gamma.cos());
    }
}
