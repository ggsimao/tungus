#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![feature(offset_of)]

// use assimp;
use beryllium::*;
use camera::Camera;
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
    ptr::null,
};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use lighting::{DirectionalLight, Lighting, PointLight, Spotlight};
use meshes::{Draw, Mesh, Vertex};
use models::Model;
use nalgebra_glm::*;
use rendering::{Buffer, BufferType, Framebuffer, PolygonMode, VertexArray};
use russimp::light::Light;
use scene::{Scene, SceneObject};
use screen::Screen;
use shaders::{Shader, ShaderProgram, ShaderType};
use std::{ffi::c_void, path::Path};
use textures::{Material, Texture, TextureType};

pub mod camera;
pub mod helpers;
pub mod lighting;
pub mod meshes;
pub mod models;
pub mod rendering;
pub mod scene;
pub mod screen;
pub mod shaders;
pub mod textures;

const REGULAR_VERT_SHADER: &str = "./src/shaders/regular_vert_shader.vs";
const FRAG_SHADER_OBJECT: &str = "./src/shaders/object_frag_shader.fs";
const FRAG_SHADER_BUFFER: &str = "./src/shaders/buffer_frag_shader.fs";

const SCREEN_VERT_SHADER: &str = "./src/shaders/screen_vert_shader.vs";
const SCREEN_FRAG_SHADER: &str = "./src/shaders/screen_frag_shader.fs";

const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container2.png";
const CONTAINER_SPECULAR: &str = "./src/resources/textures/container2_specular.png";
const FACE_TEXTURE: &str = "./src/resources/textures/awesomeface.png";
const GRASS_TEXTURE: &str = "./src/resources/textures/grass.png";
const LAMP_TEXTURE: &str = "./src/resources/textures/glowstone.png";
const WINDOW_TEXTURE: &str = "./src/resources/textures/blending_transparent_window.png";

const WINDOW_TITLE: &str = "Tungus";
const WINDOW_SIZE: (u32, u32) = (600, 600);

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();
    sdl.gl_set_attribute(SdlGlAttr::StencilSize, 8).unwrap();

    let win = sdl
        .create_gl_window(
            WINDOW_TITLE,
            WindowPosition::XY(500, 50),
            WINDOW_SIZE.0,
            WINDOW_SIZE.1,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync);

    unsafe {
        let fun = |x: *const u8| win.get_proc_address(x as *const i8) as *const std::ffi::c_void;
        load_global_gl(&fun);
    }

    let framebuffer = Framebuffer::new().unwrap();
    framebuffer.setup_with_renderbuffer(WINDOW_SIZE);

    let mirrored_framebuffer = Framebuffer::new().unwrap();
    mirrored_framebuffer.setup_with_renderbuffer(WINDOW_SIZE);

    unsafe {
        glEnable(GL_DEPTH_TEST);
        glEnable(GL_STENCIL_TEST);
        glEnable(GL_BLEND);
        glEnable(GL_CULL_FACE);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        glStencilOp(GL_KEEP, GL_KEEP, GL_REPLACE);
    }

    let _ = sdl.set_relative_mouse_mode(true);

    let mut objects_list: Vec<&SceneObject> = vec![];

    // let backpack = Model::new(Path::new("./src/resources/models/backpack/backpack.obj"));

    let mut box_mesh = Mesh::cube(1.0);
    let mut window_tex = Texture::new(TextureType::Diffuse);
    window_tex.load(Path::new(WINDOW_TEXTURE));
    window_tex.set_wrapping(GL_CLAMP_TO_EDGE);
    box_mesh.material = Material::new(vec![window_tex], vec![], 1.0);
    let mut box_object = SceneObject::from(box_mesh);
    // box_object.translate(&vec3(0.0, 0.0, -1.0));
    objects_list.push(&box_object);

    let canvas = SceneObject::from(Mesh::square(2.0));
    let mirror = SceneObject::from(Mesh::square(2.0));

    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));
    let mut mirror_camera;

    let ambient = vec3(0.2, 0.2, 0.2);
    let diffuse = vec3(1.0, 1.0, 1.0);
    let specular = vec3(1.0, 1.0, 1.0);
    let attenuation = vec3(1.0, 0.5, 0.25);

    let sun = DirectionalLight::new(vec3(0.5, -1.0, 0.5), ambient, diffuse, specular);

    let mut lamps: [PointLight; 4] =
        [PointLight::new(vec3(0.0, 0.0, 0.0), ambient, diffuse, specular, attenuation); 4];
    lamps[0].pos = vec3(0.0, 2.0, 0.0);
    lamps[1].pos = vec3(-1.0, -2.0, -1.0);
    lamps[2].pos = vec3(1.0, 0.0, 1.0);
    lamps[3].pos = vec3(0.0, -10.0, 0.0);

    let (phi, gamma) = (15.0_f32.to_radians(), 20.0_f32.to_radians());

    let flashlight = Spotlight::new(
        main_camera.get_pos(),
        main_camera.get_dir(),
        ambient / 2.0,
        diffuse / 2.0,
        specular / 2.0,
        attenuation,
        15.0_f32.to_radians(),
        20.0_f32.to_radians(),
    );

    let mut lighting = Lighting {
        dir: sun,
        point: Vec::from(lamps),
        spot: flashlight,
    };

    let mut lamp_mesh = Mesh::cube(1.0);
    let mut lamp_texture = Texture::new(TextureType::Diffuse);
    lamp_texture.load(Path::new(LAMP_TEXTURE));
    lamp_mesh.material = Material::new(vec![lamp_texture], vec![], 1.0);
    let mut lamp_objects: Vec<SceneObject> = Vec::new();
    for i in 0..lamps.len() {
        let mut lamp_object = SceneObject::from(lamp_mesh.clone());
        lamp_object.translate(&lamps[i].pos);
        lamp_object.scale(0.1);
        lamp_objects.push(lamp_object);
    }
    for i in 0..lamps.len() {
        objects_list.push(&lamp_objects[i]);
    }

    let shader_program_model =
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, FRAG_SHADER_OBJECT).unwrap();
    let shader_program_outline =
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, FRAG_SHADER_BUFFER).unwrap();
    let shader_program_screen =
        ShaderProgram::from_vert_frag(SCREEN_VERT_SHADER, SCREEN_FRAG_SHADER).unwrap();

    let screen = Screen::new(
        canvas,
        vec4(0.1, 0.1, 0.1, 1.0),
        framebuffer,
        &shader_program_screen,
    );
    let mirrored_screen = Screen::new(
        mirror,
        vec4(0.1, 0.1, 0.1, 1.0),
        mirrored_framebuffer,
        &shader_program_screen,
    );

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
                        Keycode::F => {
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
        lighting.spot.phi = phi * flashlight_on as i32 as f32;
        lighting.spot.gamma = gamma * flashlight_on as i32 as f32;
        lighting.spot.pos = main_camera.get_pos();
        lighting.spot.dir = main_camera.get_dir();

        // let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
        // let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

        let scene = Scene {
            objects: &objects_list,
            object_shader: &shader_program_model,
            outline_shader: &shader_program_outline,
            camera: &main_camera,
            lighting: &lighting,
        };
        mirror_camera = main_camera.invert();
        let mirrored_scene = Scene {
            objects: &objects_list,
            object_shader: &shader_program_model,
            outline_shader: &shader_program_outline,
            camera: &mirror_camera,
            lighting: &lighting,
        };

        mirrored_screen.draw_on_framebuffer(&mirrored_scene);
        screen.draw_on_framebuffer(&scene);
        mirrored_screen.draw_on_another(&screen, 0.3, vec2(0.5, 0.5));
        screen.draw_on_screen();

        win.swap_window();
    }
}
