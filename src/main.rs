use std::error::Error;

use app::App;
use clap::Parser;
use glam::{vec2, Vec2, vec4, Vec4};
use glow::HasContext;
use glutin::config::ConfigTemplateBuilder;
use render::gl_buffer::GlBuffer;
use render::{shader, BlendMode, OpenglRendererError};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};
use winit::window::WindowBuilder;

mod app;
mod camera;
mod render;

const RECT_VERT: &str = include_str!("../shaders/rect.vert");
const RECT_FRAG: &str = include_str!("../shaders/round-rect.frag");

#[allow(unused)]
struct Data {
    vao: glow::NativeVertexArray,
    positions: GlBuffer<Vec4>,
    uvs: GlBuffer<Vec2>,
    indices: GlBuffer<u16>,
    rect_program: glow::NativeProgram,
    u_mvp: Option<glow::NativeUniformLocation>,
    u_dimensions: Option<glow::NativeUniformLocation>,
    u_radius: Option<glow::NativeUniformLocation>,
    u_zoom: Option<glow::NativeUniformLocation>,
    // textures: Vec<Texture>,
}

/// Executed once at the start of the application
fn setup(yui_app: &mut App) -> Data {
    #[rustfmt::skip]
    let positions = GlBuffer::from(vec![
        vec4(-200., -100., 0., 1.),
        vec4( 200., -100., 0., 1.),
        vec4( 200.,  100., 0., 1.),
        vec4(-200.,  100., 0., 1.),
    ]);

    #[rustfmt::skip]
    let uvs = GlBuffer::from(vec![
        vec2(0., 0.),
        vec2(1., 0.),
        vec2(1., 1.),
        vec2(0., 1.),
    ]);

    #[rustfmt::skip]
    let indices = GlBuffer::from(vec![
        0, 1, 2,
        2, 3, 0,
    ]);

    let gl = &yui_app.renderer.gl;

    // Initialize buffers
    let vao;
    unsafe {
        vao = gl
            .create_vertex_array()
            .map_err(OpenglRendererError::Opengl)
            .unwrap();
        gl.bind_vertex_array(Some(vao));

        positions.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        uvs.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        indices.upload(&gl, glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW);
    }

    // Shaders
    let rect_program = shader::compile(gl, RECT_VERT, RECT_FRAG).unwrap();
    yui_app.renderer.bind_shader(&&rect_program);
    let u_mvp = unsafe { gl.get_uniform_location(rect_program, "u_mvp") };
    let u_dimensions = unsafe { gl.get_uniform_location(rect_program, "u_dimensions") };
    let u_radius = unsafe { gl.get_uniform_location(rect_program, "u_radius") };
    let u_zoom = unsafe { gl.get_uniform_location(rect_program, "u_zoom") };

    unsafe {
        gl.enable(glow::BLEND);
        gl.uniform_1_f32(u_radius.as_ref(), 50.);
        gl.uniform_2_f32(u_dimensions.as_ref(), 400., 200.);
    }

    Data {
        vao,
        positions,
        uvs,
        indices,
        rect_program,
        u_mvp,
        u_dimensions,
        u_radius,
        u_zoom,
    }
}

/// Executed on every update
fn draw(yui_app: &mut App, data: &Data) {
    let gl = &yui_app.renderer.gl;

    let count = data.indices.len();
    let renderer = &yui_app.renderer;
    let camera = &renderer.camera;
    let mvp = camera.matrix(renderer.viewport.as_vec2());
    renderer.set_blend_mode(BlendMode::Normal);
    unsafe {
        gl.uniform_1_f32(data.u_zoom.as_ref(), camera.scale.x);
        gl.uniform_matrix_4_f32_slice(data.u_mvp.as_ref(), false, mvp.as_ref());
        gl.draw_elements(glow::TRIANGLES, count as i32, glow::UNSIGNED_SHORT, 0);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli;

fn main() -> Result<(), Box<dyn Error>> {
    miette::set_panic_hook();
    let _cli = Cli::parse();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(LevelFilter::DEBUG)
        .init();

    info!("Setting up windowing and OpenGL");
    let window_builder = WindowBuilder::new()
        .with_transparent(false)
        .with_resizable(true)
        .with_inner_size(winit::dpi::PhysicalSize::new(600, 400))
        .with_title("Yui app");

    let config_template_builder = ConfigTemplateBuilder::new().with_multisampling(4);

    let yui_app = app::app(window_builder, config_template_builder)?;
    yui_app.run(setup, draw)
}
