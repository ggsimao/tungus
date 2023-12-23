#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![feature(offset_of)]

// use assimp;
use beryllium::*;
use camera::{Camera, CameraController};
use controls::{Controller, SignalHandler};
use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
    ptr::null,
};
use data::{Buffer, BufferType, Framebuffer, PolygonMode, UniformBuffer, VertexArray};
use gl33::gl_core_types::*;
use gl33::gl_enumerations::*;
use gl33::gl_groups::*;
use gl33::global_loader::*;
use lighting::{DirectionalLight, FlashlightController, Lighting, PointLight, Spotlight};
use meshes::{BasicMesh, Draw, Skybox, Vertex};
use models::Model;
use nalgebra_glm::*;
use russimp::light::Light;
use scene::{Scene, SceneObject};
use screen::{Screen, ScreenController};
use shaders::{Shader, ShaderProgram, ShaderType};
use std::{cell::RefCell, ffi::c_void, path::Path, rc::Rc};
use systems::{Program, ProgramController};
use textures::{CubeMap, Material, Texture2D, TextureType};

pub mod camera;
pub mod controls;
pub mod data;
pub mod helpers;
pub mod lighting;
pub mod meshes;
pub mod models;
pub mod scene;
pub mod screen;
pub mod shaders;
pub mod systems;
pub mod textures;

// const SHADERS: &str = "./src/shaders/"
const REGULAR_VERT_SHADER: &str = "./src/shaders/regular_vert_shader.vs";
const FRAG_SHADER_OBJECT: &str = "./src/shaders/object_frag_shader.fs";
const FRAG_SHADER_BUFFER: &str = "./src/shaders/buffer_frag_shader.fs";
const SCREEN_VERT_SHADER: &str = "./src/shaders/screen_vert_shader.vs";
const SCREEN_FRAG_SHADER: &str = "./src/shaders/screen_frag_shader.fs";
const SKYBOX_VERT_SHADER: &str = "./src/shaders/skybox_vert_shader.vs";
const SKYBOX_FRAG_SHADER: &str = "./src/shaders/skybox_frag_shader.fs";

const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container2.png";
const CONTAINER_SPECULAR: &str = "./src/resources/textures/container2_specular.png";
const FACE_TEXTURE: &str = "./src/resources/textures/awesomeface.png";
const GRASS_TEXTURE: &str = "./src/resources/textures/grass.png";
const LAMP_TEXTURE: &str = "./src/resources/textures/glowstone.png";
const WINDOW_TEXTURE: &str = "./src/resources/textures/blending_transparent_window.png";

const ABSTRACT_CUBE: &str = "./src/resources/models/cube.obj";

const SKYBOX_FACES: [&str; 6] = [
    "./src/resources/textures/skybox/right.jpg",
    "./src/resources/textures/skybox/left.jpg",
    "./src/resources/textures/skybox/top.jpg",
    "./src/resources/textures/skybox/bottom.jpg",
    "./src/resources/textures/skybox/front.jpg",
    "./src/resources/textures/skybox/back.jpg",
];

const WINDOW_TITLE: &str = "Tungus";
const WINDOW_SIZE: (u32, u32) = (600, 600);

fn main() {
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // System initialization
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

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Camera initialization
    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));
    let mut mirror_camera;

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Lighting initialization
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

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // UBO initialization

    let matrices_ubo = UniformBuffer::new(0).unwrap();
    matrices_ubo.allocate(240);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Scene objects initialization
    let mut objects_list: Vec<&SceneObject> = vec![];

    // let backpack = Model::new(Path::new("./src/resources/models/backpack/backpack.obj"));

    let mut cube_map = CubeMap::new(TextureType::Diffuse);
    cube_map.load(SKYBOX_FACES);
    cube_map.set_wrapping(GL_CLAMP_TO_EDGE);
    cube_map.set_filters(GL_LINEAR, GL_LINEAR);
    let skybox = Skybox::new(cube_map);
    let sky_object = SceneObject::from(skybox);

    let cube = Model::new(Path::new(ABSTRACT_CUBE));

    let mut box_mesh = BasicMesh::cube(1.0);
    let mut cont_tex = Texture2D::new(TextureType::Diffuse);
    cont_tex.load(&Path::new(CONTAINER_TEXTURE));
    cont_tex.set_wrapping(GL_CLAMP_TO_EDGE);
    let mut cont_spec = Texture2D::new(TextureType::Specular);
    cont_spec.load(&Path::new(CONTAINER_SPECULAR));
    cont_spec.set_wrapping(GL_CLAMP_TO_EDGE);
    box_mesh.material = Material::new(vec![cont_tex], vec![cont_spec], 1.0);
    let box_object = SceneObject::from(box_mesh);
    objects_list.push(&box_object);

    let mut cube_object = SceneObject::from(cube);
    cube_object.scale(0.3);
    cube_object.translate(&vec3(0.0, 1.0, 0.0));
    objects_list.push(&cube_object);

    let mut lamp_mesh = BasicMesh::cube(1.0);
    let mut lamp_texture = Texture2D::new(TextureType::Diffuse);
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

    let canvas = SceneObject::from(BasicMesh::square(2.0));
    let mirror = SceneObject::from(BasicMesh::square(2.0));

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Shader initialization
    let shader_program_model =
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, FRAG_SHADER_OBJECT).unwrap();
    let shader_program_outline =
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, FRAG_SHADER_BUFFER).unwrap();
    let shader_program_screen =
        ShaderProgram::from_vert_frag(SCREEN_VERT_SHADER, SCREEN_FRAG_SHADER).unwrap();
    let shader_program_skybox =
        ShaderProgram::from_vert_frag(SKYBOX_VERT_SHADER, SKYBOX_FRAG_SHADER).unwrap();

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Screen initialization
    let mut screen = Screen::new(
        canvas,
        vec4(0.1, 0.1, 0.1, 1.0),
        framebuffer,
        &shader_program_screen,
        &matrices_ubo,
    );
    let mirrored_screen = Screen::new(
        mirror,
        vec4(0.1, 0.1, 0.1, 1.0),
        mirrored_framebuffer,
        &shader_program_screen,
        &matrices_ubo,
    );

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // This has an error for some reason
    data::polygon_mode(PolygonMode::Fill);

    let error;
    unsafe {
        error = glGetError();
    }
    println!("polygon_mode: {:?}", error);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Control initialization
    let camera_controller = CameraController::new();
    let flashlight_controller = FlashlightController::new();
    let program_controller = ProgramController::new();
    let screen_controller = ScreenController::new();
    let mut signal_handler = SignalHandler::new(&sdl);
    signal_handler.connect(camera_controller.clone());
    signal_handler.connect(flashlight_controller.clone());
    signal_handler.connect(program_controller.clone());
    signal_handler.connect(screen_controller.clone());

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Program loop
    let mut program_loop = Program { loop_active: true };
    let (mut elapsed_time, mut previous_time): (u32, u32);

    elapsed_time = 0;
    let mut cycle_time;

    while program_loop.loop_active {
        previous_time = elapsed_time;
        elapsed_time = sdl.get_ticks();
        cycle_time = (elapsed_time - previous_time) as f32;

        camera_controller.update_control_parameters(&mut |controller: &mut CameraController| {
            controller.set_speeds(cycle_time);
        });
        signal_handler.wait_event();
        camera_controller.process_signals(&mut main_camera);
        flashlight_controller.process_signals(&mut lighting.spot);
        program_controller.process_signals(&mut program_loop);
        screen_controller.process_signals(&mut screen);
        lighting.spot.pos = main_camera.get_pos();
        lighting.spot.dir = main_camera.get_dir();
        // let time_value: f32 = sdl.get_ticks() as f32 / 500.0;
        // let pulse: f32 = (time_value.sin() / 4.0) + 0.75;

        let scene = Scene {
            objects: &objects_list,
            skyboxes: &vec![&sky_object],
            object_shader: &shader_program_model,
            skybox_shader: &shader_program_skybox,
            outline_shader: &shader_program_outline,
            camera: &main_camera,
            lighting: &lighting,
        };
        mirror_camera = main_camera.invert();
        let mirrored_scene = Scene {
            objects: &objects_list,
            skyboxes: &vec![&sky_object],
            object_shader: &shader_program_model,
            skybox_shader: &shader_program_skybox,
            outline_shader: &shader_program_outline,
            camera: &mirror_camera,
            lighting: &lighting,
        };

        screen.draw_on_framebuffer(&scene);
        mirrored_screen.draw_on_framebuffer(&mirrored_scene);
        mirrored_screen.draw_on_another(&screen, 0.3, vec2(0.5, 0.5));
        screen.draw_on_screen();

        win.swap_window();
    }
}
