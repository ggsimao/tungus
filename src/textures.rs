use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use stb_image::stb_image::bindgen::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub enum TextureType {
    Diffuse,
    Specular,
    Attachment,
}

#[derive(Clone, Debug)]
pub struct Texture {
    id: u32,
    ttype: TextureType,
    path: String,
}

impl Texture {
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
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA.0 as i32,
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
}

#[derive(Clone)]
pub struct Material {
    diffuse_maps: Vec<Texture>,
    specular_maps: Vec<Texture>,
    shininess: f32,
}

impl Material {
    pub fn new(diff: Vec<Texture>, spec: Vec<Texture>, shininess: f32) -> Self {
        Material {
            diffuse_maps: diff,
            specular_maps: spec,
            shininess,
        }
    }

    pub fn get_diffuse_maps(&self) -> &Vec<Texture> {
        &self.diffuse_maps
    }

    pub fn get_specular_maps(&self) -> &Vec<Texture> {
        &self.specular_maps
    }

    pub fn get_shininess(&self) -> f32 {
        self.shininess
    }
}
