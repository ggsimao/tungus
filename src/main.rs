#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]

const WINDOW_TITLE: &str = "Tungus";

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
use helpers::{
    // , //TextureCoord, Vertex, VertexArray, VertexColor, VertexPos,
    Buffer,
    BufferType,
    PolygonMode,
    VertexArray,
};
use model::{Pyramid, Triangle, Vertex};
use nalgebra_glm::*;
use shaders::{Shader, ShaderProgram, ShaderType};
use std::path::Path;
use systems::Camera;
use textures::Texture;

pub mod helpers;
pub mod model;
pub mod shaders;
pub mod systems;
pub mod textures;

// const INDEX_DIMENSIONS: usize = 3;

// type TriangleIndexes = [u32; INDEX_DIMENSIONS];

// const INDICES: [TriangleIndexes; 4] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11]];

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";
const FRAG_SHADER_GREEN: &str = "./src/shaders/green_frag_shader.fs";
const FRAG_SHADER_RAINBOW: &str = "./src/shaders/rainbow_frag_shader.fs";
const FRAG_SHADER_TEXTURE: &str = "./src/shaders/texture_frag_shader.fs";

const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
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
            WindowPosition::XY(1000, 100),
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

    let _ = sdl.set_relative_mouse_mode(true);

    unsafe {
        glEnable(GL_DEPTH_TEST);
    }

    let mut pyramid = Pyramid::regular(1.0);
    let all_colors: [Vec3; 12] = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 0.0),
    ];
    let all_texcoords: [Vec2; 12] = [
        vec2(3.0, 2.0),
        vec2(0.0, 2.0),
        vec2(1.5, 0.0),
        vec2(3.0, 2.0),
        vec2(0.0, 2.0),
        vec2(1.5, 0.0),
        vec2(3.0, 2.0),
        vec2(0.0, 2.0),
        vec2(1.5, 0.0),
        vec2(3.0, 2.0),
        vec2(0.0, 2.0),
        vec2(1.5, 0.0),
    ];
    pyramid.colors_from_vectors(all_colors);
    pyramid.tex_coords_from_vectors(all_texcoords);

    helpers::clear_color(0.2, 0.3, 0.3, 1.0);

    // let mut triangles: Vec<Triangle> = Vec::new();

    // for indices in INDICES {
    //     let mut vertices: Vec<Vertex> = Vec::new();
    //     for index in indices {
    //         vertices.push(all_vertices[index as usize]);
    //     }
    //     triangles.push(Triangle::from_array(vertices[..].try_into().unwrap()));
    // }

    struct VertexObjects {
        vao: VertexArray,
        vbo: Buffer,
    }
    fn new_vertex_objects(vao: VertexArray, vbo: Buffer) -> VertexObjects {
        VertexObjects { vao: vao, vbo: vbo }
    }

    let mut all_vertex_objects: Vec<VertexObjects> = vec![];
    for triangle in pyramid.get_triangles() {
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
    let shader_program_2 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    let shader_program_3 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    let shader_program_4 = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_TEXTURE).unwrap();
    all_shader_programs.push(shader_program_1);
    all_shader_programs.push(shader_program_2);
    all_shader_programs.push(shader_program_3);
    all_shader_programs.push(shader_program_4);

    helpers::polygon_mode(PolygonMode::Fill);

    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));
    let (mut elapsed_time, mut previous_time): (u32, u32);
    let (mut translate_speed, mut rotation_speed, mut zoom_speed, mut walk_speed): (
        f32,
        f32,
        f32,
        f32,
    );
    let mut translation_delta = Vec3::zeros();
    let mut walk_delta: f32 = 0.0;

    elapsed_time = 0;

    // TODO: exercícios 10.10

    'main_loop: loop {
        // handle events this frame
        previous_time = elapsed_time;
        elapsed_time = sdl.get_ticks();
        translate_speed = (elapsed_time - previous_time) as f32 * 0.002;
        walk_speed = (elapsed_time - previous_time) as f32 * 0.002;
        rotation_speed = (elapsed_time - previous_time) as f32 * 0.01;
        zoom_speed = (elapsed_time - previous_time) as f32 * 0.1;

        'event_polling: while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                Event::Keyboard(key_event) => {
                    let pressed_value = key_event.is_pressed as i32 as f32;
                    match key_event.key.keycode {
                        Keycode::ESCAPE => break 'main_loop,
                        Keycode::A => translation_delta.x = translate_speed * -pressed_value,
                        Keycode::D => translation_delta.x = translate_speed * pressed_value,
                        Keycode::SPACE => translation_delta.y = translate_speed * -pressed_value,
                        Keycode::LCTRL => translation_delta.y = translate_speed * pressed_value,
                        Keycode::S => walk_delta = walk_speed * pressed_value,
                        Keycode::W => walk_delta = walk_speed * -pressed_value,
                        _ => (),
                    }
                    break 'event_polling;
                }
                Event::MouseMotion(motion_event) => {
                    main_camera.rotate(vec3(
                        -motion_event.y_delta as f32 * rotation_speed,
                        motion_event.x_delta as f32 * rotation_speed,
                        0.0,
                    ));
                }
                Event::MouseWheel(wheel_event) => {
                    main_camera.change_fov(wheel_event.y_delta as f32 * zoom_speed);
                }
                _ => (),
            }
        }
        main_camera.translate(translation_delta);
        main_camera.translate_forward(walk_delta);
        // now the events are clear.

        // here's where we could change the world state if we had some.

        // and then draw!
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            for (i, vertex_objects) in all_vertex_objects.iter().enumerate() {
                glActiveTexture(GL_TEXTURE0);
                texture1.bind();
                glActiveTexture(GL_TEXTURE1);
                texture2.bind();
                helpers::use_vertex_objects(&vertex_objects.vao, &vertex_objects.vbo);
                all_shader_programs[i].use_program();
                let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
                let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

                let model = Mat4::identity();
                let view = main_camera.look_at();
                let projection = perspective(main_camera.get_fov(), 1.0, 0.1, 100.0);

                all_shader_programs[i].set_matrix_4fv("model", model.as_ptr());
                all_shader_programs[i].set_matrix_4fv("view", view.as_ptr());
                all_shader_programs[i].set_matrix_4fv("projection", projection.as_ptr());
                all_shader_programs[i].set_4f("ourColor", [0.0, pulse, 0.0, 1.0]);
                all_shader_programs[i].set_1i("ourTexture1", 0);
                all_shader_programs[i].set_1i("ourTexture2", 1);
                all_shader_programs[i].set_1f("mixer", 0.2 as f32);
                glDrawArrays(GL_TRIANGLES, 0, 3);
            }
        }
        win.swap_window();
    }
}
