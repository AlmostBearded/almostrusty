use std::ffi::CStr;
use std::path::Path;

use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder, GlProfile, GlRequest,
};
use log::Level;

use utils::log::Logger;

use crate::assets::shader_asset::load_shader_assets;
use crate::assets::shader_program_asset::load_shader_program_assets;
use crate::assets::window_config::load_window_config_from_path;
use crate::graphics::gl::shader::shader::compile_shaders;
use crate::graphics::gl::shader::shader_program::link_shader_programs;

pub mod assets;
pub mod graphics;
pub mod utils;

fn main() {
    Logger::init_with_level(Level::Debug);
    log::info!("Game started");

    let shader_meta_paths = vec![
        Path::new("assets/shaders/triangle.frag.meta"),
        Path::new("assets/shaders/triangle.vert.meta"),
    ];
    log::debug!("Shader meta paths: {:?}", shader_meta_paths);

    let (shader_id_lookup_map, shader_assets) = load_shader_assets(&shader_meta_paths);
    log::debug!("Shader id lookup map: {:?}", shader_id_lookup_map);
    log::debug!("Shader assets: {:?}", shader_assets);

    let shader_program_meta_paths = vec![Path::new("assets/shaders/triangle.shader.meta")];
    log::debug!("Shader program meta paths: {:?}", shader_program_meta_paths);

    let (shader_program_id_lookup_map, shader_program_assets) =
        load_shader_program_assets(&shader_program_meta_paths);
    log::debug!(
        "Shader program id lookup map: {:?}",
        shader_program_id_lookup_map
    );
    log::debug!("Shader program assets: {:?}", shader_program_assets);

    let window_config = load_window_config_from_path(Path::new("assets/window.ron"));

    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title(window_config.title);

    let windowed_context = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build_windowed(wb, &el)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let gl_context = windowed_context.context();

    log::info!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);
    let version = unsafe {
        String::from_utf8(
            CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
                .to_bytes()
                .to_vec(),
        )
        .unwrap()
    };
    log::info!("OpenGL version: {}", version);

    unsafe {
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    let shaders = compile_shaders(&shader_assets);
    log::debug!("Compiled shaders: {:?}", shaders);

    let shader_programs =
        link_shader_programs(&shader_program_assets, &shader_id_lookup_map, &shaders);
    log::debug!("Linked shader programs: {:?}", shader_programs);

    // program.activate();

    el.run(move |event, _, control_flow| {
        log::trace!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
