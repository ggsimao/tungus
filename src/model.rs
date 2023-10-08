use bytemuck::{NoUninit, Pod, Zeroable};
use nalgebra_glm::*;

use crate::rendering::Buffer;

#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
    normal: Vec3,
    color: Vec3,
    tex_coords: Vec3,
}

impl Vertex {
    pub fn new(posx: f32, posy: f32, posz: f32) -> Self {
        Vertex {
            pos: vec3(posx, posy, posz),
            color: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
            normal: vec3(0.0, 0.0, 0.0),
        }
    }
    pub fn from_vector(pos: Vec3) -> Self {
        Vertex {
            pos,
            color: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
            normal: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        self.pos = self.pos + vec3(offset_x, offset_y, offset_z);
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        let matrix = rotation(angle, axis);
        self.pos = vec4_to_vec3(&(matrix * vec3_to_vec4(&self.pos)));
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn set_normal(&mut self, normal: Vec3) {
        self.normal = normal;
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color = vec3(r, g, b);
    }
    pub fn color_from_vector(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn set_tex_coords(&mut self, x: f32, y: f32) {
        self.tex_coords = vec3(x, y, 0.0);
    }
    pub fn tex_coords_from_vector(&mut self, tex_coords: Vec2) {
        self.tex_coords = vec3(tex_coords.x, tex_coords.y, 0.0);
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

    pub fn colors_from_vectors(&mut self, colors: [Vec3; 3]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].color_from_vector(colors[i]);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 3]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].tex_coords_from_vector(tex_coords[i]);
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

    pub fn colors_from_vectors(&mut self, colors: [Vec3; 4]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].color_from_vector(colors[i]);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 4]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].tex_coords_from_vector(tex_coords[i]);
        }
    }
}

pub struct Polygon {
    vertices: Vec<Vertex>,
    indices: Vec<[usize; 3]>,
}

pub struct Hexahedron {
    vertices: [Vertex; 8],
}

impl Hexahedron {
    pub fn cube(side: f32) -> Self {
        Hexahedron {
            vertices: [
                Vertex::new(-side / 2.0, side / 2.0, -side / 2.0),
                Vertex::new(side / 2.0, side / 2.0, -side / 2.0),
                Vertex::new(-side / 2.0, side / 2.0, side / 2.0),
                Vertex::new(side / 2.0, side / 2.0, side / 2.0),
                Vertex::new(-side / 2.0, -side / 2.0, -side / 2.0),
                Vertex::new(side / 2.0, -side / 2.0, -side / 2.0),
                Vertex::new(-side / 2.0, -side / 2.0, side / 2.0),
                Vertex::new(side / 2.0, -side / 2.0, side / 2.0),
            ],
        }
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

    pub fn colors_from_vectors(&mut self, colors: [Vec3; 8]) {
        for i in 0..8 {
            self.vertices[i].color_from_vector(colors[i]);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 8]) {
        for i in 0..8 {
            self.vertices[i].tex_coords_from_vector(tex_coords[i]);
        }
    }

    pub fn get_vertices(&self) -> &[Vertex; 8] {
        &self.vertices
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
                &(v1.get_pos() - main_vertex.get_pos()),
                &(v2.get_pos() - main_vertex.get_pos()),
            ));
            normals[indices[i * 3] as usize] += normal;
            normals[indices[i * 3 + 1] as usize] += normal;
            normals[indices[i * 3 + 2] as usize] += normal;
        }
        for i in 0..4 {
            vertices[i].set_normal(normals[i] / 3.0);
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

    pub fn colors_from_vectors(&mut self, colors: [Vec3; 4]) {
        for i in 0..4 {
            self.vertices[i].color_from_vector(colors[i]);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 4]) {
        for i in 0..4 {
            self.vertices[i].tex_coords_from_vector(tex_coords[i]);
        }
    }

    pub fn get_vertices(&self) -> [Vertex; 4] {
        self.vertices
    }

    pub fn get_indices(&self) -> [u32; 12] {
        self.indices
    }
}
