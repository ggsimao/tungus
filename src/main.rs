#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Triangle: Elements";

use crate::helpers::{
    Buffer, BufferType, PolygonMode, Vertex, VertexArray, VertexColor, VertexPos,
};
use crate::shaders::{Shader, ShaderProgram, ShaderType};
use beryllium::*;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
    ptr::null,
};
// use learn_opengl as learn;
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;

pub mod helpers;
pub mod shaders;

type TriIndexes = [u32; 3];

const VERTICES: [Vertex; 5] = [
    Vertex::new([0.25, 0.5, 0.0], [1.0, 0.0, 0.0]),
    Vertex::new([0.0, -0.5, 0.0], [0.0, 1.0, 0.0]),
    Vertex::new([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),
    Vertex::new([-0.25, 0.5, 0.0], [0.0, 0.0, 1.0]),
    Vertex::new([0.5, -0.5, 0.0], [0.0, 0.0, 1.0]),
];

const INDICES: [TriIndexes; 2] = [[0, 1, 4], [1, 2, 3]];

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";

const FRAG_SHADER_ORANGE: &str = "./src/shaders/orange_frag_shader.fs";

const FRAG_SHADER_YELLOW: &str = "./src/shaders/yellow_frag_shader.fs";

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
        // |f_name| win.get_proc_address(f_name)
    }

    helpers::clear_color(0.2, 0.3, 0.3, 1.0);

    // println!("{:?}", &[VERTICES[0][0], VERTICES[1][0], VERTICES[4][0]]);
    // println!("{:?}", &[VERTICES[1][0], VERTICES[2][0], VERTICES[3][0]]);

    let (vao1, vbo1) = helpers::initialize_vertex_objects(&[VERTICES[0], VERTICES[1], VERTICES[4]]);
    let (vao2, vbo2) = helpers::initialize_vertex_objects(&[VERTICES[1], VERTICES[2], VERTICES[3]]);

    // let ebo = Buffer::new().expect("Couldn't make the element buffer.");
    // ebo.bind(BufferType::ElementArray);
    // learn::buffer_data(
    //     BufferType::ElementArray,
    //     bytemuck::cast_slice(&INDICES[0]),
    //     GL_STATIC_DRAW,
    // );

    let shader_program_1: ShaderProgram =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_ORANGE).unwrap();
    let shader_program_2 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_YELLOW).unwrap();

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
            helpers::use_vertex_objects(&vao1, &vbo1);
            let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
            let green_value: f32 = (time_value.sin() / 2.0) + 0.5;
            let x_offset: f32 = time_value.sin();
            shader_program_1.use_program();
            shader_program_1.set_4f("ourColor", [0.0, green_value, 0.0, 1.0]);
            shader_program_1.set_1f("offset_x", x_offset);
            glDrawArrays(GL_TRIANGLES, 0, 3);
            helpers::use_vertex_objects(&vao2, &vbo2);
            shader_program_2.use_program();
            shader_program_1.set_1f("offset_x", x_offset);
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
        win.swap_window();
    }
}
