#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![feature(offset_of)]

const WINDOW_TITLE: &str = "Tungus";

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
use lighting::{DirectionalLight, PointLight, Spotlight};
use meshes::{Draw, Mesh, Vertex};
use models::Model;
use nalgebra_glm::*;
use rendering::{Buffer, BufferType, PolygonMode, VertexArray};
use russimp::light::Light;
use scene::SceneObject;
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
pub mod shaders;
pub mod textures;

const VERT_SHADER: &str = "./src/shaders/vert_shader.vs";
const FRAG_SHADER_COLOR: &str = "./src/shaders/color_frag_shader.fs";
const FRAG_SHADER_BUFFER: &str = "./src/shaders/buffer_frag_shader.fs";
const FRAG_SHADER_TEXTURE: &str = "./src/shaders/texture_frag_shader.fs";
const FRAG_SHADER_LIGHT: &str = "./src/shaders/light_frag_shader.fs";

const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container2.png";
const CONTAINER_SPECULAR: &str = "./src/resources/textures/container2_specular.png";
const FACE_TEXTURE: &str = "./src/resources/textures/awesomeface.png";

struct Lighting {
    pub dir: DirectionalLight,
    pub point: Vec<PointLight>,
    pub spot: Spotlight,
}

fn draw_scene(
    scene: &Vec<&SceneObject>,
    object_shader: &ShaderProgram,
    outline_shader: &ShaderProgram,
    camera: &Camera,
    lighting: &Lighting,
) {
    let projection = &perspective(1.0, camera.get_fov(), 0.1, 100.0);
    let reinitialize_object_shader = || {
        object_shader.use_program();
        object_shader.set_view(&camera);

        object_shader.set_directional_light("dirLight", &lighting.dir);
        object_shader.set_matrix_4fv("projectionMatrix", projection);
        for (i, point) in lighting.point.iter().enumerate() {
            object_shader.set_point_light(format!("pointLights[{}]", i).as_str(), &point);
        }
        object_shader.set_spotlight("spotlight", &lighting.spot);
    };
    let reinitialize_outline_shader = || {
        outline_shader.use_program();
        outline_shader.set_view(&camera);
        outline_shader.set_matrix_4fv("projectionMatrix", projection);
    };

    let mut ordered_scene = scene.clone();

    let distance_compare = |a: &&SceneObject, b: &&SceneObject| {
        let distance_a = length(&(camera.get_pos() - a.get_pos()));
        let distance_b = length(&(camera.get_pos() - b.get_pos()));
        distance_b.partial_cmp(&distance_a).unwrap()
    };

    ordered_scene.sort_by(distance_compare);

    reinitialize_object_shader();
    for object in ordered_scene {
        object.draw(&object_shader);
        if object.has_outline() {
            reinitialize_outline_shader();
            object.draw_outline(&outline_shader);
            reinitialize_object_shader();
        }
    }
}

fn draw_lamps<T: Draw>(objects: &Vec<T>, shader: &ShaderProgram, camera: &Camera) {
    let lamp_positions = [
        vec3(0.0, 2.0, 0.0),
        vec3(-1.0, -2.0, -1.0),
        vec3(1.0, 0.0, 1.0),
        vec3(0.0, -5.0, 0.0),
    ];
    let lamp_scale = scaling(&vec3(0.1, 0.1, 0.1));
    let projection = perspective(1.0, camera.get_fov(), 0.1, 100.0);
    shader.use_program();
    shader.set_view(&camera);
    shader.set_matrix_4fv("projectionMatrix", &projection);
    for i in 0..4 {
        let lamp_trans = translation(&lamp_positions[i]);
        let lamp_model = lamp_trans * lamp_scale;

        shader.set_matrix_4fv("modelMatrix", &lamp_model);

        objects[i].draw(&shader);
    }
}

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

        glEnable(GL_DEPTH_TEST);
        glEnable(GL_STENCIL_TEST);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        glStencilOp(GL_KEEP, GL_KEEP, GL_REPLACE);
    }

    let _ = sdl.set_relative_mouse_mode(true);

    let mut scene: Vec<&SceneObject> = vec![];

    // let backpack = Model::new(Path::new("./src/resources/models/backpack/backpack.obj"));
    let mut grass_mesh = Mesh::square(1.0);
    let mut grass_tex = Texture::new(TextureType::Diffuse);
    grass_tex.load(Path::new("./src/resources/textures/grass.png"));
    grass_tex.set_wrapping(GL_CLAMP_TO_EDGE);
    grass_mesh.material = Material::new(vec![grass_tex], vec![], 1.0);
    let mut grass: SceneObject = SceneObject::from(grass_mesh);
    grass.set_outline(vec3(0.04, 0.28, 0.26));
    grass.enable_outline(true);
    scene.push(&grass);

    let mut window_mesh = Mesh::square(1.0);
    let mut window_tex = Texture::new(TextureType::Diffuse);
    window_tex.load(Path::new(
        "./src/resources/textures/blending_transparent_window.png",
    ));
    window_tex.set_wrapping(GL_CLAMP_TO_EDGE);
    window_mesh.material = Material::new(vec![window_tex], vec![], 1.0);
    let mut window1 = SceneObject::from(window_mesh.clone());
    window1.translate(&vec3(0.0, 0.0, -1.0));
    scene.push(&window1);

    let mut window2 = SceneObject::from(window_mesh);
    window2.translate(&vec3(0.0, 0.0, 1.0));
    scene.push(&window2);

    rendering::clear_color(0.2, 0.3, 0.3, 1.0);

    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));

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

    let mut lamp_meshes: Vec<Mesh> = Vec::new();
    for _ in 0..4 {
        let cube = Mesh::cube(1.0);
        lamp_meshes.push(cube);
    }

    let shader_program_model =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_COLOR).unwrap();
    let shader_program_lamp =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_LIGHT).unwrap();
    let shader_program_outline =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER_BUFFER).unwrap();

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

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
        }

        // let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
        // let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

        draw_lamps(&lamp_meshes, &shader_program_lamp, &main_camera);

        draw_scene(
            &scene,
            &shader_program_model,
            &shader_program_outline,
            &main_camera,
            &lighting,
        );

        win.swap_window();
    }
}
