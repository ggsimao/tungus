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
use lighting::{DirectionalLight, Spotlight};
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
pub mod lighting;
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
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container2.png";
const CONTAINER_SPECULAR: &str = "./src/resources/textures/container2_specular.png";
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

    let all_texcoords: [Vec2; 8] = [
        vec2(1.0, 0.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, 0.0),
    ];

    let mut cube_drawers: Vec<HexahedronDrawer> = Vec::new();

    for _ in 0..8 {
        let mut cube = Hexahedron::cube(1.0);
        cube.tex_coords_from_vectors(all_texcoords);
        let cube_drawer = HexahedronDrawer::new(cube);
        cube_drawers.push(cube_drawer);
    }

    rendering::clear_color(0.2, 0.3, 0.3, 1.0);

    let object_vao = VertexArray::new().expect("Couldn't make a VAO");
    let lamp_vao = VertexArray::new().expect("Couldn't make a VAO");

    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));

    let mut texture1 = Texture::new();
    texture1.load(Path::new(CONTAINER_TEXTURE));
    texture1.set_wrapping(GL_REPEAT);
    texture1.set_filters(GL_NEAREST, GL_NEAREST);
    let mut texture2 = Texture::new();
    texture2.load(Path::new(CONTAINER_SPECULAR));
    texture2.set_wrapping(GL_REPEAT);
    texture2.set_filters(GL_NEAREST, GL_NEAREST);

    let sun = DirectionalLight::new(vec3(0.5, -1.0, 0.5));

    let mut lamp_drawers: Vec<HexahedronDrawer> = Vec::new();

    let mut flashlight = Spotlight::new(
        main_camera.get_pos(),
        main_camera.get_dir(),
        30.0_f32.to_radians(),
        35.0_f32.to_radians(),
    );

    for _ in 0..8 {
        let mut cube = Hexahedron::cube(1.0);
        cube.tex_coords_from_vectors(all_texcoords);
        let cube_drawer = HexahedronDrawer::new(cube);
        lamp_drawers.push(cube_drawer);
    }

    let shader_program_cube =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_COLOR).unwrap();
    let shader_program_lamp =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_LIGHT).unwrap();

    rendering::polygon_mode(PolygonMode::Fill);

    let (mut elapsed_time, mut previous_time): (u32, u32);
    let (mut translate_speed, mut rotation_speed, mut zoom_speed): (f32, f32, f32);
    let (mut walk_delta, mut ascend_delta, mut side_delta): (f32, f32, f32) = (0.0, 0.0, 0.0);
    let mut flashlight_on = false;

    elapsed_time = 0;

    'main_loop: loop {
        previous_time = elapsed_time;
        elapsed_time = sdl.get_ticks();
        translate_speed = (elapsed_time - previous_time) as f32 * 0.002;
        rotation_speed = (elapsed_time - previous_time) as f32 * 0.01;
        zoom_speed = (elapsed_time - previous_time) as f32 * 0.1;

        'event_polling: while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                Event::Keyboard(key_event) => {
                    let pressed_value = key_event.is_pressed as i32 as f32;
                    match key_event.key.keycode {
                        Keycode::ESCAPE => break 'main_loop,
                        Keycode::A => side_delta = translate_speed * -pressed_value,
                        Keycode::D => side_delta = translate_speed * pressed_value,
                        Keycode::SPACE => ascend_delta = translate_speed * pressed_value,
                        Keycode::LCTRL => ascend_delta = translate_speed * -pressed_value,
                        Keycode::S => walk_delta = translate_speed * pressed_value,
                        Keycode::W => walk_delta = translate_speed * -pressed_value,
                        Keycode::Q => {
                            if pressed_value != 0.0 {
                                flashlight_on = !flashlight_on
                            }
                        }
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
        main_camera.translate_longitudinal(side_delta);
        main_camera.translate_forward(walk_delta);
        main_camera.translate_vertical(ascend_delta);

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
        // let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
        // let pulse: f32 = (time_value.sin() / 4.0) + 0.75;
        let lamp_positions = [
            vec3(0.0, 2.0, 0.0),
            vec3(-1.0, -2.0, -1.0),
            vec3(1.0, 0.0, 1.0),
            vec3(0.0, -10.0, 0.0),
        ];
        let cube_view = main_camera.look_at();
        let projection = perspective(1.0, main_camera.get_fov(), 0.1, 100.0);
        shader_program_cube.use_program();
        shader_program_cube.set_matrix_4fv("view", cube_view.as_ptr());
        shader_program_cube.set_matrix_4fv("projection", projection.as_ptr());
        shader_program_cube.set_1i("material.diffuse", texture1.0 as i32 - 1);
        unsafe {
            glActiveTexture(GL_TEXTURE0);
        }
        texture1.bind();
        shader_program_cube.set_1i("material.specular", texture2.0 as i32 - 1);
        shader_program_cube.set_1f("material.shininess", 128.0);
        shader_program_cube.set_3f("objectColor", [1.0, 1.0, 1.0]);
        shader_program_cube.set_3f("lightColor", [1.0, 1.0, 1.0]);
        shader_program_cube.set_3f(
            "dirLight.direction",
            [sun.angle.x, sun.angle.y, sun.angle.z],
        );
        shader_program_cube.set_3f("dirLight.ambient", [0.2, 0.2, 0.2]);
        shader_program_cube.set_3f("dirLight.diffuse", [1.0, 1.0, 1.0]);
        shader_program_cube.set_3f("dirLight.specular", [1.0, 1.0, 1.0]);
        shader_program_cube.set_3f("viewPos", main_camera.get_pos().into());
        unsafe {
            glActiveTexture(GL_TEXTURE1);
        }
        texture2.bind();
        object_vao.bind();

        for i in 0..8 {
            let mut cube_model = Mat4::identity();
            cube_model = translate(
                &cube_model,
                &vec3(
                    (-1.0_f32).powf(i as f32 + 1.0) * 1.0,
                    (-1.0_f32).powf((i / 4) as f32 + 1.0) * 1.0,
                    (-1.0_f32).powf((i / 2) as f32 + 1.0) * 1.0,
                ),
            );
            let normal = mat4_to_mat3(&cube_model.try_inverse().unwrap().transpose());

            shader_program_cube.set_matrix_4fv("model", cube_model.as_ptr());
            shader_program_cube.set_matrix_3fv("normalMatrix", normal.as_ptr());
            shader_program_cube.set_3f("material.diffuse", [1.0, 1.0, 1.0]);
            shader_program_cube.set_3f("material.specular", [1.0, 1.0, 1.0]);
            shader_program_cube.set_1f("material.shininess", 32.0);
            for i in 0..4 {
                shader_program_cube.set_3f(
                    format!("pointLights[{}].position", i).as_str(),
                    [
                        lamp_positions[i].x,
                        lamp_positions[i].y,
                        lamp_positions[i].z,
                    ],
                );
                shader_program_cube.set_1f(format!("pointLights[{}].constant", i).as_str(), 1.0);
                shader_program_cube.set_1f(format!("pointLights[{}].linear", i).as_str(), 0.5);
                shader_program_cube.set_1f(format!("pointLights[{}].quadratic", i).as_str(), 0.25);
                shader_program_cube.set_3f(
                    format!("pointLights[{}].ambient", i).as_str(),
                    [0.2, 0.2, 0.2],
                );
                shader_program_cube.set_3f(
                    format!("pointLights[{}].diffuse", i).as_str(),
                    [1.0, 1.0, 1.0],
                );
                shader_program_cube.set_3f(
                    format!("pointLights[{}].specular", i).as_str(),
                    [1.0, 1.0, 1.0],
                );
            }
            shader_program_cube.set_3f(
                "spotlight.position",
                [
                    main_camera.get_pos().x,
                    main_camera.get_pos().y,
                    main_camera.get_pos().z,
                ],
            );
            shader_program_cube.set_3f(
                "spotlight.direction",
                [
                    main_camera.get_dir().x,
                    main_camera.get_dir().y,
                    main_camera.get_dir().z,
                ],
            );
            shader_program_cube.set_3f("spotlight.ambient", [0.2, 0.2, 0.2]);
            shader_program_cube.set_3f("spotlight.diffuse", [0.5, 0.5, 0.5]);
            shader_program_cube.set_3f("spotlight.specular", [0.5, 0.5, 0.5]);
            if flashlight_on {
                shader_program_cube.set_1f("spotlight.phiCos", flashlight.phi.cos());
                shader_program_cube.set_1f("spotlight.gammaCos", flashlight.gamma.cos());
            } else {
                shader_program_cube.set_1f("spotlight.phiCos", 1.0);
                shader_program_cube.set_1f("spotlight.gammaCos", 1.0);
            }

            cube_drawers[i].ready_buffers();
            cube_drawers[i].draw();
        }
        let lamp_scale = scaling(&vec3(0.1, 0.1, 0.1));
        let lamp_view = main_camera.look_at();
        shader_program_lamp.use_program();
        shader_program_lamp.set_matrix_4fv("view", lamp_view.as_ptr());
        shader_program_lamp.set_matrix_4fv("projection", projection.as_ptr());
        lamp_vao.bind();
        for i in 0..4 {
            let lamp_trans = translation(&lamp_positions[i]);
            let lamp_model = lamp_trans * lamp_scale;

            shader_program_lamp.set_matrix_4fv("model", lamp_model.as_ptr());

            lamp_drawers[i].ready_buffers();
            lamp_drawers[i].draw();
        }

        win.swap_window();
    }
}
