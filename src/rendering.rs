use bytemuck::{NoUninit, Pod, Zeroable};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use nalgebra_glm::*;

use crate::model::{Hexahedron, Quadrilateral, Triangle, TriangularPyramid, Vertex};

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { glClearColor(r, g, b, a) }
}

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

pub struct Buffer(pub u32);
impl Buffer {
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            glGenBuffers(1, &mut vbo);
        }
        if vbo != 0 {
            Some(Self(vbo))
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
    unsafe { glPolygonMode(GL_FRONT_AND_BACK, GLenum(mode as u32)) };
}

pub trait Draw {
    fn get_indices(&self) -> &[u32] {
        &[]
    }
    fn get_vbo(&self) -> &Buffer {
        &Buffer(0)
    }
    fn get_ebo(&self) -> &Buffer {
        &Buffer(0)
    }
    fn ready_buffers(&self) {
        self.get_vbo().bind(BufferType::Array);
        self.get_ebo().bind(BufferType::ElementArray);
        unsafe {
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                0 as *const _,
            );
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::size_of::<Vec3>() as *const _,
            );
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(
                2,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                (core::mem::size_of::<Vec3>() * 2) as *const _,
            );
            glEnableVertexAttribArray(2);
            glVertexAttribPointer(
                3,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                (core::mem::size_of::<Vec3>() * 3) as *const _,
            );
            glEnableVertexAttribArray(3);
        }
    }

    fn draw(&self) {
        unsafe {
            glDrawElements(
                GL_TRIANGLES,
                self.get_indices().len() as i32,
                GL_UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}

pub struct TriangleDrawer {
    triangle: Triangle,
    vbo: Buffer,
}

impl TriangleDrawer {
    pub fn new(triangle: Triangle) -> Self {
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(triangle.get_vertices()),
            GL_STATIC_DRAW,
        );

        TriangleDrawer { triangle, vbo }
    }
}

impl Draw for TriangleDrawer {
    fn ready_buffers(&self) {
        self.vbo.bind(BufferType::Array);
        unsafe {
            glVertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                0 as *const _,
            );
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(
                1,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                core::mem::size_of::<Vec3>() as *const _,
            );
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(
                2,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                (core::mem::size_of::<Vec3>() * 2) as *const _,
            );
            glEnableVertexAttribArray(2);
            glVertexAttribPointer(
                3,
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                core::mem::size_of::<Vertex>().try_into().unwrap(),
                (core::mem::size_of::<Vec3>() * 3) as *const _,
            );
            glEnableVertexAttribArray(3);
        }
    }

    fn draw(&self) {
        unsafe {
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
    }
}

pub struct QuadrilateralDrawer {
    quadrilateral: Quadrilateral,
    vbo: Buffer,
    ebo: Buffer,
    indices: [u32; 6],
}

impl QuadrilateralDrawer {
    pub fn new(quadrilateral: Quadrilateral) -> Self {
        let indices = [0, 1, 3, 1, 2, 3];
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(quadrilateral.get_vertices()),
            GL_STATIC_DRAW,
        );

        let ebo = Buffer::new().expect("Couldn't make the indices buffer");
        ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&indices),
            GL_STATIC_DRAW,
        );

        QuadrilateralDrawer {
            quadrilateral,
            vbo,
            ebo,
            indices,
        }
    }
}

impl Draw for QuadrilateralDrawer {
    fn get_indices(&self) -> &[u32] {
        &self.indices
    }
    fn get_vbo(&self) -> &Buffer {
        &self.vbo
    }
    fn get_ebo(&self) -> &Buffer {
        &self.ebo
    }
}

pub struct TriangularPyramidDrawer {
    pyramid: TriangularPyramid,
    vbo: Buffer,
    ebo: Buffer,
    indices: [u32; 12],
}

impl TriangularPyramidDrawer {
    pub fn new(pyramid: TriangularPyramid) -> Self {
        let indices = pyramid.get_indices();
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(&pyramid.get_vertices()),
            GL_STATIC_DRAW,
        );

        let ebo = Buffer::new().expect("Couldn't make the indices buffer");
        ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&indices),
            GL_STATIC_DRAW,
        );

        TriangularPyramidDrawer {
            pyramid,
            vbo,
            ebo,
            indices,
        }
    }

    pub fn get_pyramid(&self) -> &TriangularPyramid {
        &self.pyramid
    }
}

impl Draw for TriangularPyramidDrawer {
    fn get_indices(&self) -> &[u32] {
        &self.indices
    }
    fn get_vbo(&self) -> &Buffer {
        &self.vbo
    }
    fn get_ebo(&self) -> &Buffer {
        &self.ebo
    }
}

pub struct HexahedronDrawer {
    hexahedron: Hexahedron,
    vbo: Buffer,
    ebo: Buffer,
    indices: [u32; 36],
}

impl HexahedronDrawer {
    pub fn new(hexahedron: Hexahedron) -> Self {
        let indices = hexahedron.get_indices();
        let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
        vbo.bind(BufferType::Array);
        buffer_data(
            BufferType::Array,
            bytemuck::cast_slice(hexahedron.get_vertices()),
            GL_STATIC_DRAW,
        );

        let ebo = Buffer::new().expect("Couldn't make the indices buffer");
        ebo.bind(BufferType::ElementArray);
        buffer_data(
            BufferType::ElementArray,
            bytemuck::cast_slice(&indices),
            GL_STATIC_DRAW,
        );

        HexahedronDrawer {
            hexahedron,
            vbo,
            ebo,
            indices,
        }
    }

    pub fn get_hexahedron(&self) -> &Hexahedron {
        &self.hexahedron
    }
}

impl Draw for HexahedronDrawer {
    fn get_indices(&self) -> &[u32] {
        &self.indices
    }
    fn get_vbo(&self) -> &Buffer {
        &self.vbo
    }
    fn get_ebo(&self) -> &Buffer {
        &self.ebo
    }
}
