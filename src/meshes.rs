use bytemuck::{NoUninit, Pod, Zeroable};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

use crate::rendering::buffer_data;
use crate::shaders::Shader;
use crate::shaders::ShaderProgram;
use crate::textures::Material;
use crate::textures::TextureType;
use crate::{
    rendering::{Buffer, BufferType, VertexArray},
    textures::Texture,
};

pub trait Draw {
    fn draw(&self, shader: &ShaderProgram);
    fn clone_box(&self) -> Box<dyn Draw>;
}

impl Clone for Box<dyn Draw> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
}

impl Vertex {
    pub fn new(posx: f32, posy: f32, posz: f32) -> Self {
        Vertex {
            pos: vec3(posx, posy, posz),
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec2(0.0, 0.0),
        }
    }
    pub fn from_vector(pos: Vec3) -> Self {
        Vertex {
            pos,
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec2(0.0, 0.0),
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

impl Clone for Vertex {
    fn clone(&self) -> Self {
        Vertex::from_vector(self.pos)
    }
}
impl Copy for Vertex {}
unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, material: Material) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mesh = Mesh {
            vertices,
            indices,
            material,
            vao,
            vbo,
            ebo,
        };
        mesh.setup_mesh();
        mesh
    }

    pub fn get_vao(&self) -> &VertexArray {
        &self.vao
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
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::offset_of!(Vertex, tex_coords) as *const _,
            );
        }
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
        ];
        // order should be counter-clockwise for each triangle
        let indices = vec![
            0, 2, 1, 1, 2, 3, 0, 1, 4, 4, 1, 5, 2, 0, 6, 6, 0, 4, 3, 7, 1, 1, 7, 5, 2, 6, 3, 3, 6,
            7, 7, 6, 5, 5, 6, 4,
        ];
        let mut normals = [Vec3::zeros(); 8];

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
        for i in 0..8 {
            vertices[i].normal = normals[i] / 4.0;
            vertices[i].tex_coords = vec2((i % 2) as f32, (i / 4) as f32);
        }
        let cube = Mesh {
            vertices,
            indices,
            material: Material::new(vec![], vec![], 1.0),
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
            vertices[i].tex_coords = vec2((i % 2) as f32, (i as i32 / -2 + 1) as f32);
        }
        let square = Mesh {
            vertices,
            indices,
            material: Material::new(vec![], vec![], 1.0),
            vao,
            vbo,
            ebo,
        };
        square.setup_mesh();
        square
    }
}

impl Draw for Mesh {
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
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Mesh::new(
            self.vertices.clone(),
            self.indices.clone(),
            self.material.clone(),
        )
    }
}
