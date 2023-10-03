#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Tungus";

use crate::helpers::{
    Buffer, BufferType, PolygonMode, TextureCoord, Vertex, VertexArray, VertexColor, VertexPos,
};
use crate::shaders::{Shader, ShaderProgram, ShaderType};
use crate::textures::Texture;
use beryllium::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
    ptr::null,
};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use std::path::Path;

pub mod helpers;
pub mod shaders;
pub mod textures;

const INDEX_DIMENSIONS: usize = 3;

type TriangleIndexes = [u32; INDEX_DIMENSIONS];

const VERTICES: [Vertex; 6] = [
    Vertex::new([0.33, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0]),
    Vertex::new([0.0, -0.66, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
    Vertex::new([-0.66, -0.66, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
    Vertex::new([-0.33, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
    Vertex::new([0.66, -0.66, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
    Vertex::new([0.0, 0.66, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5]),
];

const INDICES: [TriangleIndexes; 3] = [[0, 1, 4], [1, 2, 3], [0, 3, 5]];

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";
const FRAG_SHADER_GREEN: &str = "./src/shaders/green_frag_shader.fs";
const FRAG_SHADER_RAINBOW: &str = "./src/shaders/rainbow_frag_shader.fs";
const FRAG_SHADER_TEXTURE: &str = "./src/shaders/texture_frag_shader.fs";

// const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container.jpg";

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();

    let win = sdl
        .create_gl_window(
            WINDOW_TITLE,
            WindowPosition::Centered,
            800,
            600,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync);

    unsafe {
        let fun =
            |x: *const u8| win.get_proc_address(x as *const i8) as *const std::os::raw::c_void;
        load_global_gl(&fun);
    }

    helpers::clear_color(0.2, 0.3, 0.3, 1.0);

    let mut triangles: Vec<Vec<Vertex>> = vec![];
    for indices in INDICES {
        let mut triangle = vec![];
        for index in indices {
            triangle.push(VERTICES[index as usize]);
        }
        triangles.push(triangle);
    }

    struct VertexObjects {
        vao: VertexArray,
        vbo: Buffer,
    }
    fn new_vertex_objects(vao: VertexArray, vbo: Buffer) -> VertexObjects {
        VertexObjects { vao: vao, vbo: vbo }
    }

    let mut all_vertex_objects: Vec<VertexObjects> = vec![];
    for triangle in triangles {
        let returned_objects = helpers::initialize_vertex_objects(&triangle);
        all_vertex_objects.push(new_vertex_objects(returned_objects.0, returned_objects.1));
    }

    let mut texture = Texture::new();
    texture.load(Path::new(CONTAINER_TEXTURE));
    texture.set_wrapping(GL_REPEAT);
    texture.set_filters(GL_LINEAR, GL_LINEAR);

    // let ebo = Buffer::new().expect("Couldn't make the element buffer.");
    // ebo.bind(BufferType::ElementArray);
    // learn::buffer_data(
    //     BufferType::ElementArray,
    //     bytemuck::cast_slice(&INDICES[0]),
    //     GL_STATIC_DRAW,
    // );

    let mut all_shader_programs: Vec<ShaderProgram> = vec![];

    let shader_program_1: ShaderProgram =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_GREEN).unwrap();
    let shader_program_2 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_RAINBOW).unwrap();
    let shader_program_3 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    all_shader_programs.push(shader_program_1);
    all_shader_programs.push(shader_program_2);
    all_shader_programs.push(shader_program_3);

    helpers::polygon_mode(PolygonMode::Fill);

    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                _ => (),
            }
        }
        // now the events are clear.

        // here's where we could change the world state if we had some.

        // and then draw!
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            for (i, vertex_objects) in all_vertex_objects.iter().enumerate() {
                texture.bind();
                helpers::use_vertex_objects(&vertex_objects.vao, &vertex_objects.vbo);
                all_shader_programs[i].use_program();
                let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
                let x_offset: f32 = time_value.sin();
                if i == 0 {
                    let green_value: f32 = (time_value.sin() / 2.0) + 0.5;
                    all_shader_programs[i].set_4f("ourColor", [0.0, green_value, 0.0, 1.0]);
                }
                all_shader_programs[i].set_1f("offset_x", x_offset);
                glDrawArrays(GL_TRIANGLES, 0, 3);
            }
        }
        win.swap_window();
    }
}
