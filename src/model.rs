use std::rc::Rc;

use bytemuck::{NoUninit, Pod, Zeroable};
use nalgebra_glm::*;

#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
    color: Vec3,
    tex_coords: Vec3,
}

impl Vertex {
    pub fn new(posx: f32, posy: f32, posz: f32) -> Self {
        Vertex {
            pos: vec3(posx, posy, posz),
            color: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
        }
    }
    pub fn from_vector(pos: Vec3) -> Self {
        Vertex {
            pos,
            color: vec3(0.0, 0.0, 0.0),
            tex_coords: vec3(0.0, 0.0, 0.0),
        }
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

pub trait Shape {
    fn make_bufferable_data<A: NoUninit>(&self) -> &[A];
}

#[derive(Debug)]
pub struct Triangle {
    vertices: [Vertex; 3],
}

impl Triangle {
    pub fn from_vectors(vecs: [Vec3; 3]) -> Self {
        Triangle {
            vertices: [
                Vertex::from_vector(vecs[0]),
                Vertex::from_vector(vecs[1]),
                Vertex::from_vector(vecs[2]),
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

    pub fn colors_from_vectors(&mut self, colors: [Option<Vec3>; 3]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].color_from_vector(colors[i].unwrap());
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Option<Vec2>; 3]) {
        for i in 0..self.vertices.len() {
            self.vertices[i].tex_coords_from_vector(tex_coords[i].unwrap());
        }
    }

    pub fn make_bufferable_data<A: NoUninit + Pod>(&self) -> &[A] {
        bytemuck::cast_slice(&self.vertices)
    }
}

pub struct Polygon {
    vertices: Vec<Vertex>,
    indices: Vec<[usize; 3]>,
}
