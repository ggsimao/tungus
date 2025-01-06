#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(clippy::single_match)]
#![allow(clippy::zero_ptr)]
#![feature(offset_of)]
#![feature(div_duration)]
#![feature(core_intrinsics)]

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
use rand::Rng;
use russimp::light::Light;
use spatial::Spatial;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    f32::consts::PI,
    ffi::c_void,
    path::Path,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};
use utils::{RTController, RandomTransform};

use camera::{Camera, CameraController};
use controls::{Controller, SignalHandler};
use data::{Buffer, BufferType, Framebuffer, PolygonMode, UniformBuffer, VertexArray};
use lighting::{DirectionalLight, FlashlightController, Lighting, PointLight, Spotlight};
use meshes::{BasicMesh, Canvas, Draw, Skybox, Vertex};
use models::Model;
use scene::{Scene, SceneController, SceneObject, SceneParameters};
use screen::{Screen, ScreenController};
use shaders::{Shader, ShaderProgram, ShaderType};
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
pub mod spatial;
pub mod systems;
pub mod textures;
pub mod utils;

// const SHADERS: &str = "./src/shaders/"
const REGULAR_VERT_SHADER: &str = "./src/shaders/regular_vert_shader.vs";
const OBJECT_FRAG_SHADER: &str = "./src/shaders/object_frag_shader.fs";
const DEBUG_GEO_SHADER: &str = "./src/shaders/debug_geo_shader.gs";
const DEBUG_FRAG_SHADER: &str = "./src/shaders/debug_frag_shader.fs";
const BUFFER_FRAG_SHADER: &str = "./src/shaders/buffer_frag_shader.fs";
const SCREEN_VERT_SHADER: &str = "./src/shaders/screen_vert_shader.vs";
const SCREEN_FRAG_SHADER: &str = "./src/shaders/screen_frag_shader.fs";
const SKYBOX_VERT_SHADER: &str = "./src/shaders/skybox_vert_shader.vs";
const SKYBOX_FRAG_SHADER: &str = "./src/shaders/skybox_frag_shader.fs";
const SHADOW_VERT_SHADER: &str = "./src/shaders/shadow_vert_shader.vs";
const SHADOW_FRAG_SHADER: &str = "./src/shaders/shadow_frag_shader.fs";

const WALL_TEXTURE: &str = "./src/resources/textures/wall.jpg";
const CONTAINER_TEXTURE: &str = "./src/resources/textures/container2.png";
const CONTAINER_SPECULAR: &str = "./src/resources/textures/container2_specular.png";
const FACE_TEXTURE: &str = "./src/resources/textures/awesomeface.png";
const GRASS_TEXTURE: &str = "./src/resources/textures/grass.png";
const LAMP_TEXTURE: &str = "./src/resources/textures/glowstone.png";
const WINDOW_TEXTURE: &str = "./src/resources/textures/window_diff.png";
const WINDOW_SPECULAR: &str = "./src/resources/textures/window_spec.png";
const WOOD_TEXTURE: &str = "./src/resources/textures/wood.jpg";

const ABSTRACT_CUBE: &str = "./src/resources/models/cube/untitled.obj";
const ROCK_1: &str = "./src/resources/models/rocks/rock.obj";
const BACKPACK: &str = "./src/resources/models/backpack/backpack.obj";

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

const INSTANCES: usize = 1000;

const INPUT_POLL_INTERVAL: Duration = Duration::from_micros(2000);

fn init_shaders() -> HashMap<&'static str, ShaderProgram> {
    let mut shader_map = HashMap::new();
    shader_map.insert(
        "model",
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, OBJECT_FRAG_SHADER).unwrap(),
    );
    shader_map.insert(
        "debug",
        ShaderProgram::from_vert_geo_frag(REGULAR_VERT_SHADER, DEBUG_GEO_SHADER, DEBUG_FRAG_SHADER)
            .unwrap(),
    );
    shader_map.insert(
        "outline",
        ShaderProgram::from_vert_frag(REGULAR_VERT_SHADER, BUFFER_FRAG_SHADER).unwrap(),
    );
    shader_map.insert(
        "screen",
        ShaderProgram::from_vert_frag(SCREEN_VERT_SHADER, SCREEN_FRAG_SHADER).unwrap(),
    );
    shader_map.insert(
        "skybox",
        ShaderProgram::from_vert_frag(SKYBOX_VERT_SHADER, SKYBOX_FRAG_SHADER).unwrap(),
    );
    shader_map.insert(
        "shadow",
        ShaderProgram::from_vert_frag(SHADOW_VERT_SHADER, SHADOW_FRAG_SHADER).unwrap(),
    );
    shader_map
}

fn init_sdl() -> SDL {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();
    sdl.gl_set_attribute(SdlGlAttr::StencilSize, 8).unwrap();
    sdl
}

fn init_glwindow(sdl: &SDL) -> GlWindow {
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
    win
}

fn init_lighting(camera: &Camera) -> Lighting {
    let ambient = vec3(0.2, 0.2, 0.2);
    let diffuse = vec3(1.0, 1.0, 1.0);
    let specular = vec3(1.0, 1.0, 1.0);
    let attenuation = vec3(1.0, 0.5, 0.25);

    let sun = DirectionalLight::new(vec3(1.0, -2.0, 1.5), ambient, diffuse, specular);

    let mut lamps: [PointLight; 4] =
        [PointLight::new(vec3(0.0, 0.0, 0.0), ambient, diffuse, specular, attenuation); 4];
    lamps[0].pos = vec3(0.0, 2.0, 0.0);
    lamps[1].pos = vec3(-1.0, -2.0, -1.0);
    lamps[2].pos = vec3(1.0, 0.0, 1.0);
    lamps[3].pos = vec3(0.0, -10.0, 0.0);

    let flashlight = Spotlight::new(
        camera.get_pos(),
        camera.get_dir(),
        ambient / 2.0,
        diffuse / 2.0,
        specular / 2.0,
        attenuation,
        15.0_f32.to_radians(),
        20.0_f32.to_radians(),
    );

    Lighting {
        dir: sun,
        point: Vec::from(lamps),
        spot: flashlight,
    }
}

fn init_obj_list(lamps: &Vec<PointLight>) -> Vec<SceneObject> {
    let mut objects_list: Vec<SceneObject> = vec![];

    let rock_model = Model::new(Path::new(ROCK_1));
    let mut rock_object = SceneObject::from(rock_model);
    rock_object.scale(&vec3(0.1, 0.1, 0.1));
    rock_object.add_instances(INSTANCES - 1);
    for i in 0..INSTANCES {
        RandomTransform::position(
            rock_object.get_instance_mut(i as isize),
            (-100.0, 100.0),
            (-100.0, 100.0),
            (-100.0, 100.0),
        );
    }
    objects_list.push(rock_object);

    let mut box_mesh = BasicMesh::cube(1.0);
    let cont_tex = Texture2D::setup_new(
        TextureType::Diffuse,
        &Path::new(CONTAINER_TEXTURE),
        GL_CLAMP_TO_EDGE,
    );
    let cont_spec = Texture2D::setup_new(
        TextureType::Specular,
        &Path::new(CONTAINER_SPECULAR),
        GL_CLAMP_TO_EDGE,
    );
    box_mesh.material = Material::new(vec![cont_tex], vec![cont_spec], 8.0);
    let mut box_object = SceneObject::from(box_mesh);
    box_object.set_outline(vec4(0.5, 0.2, 0.3, 1.0));
    objects_list.push(box_object);

    let mut wind_mesh = BasicMesh::square(1.0);
    let wind_tex = Texture2D::setup_new(
        TextureType::Diffuse,
        &Path::new(WINDOW_TEXTURE),
        GL_CLAMP_TO_EDGE,
    );
    let wind_spec = Texture2D::setup_new(
        TextureType::Specular,
        &Path::new(WINDOW_SPECULAR),
        GL_CLAMP_TO_EDGE,
    );
    wind_mesh.material = Material::new(vec![wind_tex], vec![wind_spec], 8.0);
    let mut wind_object = SceneObject::from(wind_mesh);
    wind_object.translate(&vec3(0.0, 1.5, -1.5));
    objects_list.push(wind_object);

    let mut lamp_mesh = BasicMesh::cube(1.0);
    let mut lamp_texture = Texture2D::setup_new(
        TextureType::Diffuse,
        &Path::new(LAMP_TEXTURE),
        GL_CLAMP_TO_EDGE,
    );
    lamp_mesh.material = Material::new(vec![lamp_texture], vec![], 32.0);
    let mut lamp_object = SceneObject::from(lamp_mesh.clone());
    if lamps.len() > 0 {
        lamp_object.add_instances(lamps.len() - 1);
    }
    for i in 0..lamps.len() {
        lamp_object
            .get_instance_mut(i as isize)
            .translate(&lamps[i].pos);
        lamp_object
            .get_instance_mut(i as isize)
            .scale(&vec3(0.1, 0.1, 0.1));
    }
    objects_list.push(lamp_object);

    let mut floor = BasicMesh::square(10.0);
    let floor_tex = Texture2D::setup_new(TextureType::Diffuse, &Path::new(WOOD_TEXTURE), GL_REPEAT);
    floor.material = Material::new(vec![floor_tex], vec![], 16.0);
    let mut floor_object = SceneObject::from(floor);
    floor_object.rotate(-PI / 2.0, &vec3(1.0, 0.0, 0.0));
    floor_object.translate(&vec3(0.0, -1.5, 0.0));
    objects_list.push(floor_object);

    objects_list
}

fn init_skybox() -> Skybox {
    let mut cube_map = CubeMap::new(TextureType::Diffuse);
    cube_map.load(SKYBOX_FACES);
    cube_map.set_wrapping(GL_CLAMP_TO_EDGE);
    cube_map.set_filters(GL_LINEAR, GL_LINEAR);
    let skybox = Skybox::new(cube_map);
    skybox
}

fn init_random_transforms(quantity: usize) -> Vec<RandomTransform> {
    let mut rts = vec![];
    for _ in 0..quantity {
        let mut rng = rand::thread_rng();
        rts.push(RandomTransform::continuous(
            0.1,
            0.1,
            rng.gen_range(0..=1000),
            rng.gen_range(0..=1000),
        ));
    }
    rts
}

struct ControllerHub<'a> {
    pub camera: Rc<RefCell<CameraController>>,
    pub flashlight: Rc<RefCell<FlashlightController>>,
    pub program: Rc<RefCell<ProgramController>>,
    pub screen: Rc<RefCell<ScreenController>>,
    pub scene: Rc<RefCell<SceneController>>,
    pub rt: Rc<RefCell<RTController>>,
    pub handler: Rc<RefCell<SignalHandler<'a>>>,
}

impl<'a> ControllerHub<'a> {
    pub fn init(sdl: &'a SDL) -> Self {
        let camera_controller = CameraController::new();
        let flashlight_controller = FlashlightController::new();
        let program_controller = ProgramController::new();
        let screen_controller = ScreenController::new();
        let scene_controller = SceneController::new();
        let rt_controller = RTController::new();
        let mut signal_handler = SignalHandler::new(&sdl);
        signal_handler
            .connect(unsafe { Weak::from_raw(Rc::downgrade(&camera_controller).into_raw()) });
        signal_handler
            .connect(unsafe { Weak::from_raw(Rc::downgrade(&flashlight_controller).into_raw()) });
        signal_handler
            .connect(unsafe { Weak::from_raw(Rc::downgrade(&program_controller).into_raw()) });
        signal_handler
            .connect(unsafe { Weak::from_raw(Rc::downgrade(&screen_controller).into_raw()) });
        signal_handler
            .connect(unsafe { Weak::from_raw(Rc::downgrade(&scene_controller).into_raw()) });
        signal_handler.connect(unsafe { Weak::from_raw(Rc::downgrade(&rt_controller).into_raw()) });
        ControllerHub {
            camera: camera_controller,
            flashlight: flashlight_controller,
            program: program_controller,
            screen: screen_controller,
            scene: scene_controller,
            rt: rt_controller,
            handler: Rc::new(RefCell::new(signal_handler)),
        }
    }

    pub fn update(
        &'a self,
        cycle_time: f32,
        camera: &mut Camera,
        flashlight: &mut Spotlight,
        prog: &mut Program,
        screen: &mut Screen,
        params: &mut SceneParameters,
        rts: &mut Vec<RandomTransform>,
    ) {
        self.camera
            .update_control_parameters(&mut |controller: &mut CameraController| {
                controller.set_speeds(cycle_time);
            });
        (*self.handler).borrow_mut().wait_event();
        self.camera.process_signals(camera);
        self.flashlight.process_signals(flashlight);
        self.program.process_signals(prog);
        self.screen.process_signals(screen);
        self.scene.process_signals(params);
        self.rt.process_signals(rts);
        // return new_keys_state;
    }
}

struct App {
    pub sdl: SDL,
    pub win: GlWindow,
}

impl App {
    pub fn init() -> Self {
        let sdl = init_sdl();
        let win = init_glwindow(&sdl);

        unsafe {
            glEnable(GL_MULTISAMPLE);
            glEnable(GL_DEPTH_TEST);
            glEnable(GL_STENCIL_TEST);
            glEnable(GL_BLEND);
            glEnable(GL_CULL_FACE);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
            glStencilOp(GL_KEEP, GL_KEEP, GL_REPLACE);
        }

        let _ = sdl.set_relative_mouse_mode(true);

        App { sdl, win }
    }
}

fn main() {
    // System initialization
    let app = App::init();

    let mut main_camera = Camera::new(vec3(0.0, 0.0, -2.0));

    let mut lighting = init_lighting(&main_camera);

    let matrices_ubo = UniformBuffer::new(0).unwrap();
    matrices_ubo.allocate(240);

    // Scene objects initialization
    let skybox = init_skybox();
    let mut objects_list: Vec<SceneObject> = init_obj_list(&lighting.point);
    let canvas = SceneObject::from(Canvas::new());
    let mirror = SceneObject::from(Canvas::new());

    let shaders = init_shaders();

    let mut rts = init_random_transforms(INSTANCES);

    // Screen initialization
    let mut screen = Screen::new(
        canvas,
        vec4(0.1, 0.1, 0.1, 1.0),
        WINDOW_SIZE,
        shaders["screen"],
        matrices_ubo,
    );
    let mut mirrored_screen = Screen::new(
        mirror,
        vec4(0.1, 0.1, 0.1, 1.0),
        WINDOW_SIZE,
        shaders["screen"],
        matrices_ubo,
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
    let control_hub = ControllerHub::init(&app.sdl);
    (*control_hub.rt).borrow_mut().add_rts(&rts);

    // Program loop
    let mut program_loop = Program {
        loop_active: true,
        // timer: &|| app.sdl.get_ticks(),
    };
    let (mut elapsed_time, mut previous_time): (u32, u32);

    elapsed_time = 0;
    let mut cycle_time;

    let mut scene_params = SceneParameters::init();

    let mut total_update: Duration = Duration::new(0, 0);
    let mut total_instances: Duration = Duration::new(0, 0);
    let mut total_draw: Duration = Duration::new(0, 0);
    let mut total_cycles: u32 = 0;

    let mut last_update = Instant::now();

    while program_loop.loop_active {
        let start_of_frame = Instant::now();
        total_cycles += 1;

        previous_time = elapsed_time;
        elapsed_time = app.sdl.get_ticks();
        cycle_time = (elapsed_time - previous_time) as f32;

        let start_update = Instant::now();
        if last_update.elapsed() >= INPUT_POLL_INTERVAL {
            control_hub.update(
                cycle_time,
                &mut main_camera,
                &mut lighting.spot,
                &mut program_loop,
                &mut screen,
                &mut scene_params,
                &mut rts,
            );
            last_update = Instant::now();
        }
        total_update += start_update.elapsed();

        lighting.spot.pos = main_camera.get_pos();
        lighting.spot.dir = main_camera.get_dir();

        let start_instances = Instant::now();
        for i in 0..INSTANCES {
            let inst = objects_list[0].get_instance_mut(i.try_into().unwrap());
            rts[i].rotate(inst);
            rts[i].translate(inst);
        }
        total_instances += start_instances.elapsed();

        let mut scene = Scene {
            objects: objects_list.clone(),
            skyboxes: &vec![&skybox],
            object_shader: shaders["model"],
            skybox_shader: shaders["skybox"],
            outline_shader: shaders["outline"],
            shadow_shader: shaders["shadow"],
            debug_shader: shaders["debug"],
            camera: main_camera,
            lighting: &lighting,
            params: scene_params,
        };

        shaders["model"].use_program();
        shaders["model"].set_1f("time", app.sdl.get_ticks() as f32 / 500.0);

        let start_draw = Instant::now();
        screen.draw_on_framebuffer(scene.borrow_mut());
        let mut mirrored_scene = scene.mirrored();
        mirrored_screen.draw_on_framebuffer(mirrored_scene.borrow_mut());
        mirrored_screen.draw_on_another(&screen, 0.3, vec2(0.5, 0.5));
        screen.draw_on_screen();
        total_draw += start_draw.elapsed();

        app.win.swap_window();
        let fps = Duration::from_secs(1).div_duration_f32(start_of_frame.elapsed());
        let average_update = total_update / total_cycles;
        let average_instances = total_instances / total_cycles;
        let average_draw = total_draw / total_cycles;
        let mut info: String =
            String::from(std::format!("Control update time: {average_update:?}\n"));
        info += &std::format!("Instance move time: {average_instances:?}\n");
        info += &std::format!("Draw time: {average_draw:?}\n");
        info += &std::format!("FPS: {fps}\n");
        info += "----------------------------------------";
        std::println!("{info}");
    }
}
