use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;
use stb_image::stb_image::bindgen::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

const EMPTY_DATA: [u8; 4] = [0; 4];

#[derive(Copy, Clone, Debug)]
pub enum TextureType {
    Diffuse,
    Specular,
    Attachment,
}

#[derive(Debug, Clone)]
pub struct Texture2D {
    id: u32,
    ttype: TextureType,
    path: String,
}

impl Texture2D {
    pub fn new(ttype: TextureType) -> Self {
        let mut texture: u32 = 0;
        unsafe {
            glGenTextures(1, &mut texture);
        }
        Self {
            id: texture,
            ttype,
            path: String::new(),
        }
    }
    pub fn load(&mut self, path: &Path) {
        let (mut width, mut height, mut nr_channels): (i32, i32, i32) = (0, 0, 0);
        let path_string = CString::new(path.as_os_str().as_bytes()).unwrap();
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
            stbi_set_flip_vertically_on_load(1);
            let data = stbi_load(
                path_string.as_ptr(),
                &mut width,
                &mut height,
                &mut nr_channels,
                0,
            );
            let format = match nr_channels {
                4 => GL_RGBA,
                _ => GL_RGB,
            };
            let i_format = self.get_internal_format();
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                i_format.0 as i32,
                width,
                height,
                0,
                format,
                GL_UNSIGNED_BYTE,
                data as *const c_void,
            );
            glGenerateMipmap(GL_TEXTURE_2D);
            stbi_image_free(data as *mut c_void);
            glBindTexture(GL_TEXTURE_2D, 0);
        }
        self.path = path.display().to_string();
    }
    pub fn empty_texture(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA8.0 as i32,
                1,
                1,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                EMPTY_DATA.as_ptr() as *const c_void,
            );
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }
    pub fn from_color(&self, color: &Vec3) {
        let data: [u8; 4] = [
            (color.x * 255.0) as u8,
            (color.y * 255.0) as u8,
            (color.z * 255.0) as u8,
            255,
        ];
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA.0 as i32,
                1,
                1,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }

    pub fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
        }
    }

    pub fn clear_binding() {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }

    pub fn set_filters(&self, min_param: GLenum, mag_param: GLenum) {
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, min_param.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, mag_param.0 as i32);
        }
    }

    pub fn set_wrapping(&self, wrapping: GLenum) {
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrapping.0 as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrapping.0 as i32);
        }
    }

    pub fn set_wrapping_on_axis(&self, axis: GLenum, wrapping: GLenum) {
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, axis, wrapping.0 as i32);
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_type(&self) -> TextureType {
        self.ttype
    }
    pub fn get_internal_format(&self) -> GLenum {
        match self.ttype {
            TextureType::Diffuse => GL_SRGB_ALPHA,
            TextureType::Specular => GL_RGBA,
            TextureType::Attachment => GL_RGBA,
        }
    }

    pub fn setup_new(ttype: TextureType, path: &Path, wrapping: GLenum) -> Self {
        let mut tex = Texture2D::new(ttype);
        tex.load(&Path::new(path));
        tex.set_wrapping(wrapping);
        return tex;
    }
}

#[derive(Clone, Debug)]
pub struct CubeMap {
    id: u32,
    ttype: TextureType,
}

impl CubeMap {
    pub fn new(ttype: TextureType) -> Self {
        let mut texture: u32 = 0;
        unsafe {
            glGenTextures(1, &mut texture);
        }
        Self { id: texture, ttype }
    }
    pub fn load(&mut self, paths: [&str; 6]) {
        unsafe {
            glBindTexture(GL_TEXTURE_CUBE_MAP, self.id);
        }
        let (mut width, mut height, mut nr_channels): (i32, i32, i32) = (0, 0, 0);
        for i in 0..6 {
            let path_string = CString::new(paths[i]).unwrap();
            unsafe {
                // stbi_set_flip_vertically_on_load(1);
                let data = stbi_load(
                    path_string.as_ptr(),
                    &mut width,
                    &mut height,
                    &mut nr_channels,
                    0,
                );
                let format = match nr_channels {
                    4 => GL_RGBA,
                    _ => GL_RGB,
                };
                glTexImage2D(
                    GLenum(GL_TEXTURE_CUBE_MAP_POSITIVE_X.0 + i as u32),
                    0,
                    GL_SRGB.0 as i32,
                    width,
                    height,
                    0,
                    format,
                    GL_UNSIGNED_BYTE,
                    data as *const c_void,
                );
                glGenerateMipmap(GL_TEXTURE_CUBE_MAP);
                stbi_image_free(data as *mut c_void);
            }
        }
        unsafe {
            glBindTexture(GL_TEXTURE_CUBE_MAP, 0);
        }
    }

    pub fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_CUBE_MAP, self.id);
        }
    }

    pub fn clear_binding() {
        unsafe {
            glBindTexture(GL_TEXTURE_CUBE_MAP, 0);
        }
    }

    pub fn set_filters(&self, min_param: GLenum, mag_param: GLenum) {
        unsafe {
            glTexParameteri(
                GL_TEXTURE_CUBE_MAP,
                GL_TEXTURE_MIN_FILTER,
                min_param.0 as i32,
            );
            glTexParameteri(
                GL_TEXTURE_CUBE_MAP,
                GL_TEXTURE_MAG_FILTER,
                mag_param.0 as i32,
            );
        }
    }

    pub fn set_wrapping(&self, wrapping: GLenum) {
        unsafe {
            glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, wrapping.0 as i32);
            glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, wrapping.0 as i32);
            glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, wrapping.0 as i32);
        }
    }

    pub fn set_wrapping_on_axis(&self, axis: GLenum, wrapping: GLenum) {
        unsafe {
            glTexParameteri(GL_TEXTURE_CUBE_MAP, axis, wrapping.0 as i32);
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_type(&self) -> TextureType {
        self.ttype
    }
}

#[derive(Clone)]
pub struct Material {
    diffuse_maps: Vec<Texture2D>,
    specular_maps: Vec<Texture2D>,
    shininess: f32,
}

impl Material {
    pub fn new(diff: Vec<Texture2D>, spec: Vec<Texture2D>, shininess: f32) -> Self {
        Material {
            diffuse_maps: diff,
            specular_maps: spec,
            shininess,
        }
    }

    pub fn get_diffuse_maps(&self) -> &Vec<Texture2D> {
        &self.diffuse_maps
    }

    pub fn get_specular_maps(&self) -> &Vec<Texture2D> {
        &self.specular_maps
    }

    pub fn get_shininess(&self) -> f32 {
        self.shininess
    }
}

#[derive(Debug, Clone)]
pub struct Texture2DMultisample {
    id: u32,
    samples: u32,
}

impl Texture2DMultisample {
    pub fn new(samples: u32) -> Self {
        let mut texture: u32 = 0;
        unsafe {
            glGenTextures(1, &mut texture);
        }
        Self {
            id: texture,
            samples,
        }
    }
    pub fn create_texture(&self, size: (u32, u32)) {
        self.bind();
        unsafe {
            glTexImage2DMultisample(
                GL_TEXTURE_2D_MULTISAMPLE,
                self.samples as i32,
                GL_RGB,
                size.0 as i32,
                size.1 as i32,
                GL_TRUE.0 as u8,
            );
        }
        Self::clear_binding();
    }

    pub fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D_MULTISAMPLE, self.id);
        }
    }

    pub fn clear_binding() {
        unsafe {
            glBindTexture(GL_TEXTURE_2D_MULTISAMPLE, 0);
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_samples(&self) -> u32 {
        self.samples
    }
}
