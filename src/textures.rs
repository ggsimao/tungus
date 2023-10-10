use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use stb_image::stb_image::bindgen::*;
use std::ffi::c_void;
use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

pub struct Texture(pub u32);

impl Texture {
    pub fn new() -> Self {
        let mut texture: u32 = 0;
        unsafe {
            glGenTextures(1, &mut texture);
        }
        Self(texture)
    }
    pub fn load(&mut self, path: &Path) {
        let (mut width, mut height, mut nr_channels): (i32, i32, i32) = (0, 0, 0);
        let path_string = CString::new(path.as_os_str().as_bytes()).unwrap();
        let format = if path.extension().unwrap() == "png" {
            GL_RGBA
        } else {
            GL_RGB
        };
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.0);
            stbi_set_flip_vertically_on_load(1);
            let data = stbi_load(
                path_string.as_ptr(),
                &mut width,
                &mut height,
                &mut nr_channels,
                0,
            );
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGB.0 as i32,
                width,
                height,
                0,
                format,
                GL_UNSIGNED_BYTE,
                data as *const c_void,
            );
            glGenerateMipmap(GL_TEXTURE_2D);
            stbi_image_free(data as *mut c_void);
        }
    }

    pub fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.0);
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
}

pub struct Material {
    diffuse: Texture,
    specular: Texture,
    shininess: f32,
}

impl Material {
    pub fn new(diff: Texture, spec: Texture, shininess: f32) -> Self {
        Material {
            diffuse: diff,
            specular: spec,
            shininess,
        }
    }

    pub fn get_diffuse(&self) -> &Texture {
        &self.diffuse
    }

    pub fn get_specular(&self) -> &Texture {
        &self.specular
    }

    pub fn get_shininess(&self) -> f32 {
        self.shininess
    }
}
