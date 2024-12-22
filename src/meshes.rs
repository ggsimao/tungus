use bytemuck::{NoUninit, Pod, Zeroable};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

use crate::data::buffer_data;
use crate::scene::Instance;
use crate::shaders::Shader;
use crate::shaders::ShaderProgram;
use crate::textures::Material;
use crate::textures::TextureType;
use crate::{
    data::{Buffer, BufferType, VertexArray},
    textures::{CubeMap, Texture2D},
};

pub trait Draw {
    fn draw(&self, shader: &ShaderProgram);
    fn clone_box(&self) -> Box<dyn Draw>;
    fn instanced_draw(&self, shader: &ShaderProgram, instances: usize);
    fn setup_inst_attr(&self);
    fn cull_faces(&self) -> bool {
        false
    }
}

impl Clone for Box<dyn Draw> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec3,
}

impl Vertex {
    pub fn new(posx: f32, posy: f32, posz: f32) -> Self {
        Vertex {
            pos: vec3(posx, posy, posz),
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
        }
    }
    pub fn from_vector(pos: Vec3) -> Self {
        Vertex {
            pos,
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        self.pos = self.pos + vec3(offset_x, offset_y, offset_z);
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        let matrix = rotation(angle, axis);
        self.pos = vec4_to_vec3(&(matrix * vec3_to_vec4(&self.pos)));
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

#[derive(Clone)]
pub struct BasicMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    cull_faces: bool,
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl BasicMesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, material: Material) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mesh = BasicMesh {
            vertices,
            indices,
            material,
            cull_faces: true,
            vao,
            vbo,
            ebo,
        };
        mesh.setup_mesh();
        mesh
    }

    pub fn cube(side: f32) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mut vertices = vec![
            Vertex::new(-side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, side / 2.0),
        ];

        let indices = vec![
            0, 2, 1, 1, 2, 3, 4, 5, 6, 6, 5, 7, 8, 10, 9, 9, 10, 11, 12, 14, 13, 13, 14, 15, 16,
            18, 17, 17, 18, 19, 20, 22, 21, 21, 22, 23,
        ];
        let mut normals = [Vec3::zeros(); 24];

        for i in 0..6 {
            let main_vertex = vertices[indices[i * 6] as usize];
            let v1 = vertices[indices[i * 6 + 1] as usize];
            let v2 = vertices[indices[i * 6 + 2] as usize];
            let normal = normalize(&cross(
                &(v1.pos - main_vertex.pos),
                &(v2.pos - main_vertex.pos),
            ));
            normals[indices[i * 6] as usize] += normal;
            normals[indices[i * 6 + 1] as usize] += normal;
            normals[indices[i * 6 + 2] as usize] += normal;

            let opposite_vertex = vertices[indices[i * 6 + 5] as usize];
            let normal = normalize(&cross(
                &(v2.pos - opposite_vertex.pos),
                &(v1.pos - opposite_vertex.pos),
            ));
            normals[indices[i * 6 + 5] as usize] += normal;
        }
        for i in 0..24 {
            vertices[i].normal = normals[i] / 4.0;
            vertices[i].tex_coords = vec3((i % 2) as f32, ((i / 2) % 2) as f32, 0.0);
        }
        let cube = BasicMesh {
            vertices,
            indices,
            material: Material::new(vec![], vec![], 1.0),
            cull_faces: true,
            vao,
            vbo,
            ebo,
        };
        cube.setup_mesh();
        cube
    }

    pub fn square(side: f32) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mut vertices = vec![
            Vertex::new(-side / 2.0, side / 2.0, 0.0),
            Vertex::new(side / 2.0, side / 2.0, 0.0),
            Vertex::new(-side / 2.0, -side / 2.0, 0.0),
            Vertex::new(side / 2.0, -side / 2.0, 0.0),
        ];
        let indices = vec![0, 2, 1, 1, 2, 3];

        let main_vertex = vertices[indices[0] as usize];
        let v1 = vertices[indices[1] as usize];
        let v2 = vertices[indices[2] as usize];
        let normal = normalize(&cross(
            &(v1.pos - main_vertex.pos),
            &(v2.pos - main_vertex.pos),
        ));
        for i in 0..4 {
            vertices[i].normal = normal;
            vertices[i].tex_coords = vec3((i % 2) as f32, (i as i32 / -2 + 1) as f32, 0.0);
        }
        let square = BasicMesh {
            vertices,
            indices,
            material: Material::new(vec![], vec![], 1.0),
            cull_faces: false,
            vao,
            vbo,
            ebo,
        };
        square.setup_mesh();
        square
    }

    fn setup_mesh(&self) {
        self.vao.bind();

        self.vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(&self.vertices),
            GL_STATIC_DRAW,
        );

        self.ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&self.indices),
            GL_STATIC_DRAW,
        );

        unsafe {
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, pos) as *const _, // might seem redundant, but it's just in case the order changes
            );
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, normal) as *const _,
            );
            glEnableVertexAttribArray(2);
            glVertexAttribPointer(
                2,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, tex_coords) as *const _,
            );
        }
    }
}

impl Draw for BasicMesh {
    fn draw(&self, shader: &ShaderProgram) {
        shader.set_material("material", &self.material);
        self.vao.bind();
        unsafe {
            glDrawElements(
                GL_TRIANGLES,
                self.indices.len() as i32,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        VertexArray::clear_binding();
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(self.clone())
    }
    fn instanced_draw(&self, shader: &ShaderProgram, instances: usize) {
        shader.set_material("material", &self.material);
        self.vao.bind();
        unsafe {
            glDrawElementsInstanced(
                GL_TRIANGLES,
                self.indices.len() as i32,
                GL_UNSIGNED_INT,
                std::ptr::null(),
                instances as i32,
            );
        }
        VertexArray::clear_binding();
    }
    fn setup_inst_attr(&self) {
        self.vao.bind();
        unsafe {
            for i in 0..4 {
                glEnableVertexAttribArray(3 + i);
                glVertexAttribPointer(
                    3 + i,
                    4,
                    GL_FLOAT,
                    GL_FALSE.0 as u8,
                    core::mem::size_of::<Instance>().try_into().unwrap(),
                    (i as usize * core::mem::size_of::<Vec4>()) as *const _,
                );
                glVertexAttribDivisor(3 + i, 1);
            }
            for i in 0..3 {
                glEnableVertexAttribArray(7 + i);
                glVertexAttribPointer(
                    7 + i,
                    3,
                    GL_FLOAT,
                    GL_FALSE.0 as u8,
                    core::mem::size_of::<Instance>().try_into().unwrap(),
                    (core::mem::offset_of!(Instance, normal)
                        + i as usize * core::mem::size_of::<Vec3>())
                        as *const _,
                );
                glVertexAttribDivisor(7 + i, 1);
            }
        }
        VertexArray::clear_binding();
    }
    fn cull_faces(&self) -> bool {
        self.cull_faces
    }
}

pub struct Skybox {
    pub texture: CubeMap,
    vertices: [Vertex; 24],
    indices: [u32; 36],
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Skybox {
    pub fn new(texture: CubeMap) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let vertices = [
            Vertex::new(-5.0, 5.0, -5.0),
            Vertex::new(5.0, 5.0, -5.0),
            Vertex::new(-5.0, 5.0, 5.0),
            Vertex::new(5.0, 5.0, 5.0),
            Vertex::new(-5.0, -5.0, -5.0),
            Vertex::new(5.0, -5.0, -5.0),
            Vertex::new(-5.0, -5.0, 5.0),
            Vertex::new(5.0, -5.0, 5.0),
            Vertex::new(5.0, 5.0, -5.0),
            Vertex::new(-5.0, 5.0, -5.0),
            Vertex::new(5.0, -5.0, -5.0),
            Vertex::new(-5.0, -5.0, -5.0),
            Vertex::new(5.0, 5.0, 5.0),
            Vertex::new(5.0, 5.0, -5.0),
            Vertex::new(5.0, -5.0, 5.0),
            Vertex::new(5.0, -5.0, -5.0),
            Vertex::new(-5.0, 5.0, 5.0),
            Vertex::new(5.0, 5.0, 5.0),
            Vertex::new(-5.0, -5.0, 5.0),
            Vertex::new(5.0, -5.0, 5.0),
            Vertex::new(-5.0, 5.0, -5.0),
            Vertex::new(-5.0, 5.0, 5.0),
            Vertex::new(-5.0, -5.0, -5.0),
            Vertex::new(-5.0, -5.0, 5.0),
        ];

        let indices = [
            0, 2, 1, 1, 2, 3, 4, 5, 6, 6, 5, 7, 8, 10, 9, 9, 10, 11, 12, 14, 13, 13, 14, 15, 16,
            18, 17, 17, 18, 19, 20, 22, 21, 21, 22, 23,
        ];

        let skybox = Skybox {
            texture,
            vertices,
            indices,
            vao,
            vbo,
            ebo,
        };
        skybox.setup_mesh();
        skybox
    }

    fn setup_mesh(&self) {
        self.vao.bind();

        self.vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(&self.vertices),
            GL_STATIC_DRAW,
        );

        self.ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&self.indices),
            GL_STATIC_DRAW,
        );

        unsafe {
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, pos) as *const _, // might seem redundant, but it's just in case the order changes
            );
        }
    }
}

impl Draw for Skybox {
    fn draw(&self, shader: &ShaderProgram) {
        self.vao.bind();
        shader.set_cubemap("skybox", &self.texture);
        unsafe {
            glDrawElements(
                GL_TRIANGLES,
                self.indices.len() as i32,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        VertexArray::clear_binding();
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(self.clone())
    }
    fn instanced_draw(&self, shader: &ShaderProgram, _: usize) {
        self.draw(shader);
    }
    fn setup_inst_attr(&self) {}
}

impl Clone for Skybox {
    fn clone(&self) -> Self {
        Skybox::new(self.texture.clone())
    }
}

pub struct Canvas {
    pub vertices: [Vertex; 4],
    pub indices: [u32; 6],
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Canvas {
    pub fn new() -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mut vertices = [
            Vertex::new(-1.0, 1.0, 0.0),
            Vertex::new(1.0, 1.0, 0.0),
            Vertex::new(-1.0, -1.0, 0.0),
            Vertex::new(1.0, -1.0, 0.0),
        ];
        let indices = [0, 2, 1, 1, 2, 3];

        vertices[0].tex_coords = vec3(0.0, 1.0, 0.0);
        vertices[1].tex_coords = vec3(1.0, 1.0, 0.0);
        vertices[2].tex_coords = vec3(0.0, 0.0, 0.0);
        vertices[3].tex_coords = vec3(1.0, 0.0, 0.0);

        let square = Canvas {
            vertices,
            indices,
            vao,
            vbo,
            ebo,
        };
        square.setup_mesh();
        square
    }
    fn setup_mesh(&self) {
        self.vao.bind();

        self.vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(&self.vertices),
            GL_STATIC_DRAW,
        );

        self.ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&self.indices),
            GL_STATIC_DRAW,
        );

        unsafe {
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, pos) as *const _, // might seem redundant, but it's just in case the order changes
            );
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, tex_coords) as *const _,
            );
        }
    }
}

impl Draw for Canvas {
    fn draw(&self, _shader: &ShaderProgram) {
        self.vao.bind();
        unsafe {
            glDrawElements(
                GL_TRIANGLES,
                self.indices.len() as i32,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        VertexArray::clear_binding();
    }
    fn clone_box(&self) -> Box<dyn Draw> {
        Box::new(Canvas::new())
    }
    fn instanced_draw(&self, shader: &ShaderProgram, _: usize) {
        self.draw(shader);
    }
    fn setup_inst_attr(&self) {}
}
