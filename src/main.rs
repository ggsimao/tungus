#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Tungus";

use crate::helpers::{
    Buffer,
    BufferType,
    PolygonMode,
    VertexArray, //TextureCoord, Vertex, VertexArray, VertexColor, VertexPos,
};
use crate::model::{Triangle, Vertex};
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
use nalgebra_glm::*;
use std::path::Path;

pub mod helpers;
pub mod model;
pub mod shaders;
pub mod textures;

const INDEX_DIMENSIONS: usize = 3;

type TriangleIndexes = [u32; INDEX_DIMENSIONS];

const INDICES: [TriangleIndexes; 3] = [[0, 1, 2], [3, 4, 5], [6, 7, 8]];

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";
const FRAG_SHADER_GREEN: &str = "./src/shaders/green_frag_shader.fs";
const FRAG_SHADER_RAINBOW: &str = "./src/shaders/rainbow_frag_shader.fs";
const FRAG_SHADER_TEXTURE: &str = "./src/shaders/texture_frag_shader.fs";

// const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container.jpg";
const FACE_TEXTURE: &str = "./src/resources/textures/awesomeface.png";

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
            800,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync);

    unsafe {
        let fun =
            |x: *const u8| win.get_proc_address(x as *const i8) as *const std::os::raw::c_void;
        load_global_gl(&fun);
    }

    let mut all_vertices: [Vertex; 9] = [
        Vertex::new(0.33, 0.0, 0.0),
        Vertex::new(0.0, -0.66, 0.0),
        Vertex::new(0.66, -0.66, 0.0),
        Vertex::new(0.0, -0.66, 0.0),
        Vertex::new(-0.66, -0.66, 0.0),
        Vertex::new(-0.33, 0.0, 0.0),
        Vertex::new(0.33, 0.0, 0.0),
        Vertex::new(-0.33, 0.0, 0.0),
        Vertex::new(0.0, 0.66, 0.0),
    ];
    let all_colors: [Vec3; 9] = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 0.0),
    ];
    let all_texcoords: [Vec2; 9] = [
        vec2(0.5, 0.501),
        vec2(0.499, 0.499),
        vec2(0.501, 0.499),
        vec2(2.99, 1.99),
        vec2(0.0, 0.0),
        vec2(0.0, 2.0),
        vec2(3.0, 2.0),
        vec2(0.0, 2.0),
        vec2(1.5, 0.0),
    ];
    for i in 0..all_vertices.len() {
        all_vertices[i].color_from_vector(all_colors[i]);
        all_vertices[i].tex_coords_from_vector(all_texcoords[i]);
    }

    helpers::clear_color(0.2, 0.3, 0.3, 1.0);

    let mut triangles: Vec<Triangle> = Vec::new();

    for indices in INDICES {
        let mut vertices: Vec<Vertex> = Vec::new();
        for index in indices {
            vertices.push(all_vertices[index as usize]);
        }
        triangles.push(Triangle::from_array(vertices[..].try_into().unwrap()));
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
        let returned_objects =
            helpers::initialize_vertex_objects(triangle.make_bufferable_data() as &[u8]);
        all_vertex_objects.push(new_vertex_objects(returned_objects.0, returned_objects.1));
    }

    let mut texture1 = Texture::new();
    texture1.load(Path::new(CONTAINER_TEXTURE));
    texture1.set_wrapping(GL_REPEAT);
    texture1.set_filters(GL_NEAREST, GL_NEAREST);
    let mut texture2 = Texture::new();
    texture2.load(Path::new(FACE_TEXTURE));
    texture2.set_wrapping(GL_REPEAT);
    texture2.set_filters(GL_NEAREST, GL_NEAREST);

    // let ebo = Buffer::new().expect("Couldn't make the element buffer.");
    // ebo.bind(BufferType::ElementArray);
    // learn::buffer_data(
    //     BufferType::ElementArray,
    //     bytemuck::cast_slice(&INDICES[0]),
    //     GL_STATIC_DRAW,
    // );

    let mut all_shader_programs: Vec<ShaderProgram> = vec![];

    let shader_program_1 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    let shader_program_2 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_RAINBOW).unwrap();
    let shader_program_3 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    all_shader_programs.push(shader_program_1);
    all_shader_programs.push(shader_program_2);
    all_shader_programs.push(shader_program_3);

    helpers::polygon_mode(PolygonMode::Fill);

    let mut mixer: f64 = 0.2;

    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                Event::Keyboard(key_event) => match key_event.key.keycode {
                    Keycode::UP => mixer = (mixer + 0.02).min(1.0),
                    Keycode::DOWN => mixer = (mixer - 0.02).max(0.0),
                    _ => (),
                },
                _ => (),
            }
        }
        // now the events are clear.

        // here's where we could change the world state if we had some.

        // and then draw!
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            for (i, vertex_objects) in all_vertex_objects.iter().enumerate() {
                glActiveTexture(GL_TEXTURE0);
                texture1.bind();
                glActiveTexture(GL_TEXTURE1);
                texture2.bind();
                helpers::use_vertex_objects(&vertex_objects.vao, &vertex_objects.vbo);
                all_shader_programs[i].use_program();
                let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
                let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

                let mut transform = Mat4::identity();
                transform = rotate(&transform, time_value.into(), &vec3(0.0, 0.0, 1.0));
                transform = scale(&transform, &vec3(pulse, pulse, pulse));

                all_shader_programs[i].set_matrix_4fv("transform", transform.as_ptr());
                all_shader_programs[i].set_4f("ourColor", [0.0, pulse, 0.0, 1.0]);
                all_shader_programs[i].set_1i("ourTexture1", 0);
                all_shader_programs[i].set_1i("ourTexture2", 1);
                all_shader_programs[i].set_1f("mixer", mixer as f32);
                glDrawArrays(GL_TRIANGLES, 0, 3);
            }
        }
        win.swap_window();
    }
}
