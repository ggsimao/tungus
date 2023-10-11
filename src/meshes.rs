use bytemuck::{NoUninit, Pod, Zeroable};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

use crate::rendering::buffer_data;
use crate::shaders::Shader;
use crate::shaders::ShaderProgram;
use crate::textures::TextureType;
use crate::{
    rendering::{Buffer, BufferType, VertexArray},
    textures::Texture,
};

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
    pub textures: Vec<Texture>,
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Self {
        let vao = VertexArray::new().expect("Couldn't make a VAO");
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        let ebo = Buffer::new().expect("Couldn't make the indices buffer");

        let mesh = Mesh {
            vertices,
            indices,
            textures,
            vao,
            vbo,
            ebo,
        };
        mesh.setup_mesh();
        mesh
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

    pub fn draw(&self, shader: &ShaderProgram) {
        let (mut diffuse_count, mut specular_count) = (0, 0);
        for i in 0..self.textures.len() {
            unsafe {
                glActiveTexture(GLenum(GL_TEXTURE0.0 + i as u32));
            }
            self.textures[i].bind();
            let ttype = self.textures[i].get_type();
            let name;
            match ttype {
                TextureType::Diffuse => {
                    name = format!(
                        "material.{:?}[{}]",
                        self.textures[i].get_type(),
                        diffuse_count
                    );
                    diffuse_count += 1;
                }
                TextureType::Specular => {
                    name = format!(
                        "material.{:?}[{}]",
                        self.textures[i].get_type(),
                        specular_count
                    );
                    specular_count += 1;
                }
            }
            shader.set_1i(&name, (self.textures[i].get_id() - 1) as i32);
        }
        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        shader.set_1f("material.shininess", 128.0); // TEMP
        unsafe {
            glActiveTexture(GL_TEXTURE0);
        }
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
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    vertices: [Vertex; 3],
}

impl Triangle {
    pub fn equilateral(side: f32) -> Self {
        let height = (0.75 * side * side).sqrt();
        Triangle {
            vertices: [
                Vertex::new(-side / 2.0, -height / 2.0, 0.0),
                Vertex::new(side / 2.0, -height / 2.0, 0.0),
                Vertex::new(0.0, height / 2.0, 0.0),
            ],
        }
    }
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Triangle {
            vertices: [a, b, c],
        }
    }
    pub fn from_array(vertices: [Vertex; 3]) -> Self {
        Triangle { vertices }
    }

    pub fn get_vertices(&self) -> &[Vertex; 3] {
        &self.vertices
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        for vertex in &mut self.vertices {
            vertex.translate(offset_x, offset_y, offset_z);
        }
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        for vertex in &mut self.vertices {
            vertex.rotate(angle, axis);
        }
    }

    pub fn centroid(&self) {
        let mut centroid = vec3(0.0, 0.0, 0.0);
        for vertex in self.vertices {
            centroid += vertex.pos;
        }
        centroid /= 3.0
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 3]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].tex_coords = tex_coords[i];
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Quadrilateral {
    vertices: [Vertex; 4],
}

impl Quadrilateral {
    pub fn square(side: f32) -> Self {
        Quadrilateral {
            vertices: [
                Vertex::new(-side / 2.0, side / 2.0, 0.0),
                Vertex::new(side / 2.0, side / 2.0, 0.0),
                Vertex::new(side / 2.0, -side / 2.0, 0.0),
                Vertex::new(-side / 2.0, -side / 2.0, 0.0),
            ],
        }
    }

    pub fn get_vertices(&self) -> &[Vertex; 4] {
        &self.vertices
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        for vertex in &mut self.vertices {
            vertex.translate(offset_x, offset_y, offset_z);
        }
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        for vertex in &mut self.vertices {
            vertex.rotate(angle, axis);
        }
    }

    pub fn centroid(&self) {
        let mut centroid = vec3(0.0, 0.0, 0.0);
        for vertex in self.vertices {
            centroid += vertex.pos;
        }
        centroid /= 4.0
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 4]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].tex_coords = tex_coords[i];
        }
    }
}

#[derive(Debug)]
pub struct Hexahedron {
    vertices: [Vertex; 8],
    indices: [u32; 36],
}

impl Hexahedron {
    pub fn cube(side: f32) -> Self {
        let mut vertices = [
            Vertex::new(-side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, side / 2.0, side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, -side / 2.0),
            Vertex::new(-side / 2.0, -side / 2.0, side / 2.0),
            Vertex::new(side / 2.0, -side / 2.0, side / 2.0),
        ];
        let indices: [u32; 36] = [
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
        }
        Hexahedron { vertices, indices }
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        for vertex in &mut self.vertices {
            vertex.translate(offset_x, offset_y, offset_z);
        }
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        for vertex in &mut self.vertices {
            vertex.rotate(angle, axis);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 8]) {
        for i in 0..8 {
            self.vertices[i].tex_coords = tex_coords[i];
        }
    }

    pub fn get_vertices(&self) -> &[Vertex; 8] {
        &self.vertices
    }
    pub fn get_indices(&self) -> [u32; 36] {
        self.indices
    }
}

pub struct TriangularPyramid {
    vertices: [Vertex; 4],
    indices: [u32; 12],
}

impl TriangularPyramid {
    pub fn regular(side: f32) -> Self {
        let slant_height = (0.75 * side * side).sqrt();
        let centroid_height = slant_height / 3.0;
        let height = (slant_height * slant_height - centroid_height * centroid_height).sqrt();

        let mut vertices = [
            Vertex::new(0.0, 0.75 * height, 0.0),
            Vertex::new(0.0, -0.25 * height, 2.0 * centroid_height),
            Vertex::new(-side / 2.0, -0.25 * height, -centroid_height),
            Vertex::new(side / 2.0, -0.25 * height, -centroid_height),
        ];
        let indices = [0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
        let mut normals = [Vec3::zeros(); 4];

        for i in 0..4 {
            let main_vertex = vertices[indices[i * 3] as usize];
            let v1 = vertices[indices[i * 3 + 1] as usize];
            let v2 = vertices[indices[i * 3 + 2] as usize];
            let normal = normalize(&cross(
                &(v1.pos - main_vertex.pos),
                &(v2.pos - main_vertex.pos),
            ));
            normals[indices[i * 3] as usize] += normal;
            normals[indices[i * 3 + 1] as usize] += normal;
            normals[indices[i * 3 + 2] as usize] += normal;
        }
        for i in 0..4 {
            vertices[i].normal = normals[i] / 3.0;
        }

        TriangularPyramid { vertices, indices }
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        for vertex in &mut self.vertices {
            vertex.translate(offset_x, offset_y, offset_z);
        }
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        for vertex in &mut self.vertices {
            vertex.rotate(angle, axis);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 4]) {
        for i in 0..4 {
            self.vertices[i].tex_coords = tex_coords[i];
        }
    }

    pub fn get_vertices(&self) -> [Vertex; 4] {
        self.vertices
    }

    pub fn get_indices(&self) -> [u32; 12] {
        self.indices
    }
}
