extern crate sdl2;
extern crate dust;

mod scene_objects;

use std::process;

use sdl2::event::{Event};
use sdl2::keyboard::Keycode;

use dust::*;

fn main() {
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();

    #[cfg(target_os = "macos")] // Use OpenGL 4.1 since that is the newest version supported on macOS
    {
        let gl_attr = video_ctx.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
    }

    let width: usize = 900;
    let height: usize = 700;
    let window = video_ctx
        .window("Dust", width as u32, height as u32)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let gl = gl::Gl::load_with(|s| video_ctx.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Screen
    let screen = screen::Screen {width, height};

    // Renderer
    let renderer = pipeline::DeferredPipeline::create(&gl, &screen, false).unwrap();
    let mirror_renderer = pipeline::DeferredPipeline::new(&gl, &screen.width/2, &screen.height/2, true).unwrap();

    // Camera
    let mut camera = camera::PerspectiveCamera::new(vec3(5.0, 5.0, 5.0), vec3(0.0, 1.0, 0.0),
                                                    vec3(0.0, 1.0, 0.0),screen.aspect(), 0.25 * ::std::f32::consts::PI, 0.1, 1000.0);

    // Objects
    let mut mesh = gust::loader::load_obj_as_dynamic_mesh("../Dust/examples/assets/models/box.obj").unwrap();
    mesh.update_vertex_normals();
    mesh.translate(&vec3(0.0, 1.0, 0.0));
    let model = ::objects::ShadedMesh::create(&gl, &mesh.to_static()).unwrap();

    let mut wireframe = ::objects::Wireframe::create(&gl, &mesh, 0.015);
    wireframe.set_parameters(0.8, 0.2, 5.0);

    let mut plane = ::objects::ShadedMesh::create(&gl, &mesh_generator::create_plane().unwrap()).unwrap();
    plane.diffuse_intensity = 0.2;
    plane.specular_intensity = 0.4;
    plane.specular_power = 20.0;

    let mut ambient_light = ::light::AmbientLight::new();
    ambient_light.base.intensity = 0.2;

    let mut light1 = dust::light::SpotLight::new(vec3(5.0, 5.0, 5.0), vec3(-1.0, -1.0, -1.0));
    light1.enable_shadows(&gl, 20.0).unwrap();
    light1.base.intensity = 0.5;

    let mut light2 = dust::light::SpotLight::new(vec3(-5.0, 5.0, 5.0), vec3(1.0, -1.0, -1.0));
    light2.enable_shadows(&gl, 20.0).unwrap();
    light2.base.intensity = 0.5;

    let mut light3 = dust::light::SpotLight::new(vec3(-5.0, 5.0, -5.0), vec3(1.0, -1.0, 1.0));
    light3.enable_shadows(&gl, 20.0).unwrap();
    light3.base.intensity = 0.5;

    let mut light4 = dust::light::SpotLight::new(vec3(5.0, 5.0, -5.0), vec3(-1.0, -1.0, 1.0));
    light4.enable_shadows(&gl, 20.0).unwrap();
    light4.base.intensity = 0.5;

    // Mirror
    let mirror_program = program::Program::from_resource(&gl, "../Dust/examples/assets/shaders/copy",
                                                                 "../Dust/examples/assets/shaders/mirror").unwrap();

    // set up event handling
    let mut events = ctx.event_pump().unwrap();

    // main loop
    let main_loop = || {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    process::exit(1);
                },
                Event::MouseMotion {xrel, yrel, mousestate, .. } => {
                    if mousestate.left()
                    {
                        eventhandler::rotate(&mut camera, xrel, yrel);
                    }
                },
                Event::MouseWheel {y, .. } => {
                    eventhandler::zoom(&mut camera, y);
                },
                _ => {}
            }
        }

        // Draw
        let render_scene = |camera: &Camera| {
            //model.render(&Mat4::identity(), camera);
            wireframe.render(camera);
        };

        // Shadow pass
        light1.shadow_cast_begin().unwrap();
        render_scene(light1.shadow_camera().unwrap());

        light2.shadow_cast_begin().unwrap();
        render_scene(light2.shadow_camera().unwrap());

        light3.shadow_cast_begin().unwrap();
        render_scene(light3.shadow_camera().unwrap());

        light4.shadow_cast_begin().unwrap();
        render_scene(light4.shadow_camera().unwrap());

        // Mirror pass
        camera.mirror_in_xz_plane();

        // Mirror pass (Geometry pass)
        mirror_renderer.geometry_pass_begin().unwrap();
        render_scene(&camera);

        // Mirror pass (Light pass)
        mirror_renderer.light_pass_begin(&camera).unwrap();
        mirror_renderer.shine_ambient_light(&ambient_light).unwrap();
        mirror_renderer.shine_spot_light(&light1).unwrap();
        mirror_renderer.shine_spot_light(&light2).unwrap();
        mirror_renderer.shine_spot_light(&light3).unwrap();
        mirror_renderer.shine_spot_light(&light4).unwrap();

        camera.mirror_in_xz_plane();

        // Geometry pass
        renderer.geometry_pass_begin().unwrap();
        render_scene(&camera);
        plane.render(&Mat4::new_scaling(100.0), &camera);

        // Light pass
        renderer.light_pass_begin(&camera).unwrap();
        renderer.shine_ambient_light(&ambient_light).unwrap();
        renderer.shine_spot_light(&light1).unwrap();
        renderer.shine_spot_light(&light2).unwrap();
        renderer.shine_spot_light(&light3).unwrap();
        renderer.shine_spot_light(&light4).unwrap();

        // Blend with the result of the mirror pass
        state::blend(&gl,state::BlendType::SRC_ALPHA__ONE_MINUS_SRC_ALPHA);
        state::depth_write(&gl,false);
        state::depth_test(&gl, state::DepthTestType::NONE);
        state::cull(&gl,state::CullType::BACK);

        mirror_renderer.light_pass_color_texture().unwrap().bind(0);
        mirror_program.add_uniform_int("colorMap", &0).unwrap();
        full_screen_quad::render(&gl, &mirror_program);

        window.gl_swap_window();
    };

    renderer::set_main_loop(main_loop);
}
