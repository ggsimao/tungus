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

    pub fn translate(&mut self, offset_x: f32, offset_y: f32, offset_z: f32) {
        self.pos = self.pos + vec3(offset_x, offset_y, offset_z);
    }
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        let mut matrix = Mat4::identity();
        matrix = rotate(&matrix, angle, axis);
        self.pos = vec4_to_vec3(&(matrix * vec3_to_vec4(&self.pos)));
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

    pub fn make_bufferable_data<A: NoUninit + Pod>(&self) -> &[A] {
        bytemuck::cast_slice(&self.vertices)
    }
}

pub struct Polygon {
    vertices: Vec<Vertex>,
    indices: Vec<[usize; 3]>,
}

pub struct Pyramid {
    triangles: [Triangle; 4],
}

impl Pyramid {
    pub fn regular(side: f32) -> Self {
        let slant_height = (0.75 * side * side).sqrt();
        let centroid_height = slant_height / 3.0;
        let height = (slant_height * slant_height - centroid_height * centroid_height).sqrt();
        let slant = 180.0_f32.to_radians() / 2.0 - (height / slant_height).asin();
        let mut triangles = [Triangle::equilateral(side); 4];

        triangles[0].rotate(180.0_f32.to_radians() / 2.0, &vec3(1.0, 0.0, 0.0));
        triangles[1].rotate(slant, &vec3(1.0, 0.0, 0.0));
        triangles[2].rotate(slant, &vec3(1.0, 0.0, 0.0));
        triangles[3].rotate(slant, &vec3(1.0, 0.0, 0.0));

        triangles[0].translate(0.0, -height / 2.0, slant_height / 2.0 - centroid_height);
        triangles[1].translate(0.0, 0.0, -centroid_height / 2.0);
        triangles[2].translate(0.0, 0.0, -centroid_height / 2.0);
        triangles[3].translate(0.0, 0.0, -centroid_height / 2.0);
        triangles[2].rotate(120.0_f32.to_radians(), &vec3(0.0, 1.0, 0.0));
        triangles[3].rotate(-120.0_f32.to_radians(), &vec3(0.0, 1.0, 0.0));

        Pyramid { triangles }
    }

    pub fn colors_from_vectors(&mut self, colors: [Vec3; 12]) {
        for i in 0..self.triangles.len() {
            self.triangles[i].colors_from_vectors([colors[i], colors[i + 1], colors[i + 2]]);
        }
    }

    pub fn tex_coords_from_vectors(&mut self, tex_coords: [Vec2; 12]) {
        let number_of_triangles = self.triangles.len();
        for i in 0..number_of_triangles {
            self.triangles[i].tex_coords_from_vectors([
                tex_coords[i * 3],
                tex_coords[i * 3 + 1],
                tex_coords[i * 3 + 2],
            ]);
        }
    }

    pub fn get_triangles(&self) -> &[Triangle; 4] {
        &self.triangles
    }
}
