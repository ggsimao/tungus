use std::ptr::null;

use beryllium::GlWindow;
use bytemuck::offset_of;
use bytemuck::{NoUninit, Pod, Zeroable};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

use crate::meshes::Vertex;
use crate::textures::{Texture2D, TextureType};

// I really don't like the way this file is right now.

#[derive(Clone, Copy)]
pub struct VertexArray(pub u32);
impl VertexArray {
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { glGenVertexArrays(1, &mut vao) };
        if vao != 0 {
            Some(Self(vao))
        } else {
            None
        }
    }

    pub fn bind(&self) {
        glBindVertexArray(self.0)
    }

    pub fn clear_binding() {
        glBindVertexArray(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Array = GL_ARRAY_BUFFER.0 as isize,
    ElementArray = GL_ELEMENT_ARRAY_BUFFER.0 as isize,
}

#[derive(Clone, Copy)]
pub struct Buffer(pub u32);
impl Buffer {
    pub fn new() -> Option<Self> {
        let mut bo = 0;
        unsafe {
            glGenBuffers(1, &mut bo);
        }
        if bo != 0 {
            Some(Self(bo))
        } else {
            None
        }
    }

    pub fn bind(&self, ty: BufferType) {
        unsafe { glBindBuffer(GLenum(ty as u32), self.0) }
    }

    pub fn clear_binding(ty: BufferType) {
        unsafe { glBindBuffer(GLenum(ty as u32), 0) }
    }
}

pub fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        glBufferData(
            GLenum(ty as u32),
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
    Point = GL_POINT.0 as isize,
    Line = GL_LINE.0 as isize,
    Fill = GL_FILL.0 as isize,
}

pub fn polygon_mode(mode: PolygonMode) {
    unsafe { glPolygonMode(GL_FRONT, GLenum(mode as u32)) };
}

#[derive(Debug)]
pub struct Framebuffer {
    id: u32,
    texture: Texture2D,
    rbo: Renderbuffer,
}

impl Framebuffer {
    pub fn new() -> Option<Self> {
        let mut fbo = 0;
        let texture = Texture2D::new(TextureType::Attachment);
        let rbo = Renderbuffer::new().unwrap();
        unsafe {
            glGenFramebuffers(1, &mut fbo);
        }
        if fbo != 0 {
            Some(Self {
                id: fbo,
                texture,
                rbo,
            })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn check_status() -> GLenum {
        unsafe { glCheckFramebufferStatus(GL_FRAMEBUFFER) }
    }

    pub fn setup_with_renderbuffer(&self, window_size: (u32, u32)) {
        self.bind();
        self.attach_texture(window_size);
        self.attach_renderbuffer(window_size);
        Self::clear_binding();
    }

    // If you want to render your whole screen to a texture of a smaller or larger size you need to
    // call glViewport again (before rendering to your framebuffer) with the new dimensions
    // of your texture, otherwise render commands will only fill part of the texture.
    pub fn attach_texture(&self, window_size: (u32, u32)) {
        self.texture.bind();
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGB.0 as i32,
                window_size.0 as i32,
                window_size.1 as i32,
                0,
                GL_RGB,
                GL_UNSIGNED_BYTE,
                null(),
            );
        }
        self.texture.set_filters(GL_LINEAR, GL_LINEAR);
        Texture2D::clear_binding();

        unsafe {
            glFramebufferTexture2D(
                GL_FRAMEBUFFER,
                GL_COLOR_ATTACHMENT0,
                GL_TEXTURE_2D,
                self.texture.get_id(),
                0,
            );
        }
    }

    pub fn attach_depth_stencil(&self, window_size: (u32, u32)) {
        self.texture.bind();
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_DEPTH24_STENCIL8.0 as i32,
                window_size.0 as i32,
                window_size.1 as i32,
                0,
                GL_DEPTH_STENCIL,
                GL_UNSIGNED_INT_24_8,
                null(),
            );
        }
        Texture2D::clear_binding();
        unsafe {
            glFramebufferTexture2D(
                GL_FRAMEBUFFER,
                GL_DEPTH_STENCIL_ATTACHMENT,
                GL_TEXTURE_2D,
                self.texture.get_id(),
                0,
            );
        }
    }

    pub fn attach_renderbuffer(&self, window_size: (u32, u32)) {
        self.rbo.bind();
        Renderbuffer::create_depth_stencil_storage(window_size);
        Renderbuffer::clear_binding();
        unsafe {
            glFramebufferRenderbuffer(
                GL_FRAMEBUFFER,
                GL_DEPTH_STENCIL_ATTACHMENT,
                GL_RENDERBUFFER,
                self.rbo.get_id(),
            );
        }
        if Self::check_status() != GL_FRAMEBUFFER_COMPLETE {
            panic!("Could not complete framebuffer!")
        }
    }

    pub fn bind(&self) {
        unsafe { glBindFramebuffer(GL_FRAMEBUFFER, self.id) }
    }

    pub fn clear_binding() {
        unsafe { glBindFramebuffer(GL_FRAMEBUFFER, 0) }
    }

    pub fn get_texture(&self) -> &Texture2D {
        &self.texture
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            glDeleteFramebuffers(1, &self.id);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Renderbuffer {
    id: u32,
}

impl Renderbuffer {
    pub fn new() -> Option<Self> {
        let mut rbo = 0;
        unsafe {
            glGenRenderbuffers(1, &mut rbo);
        }
        if rbo != 0 {
            Some(Self { id: rbo })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn bind(&self) {
        unsafe { glBindRenderbuffer(GL_RENDERBUFFER, self.id) }
    }

    pub fn clear_binding() {
        unsafe { glBindRenderbuffer(GL_RENDERBUFFER, 0) }
    }

    pub fn create_depth_stencil_storage(window_size: (u32, u32)) {
        unsafe {
            glRenderbufferStorage(
                GL_RENDERBUFFER,
                GL_DEPTH24_STENCIL8,
                window_size.0 as i32,
                window_size.1 as i32,
            );
        }
    }
}

#[derive(Clone, Copy)]
pub struct UniformBuffer {
    id: u32,
    binding: u32,
}

impl UniformBuffer {
    pub fn new(binding: u32) -> Option<Self> {
        let mut ubo = 0;
        unsafe {
            glGenBuffers(1, &mut ubo);
        }
        if ubo != 0 {
            Some(Self { id: ubo, binding })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn bind(&self) {
        unsafe { glBindBuffer(GL_UNIFORM_BUFFER, self.id) }
    }

    pub fn clear_binding() {
        unsafe { glBindBuffer(GL_UNIFORM_BUFFER, 0) }
    }

    pub fn allocate(&self, size: isize) {
        self.bind();
        unsafe {
            glBufferData(GL_UNIFORM_BUFFER, size, null(), GL_STATIC_DRAW);
        }
        Self::clear_binding();
    }

    pub fn bind_base(&self) {
        unsafe {
            glBindBufferBase(GL_UNIFORM_BUFFER, self.binding, self.id);
        }
    }

    pub fn set_model_mat(&self, model: &Mat4) {
        self.bind();
        unsafe {
            glBufferSubData(GL_UNIFORM_BUFFER, 0, 64, model.as_ptr().cast());
        }
        Self::clear_binding();
    }
    pub fn set_view_mat(&self, view: &Mat4) {
        self.bind();
        unsafe {
            glBufferSubData(GL_UNIFORM_BUFFER, 64, 64, view.as_ptr().cast());
        }
        Self::clear_binding();
    }
    pub fn set_projection_mat(&self, proj: &Mat4) {
        self.bind();
        unsafe {
            glBufferSubData(GL_UNIFORM_BUFFER, 128, 64, proj.as_ptr().cast());
        }
        Self::clear_binding();
    }
    pub fn set_normal_mat(&self, normal: &Mat3) {
        self.bind();
        unsafe {
            glBufferSubData(GL_UNIFORM_BUFFER, 192, 48, normal.as_ptr().cast());
        }
        Self::clear_binding();
    }
}
