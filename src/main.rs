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
use model::{Hexahedron, Triangle, TriangularPyramid, Vertex};
use nalgebra_glm::*;
use rendering::{
    Buffer, BufferType, Draw, HexahedronDrawer, PolygonMode, TriangularPyramidDrawer, VertexArray,
};
use shaders::{Shader, ShaderProgram, ShaderType};
use std::{ffi::c_void, path::Path};
use systems::Camera;
use textures::Texture;

pub mod helpers;
pub mod model;
pub mod rendering;
pub mod shaders;
pub mod systems;
pub mod textures;

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";
const FRAG_SHADER_COLOR: &str = "./src/shaders/color_frag_shader.fs";
const FRAG_SHADER_RAINBOW: &str = "./src/shaders/rainbow_frag_shader.fs";
const FRAG_SHADER_TEXTURE: &str = "./src/shaders/texture_frag_shader.fs";
const FRAG_SHADER_LIGHT: &str = "./src/shaders/light_frag_shader.fs";

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

    let pyramid = TriangularPyramid::regular(1.0);
    // let all_colors: [Vec3; 4] = [
    //     vec3(1.0, 0.0, 0.0),
    //     vec3(0.0, 1.0, 0.0),
    //     vec3(0.0, 0.0, 1.0),
    //     vec3(0.0, 0.0, 0.0),
    // ];
    // let all_texcoords: [Vec2; 4] = [
    //     vec2(1.5, 0.0),
    //     vec2(3.0, 2.0),
    //     vec2(0.0, 2.0),
    //     vec2(1.5, 0.0),
    // ];
    // pyramid.colors_from_vectors(all_colors);
    // pyramid.tex_coords_from_vectors(all_texcoords);

    let pyramid_drawer = TriangularPyramidDrawer::new(pyramid);

    rendering::clear_color(0.2, 0.3, 0.3, 1.0);

    let object_vao = VertexArray::new().expect("Couldn't make a VAO");
    // object_vao.bind();

    let lamp_vao = VertexArray::new().expect("Couldn't make a VAO");
    // lamp_vao.bind();

    // let mut texture1 = Texture::new();
    // texture1.load(Path::new(CONTAINER_TEXTURE));
    // texture1.set_wrapping(GL_REPEAT);
    // texture1.set_filters(GL_NEAREST, GL_NEAREST);
    // let mut texture2 = Texture::new();
    // texture2.load(Path::new(FACE_TEXTURE));
    // texture2.set_wrapping(GL_REPEAT);
    // texture2.set_filters(GL_NEAREST, GL_NEAREST);

    let lamp = Hexahedron::cube(0.5);
    let lamp_drawer = HexahedronDrawer::new(lamp);

    let shader_program_pyramid =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_COLOR).unwrap();
    let shader_program_lamp =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_LIGHT).unwrap();

    rendering::polygon_mode(PolygonMode::Fill);

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

    'main_loop: loop {
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

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
        // glActiveTexture(GL_TEXTURE0);
        // texture1.bind();
        // glActiveTexture(GL_TEXTURE1);
        // texture2.bind();
        // let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
        // let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

        let pyramid_model = Mat4::identity();
        let pyramid_view = main_camera.look_at();
        let projection = perspective(1.0, main_camera.get_fov(), 0.1, 100.0);
        let normal = mat4_to_mat3(&pyramid_model.try_inverse().unwrap().transpose());

        // shader_program_pyramid.set_4f("ourColor", [0.0, pulse, 0.0, 1.0]);
        // shader_program_pyramid.set_1i("ourTexture1", 0);
        // shader_program_pyramid.set_1i("ourTexture2", 1);
        // shader_program_pyramid.set_1f("mixer", 0.2 as f32);
        shader_program_pyramid.use_program();
        shader_program_pyramid.set_matrix_4fv("model", pyramid_model.as_ptr());
        shader_program_pyramid.set_matrix_4fv("view", pyramid_view.as_ptr());
        shader_program_pyramid.set_matrix_4fv("projection", projection.as_ptr());
        shader_program_pyramid.set_matrix_3fv("normalMatrix", normal.as_ptr());
        shader_program_pyramid.set_3f("material.ambient", [1.0, 0.5, 0.31]);
        shader_program_pyramid.set_3f("material.diffuse", [1.0, 0.5, 0.31]);
        shader_program_pyramid.set_3f("material.specular", [0.5, 0.5, 0.5]);
        shader_program_pyramid.set_1f("material.shininess", 32.0);
        shader_program_pyramid.set_3f("objectColor", [1.0, 0.5, 0.31]);
        shader_program_pyramid.set_3f("lightColor", [1.0, 1.0, 1.0]);
        shader_program_pyramid.set_3f("light.position", [0.0, 0.7, 0.0]);
        shader_program_pyramid.set_3f("light.ambient", [0.2, 0.2, 0.2]);
        shader_program_pyramid.set_3f("light.diffuse", [0.5, 0.9, 0.5]);
        shader_program_pyramid.set_3f("light.specular", [1.0, 1.0, 1.0]);
        shader_program_pyramid.set_3f("viewPos", main_camera.get_pos().into());

        object_vao.bind();
        pyramid_drawer.ready_buffers();
        pyramid_drawer.draw();

        let lamp_scale = scaling(&vec3(0.1, 0.1, 0.1));
        let lamp_trans = translation(&vec3(0.0, 0.7, 0.0));
        let lamp_model = lamp_trans * lamp_scale;
        let lamp_view = main_camera.look_at();

        shader_program_lamp.use_program();
        shader_program_lamp.set_matrix_4fv("model", lamp_model.as_ptr());
        shader_program_lamp.set_matrix_4fv("view", lamp_view.as_ptr());
        shader_program_lamp.set_matrix_4fv("projection", projection.as_ptr());

        lamp_vao.bind();
        lamp_drawer.ready_buffers();
        lamp_drawer.draw();

        win.swap_window();
    }
}
