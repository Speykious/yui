use std::env;
use std::error::Error;
use std::ffi::CString;
use std::num::NonZeroU32;

use glam::{uvec2, vec2, Vec2};
use glow::HasContext;

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::Display,
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};

use glutin_winit::ApiPrefence;
use raw_window_handle::HasRawWindowHandle;

use tracing::{debug, error, info, warn};

use winit::{
    event::{ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer::Renderer;

#[inline]
pub fn app(
    window_builder: WindowBuilder,
    config_template_builder: ConfigTemplateBuilder,
) -> Result<App, Box<dyn Error>> {
    App::new(window_builder, config_template_builder)
}

pub struct App {
    pub gl_ctx: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    pub gl_display: Display,
    pub window: Window,
    pub events: Option<EventLoop<()>>,
    pub renderer: Renderer,
}

impl App {
    pub fn new(
        window_builder: WindowBuilder,
        config_template_builder: ConfigTemplateBuilder,
    ) -> Result<App, Box<dyn Error>> {
        if cfg!(target_os = "linux") {
            // disables vsync sometimes on x11
            if env::var("vblank_mode").is_err() {
                env::set_var("vblank_mode", "0");
            }
        }

        let events = winit::event_loop::EventLoop::new();

        let (window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(ApiPrefence::FallbackEgl)
            .with_window_builder(Some(window_builder))
            .build(&events, config_template_builder, |configs| {
                configs
                    .filter(|c| c.srgb_capable())
                    .max_by_key(|c| c.num_samples())
                    .unwrap()
            })?;

        let window = window.unwrap(); // set in display builder
        let raw_window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 1))))
            .with_profile(glutin::context::GlProfile::Core)
            .build(Some(raw_window_handle));

        let dimensions = window.inner_size();

        let (gl_surface, gl_ctx) = {
            let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(dimensions.width).unwrap(),
                NonZeroU32::new(dimensions.height).unwrap(),
            );

            let surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs)? };
            let context = unsafe { gl_display.create_context(&gl_config, &context_attributes)? }
                .make_current(&surface)?;
            (surface, context)
        };

        // Load the OpenGL function pointers
        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                gl_display.get_proc_address(&CString::new(symbol).unwrap()) as *const _
            })
        };

        // MacOS doesn't support debug output. Rip. :(
        #[cfg(not(target_os = "macos"))]
        unsafe {
            gl.debug_message_callback(|_src, ty, _id, sevr, msg| {
                let ty = match ty {
                    glow::DEBUG_TYPE_ERROR => "Error: ",
                    glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior: ",
                    glow::DEBUG_TYPE_MARKER => "Marker: ",
                    glow::DEBUG_TYPE_OTHER => "",
                    glow::DEBUG_TYPE_POP_GROUP => "Pop Group: ",
                    glow::DEBUG_TYPE_PORTABILITY => "Portability: ",
                    glow::DEBUG_TYPE_PUSH_GROUP => "Push Group: ",
                    glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior: ",
                    glow::DEBUG_TYPE_PERFORMANCE => "Performance: ",
                    ty => unreachable!("unknown debug type {ty}"),
                };
                match sevr {
                    glow::DEBUG_SEVERITY_NOTIFICATION => debug!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_LOW => info!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_MEDIUM => warn!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_HIGH => error!(target: "opengl", "{ty}{msg}"),
                    sevr => unreachable!("unknown debug severity {sevr}"),
                };
            });

            gl.enable(glow::DEBUG_OUTPUT);
        }

        // initialize renderer
        info!("Initializing Yui renderer");
        let window_size = window.inner_size();
        let viewport = uvec2(window_size.width, window_size.height);
        let renderer = Renderer::new(gl, viewport)?;
        info!("Yui renderer initialized");

        Ok(App {
            gl_ctx,
            gl_surface,
            gl_display,
            window,
            events: Some(events),
            renderer,
        })
    }

    pub fn run<Se, Dr, T: 'static>(mut self, setup: Se, draw: Dr) -> !
    where
        Se: Fn(&mut Self) -> T + 'static,
        Dr: Fn(&mut Self, &T) -> () + 'static,
    {
        let data = (setup)(&mut self);

        let mut camera_pos = Vec2::ZERO;
        let mut mouse_pos = Vec2::ZERO;
        let mut mouse_pos_held = mouse_pos;
        let mut mouse_state = ElementState::Released;

        let events = self.events.take().unwrap();
        events.run(move |event, _, control_flow| {
            // They need to be present
            let _gl_display = &self.gl_display;
            let _window = &self.window;

            control_flow.set_wait();

            match event {
                Event::RedrawRequested(_) => {
                    debug!("Redrawing");

                    self.renderer.clear();

                    // TODO: draw something here
                    (draw)(&mut self, &data);

                    self.gl_surface.swap_buffers(&self.gl_ctx).unwrap();
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        debug!(
                            "Window resized to ({}, {})",
                            physical_size.width, physical_size.height
                        );

                        // Handle window resizing
                        self.renderer
                            .resize(physical_size.width, physical_size.height);
                        self.gl_surface.resize(
                            &self.gl_ctx,
                            NonZeroU32::new(physical_size.width).unwrap(),
                            NonZeroU32::new(physical_size.height).unwrap(),
                        );
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = vec2(position.x as f32, position.y as f32);
                        if mouse_state == ElementState::Pressed {
                            self.renderer.camera.position = camera_pos
                                + (mouse_pos - mouse_pos_held) / self.renderer.camera.scale;

                            let cpos = self.renderer.camera.position;
                            debug!("Scene moved to ({}, {})", cpos.x, cpos.y);

                            self.window.request_redraw();
                        }
                    }
                    WindowEvent::MouseInput { state, .. } => {
                        debug!("Mouse got {:?}", state);

                        mouse_state = *state;
                        if mouse_state == ElementState::Pressed {
                            mouse_pos_held = mouse_pos;
                            camera_pos = self.renderer.camera.position;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        // Handle mouse wheel (zoom)
                        let my = match delta {
                            MouseScrollDelta::LineDelta(_, y) => *y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };

                        if my.is_sign_positive() {
                            self.renderer.camera.scale *= 8.0 * my.abs() / 7.0;
                            debug!("Zooming to {}", self.renderer.camera.scale);
                        } else {
                            self.renderer.camera.scale *= 7.0 * my.abs() / 8.0;
                            debug!("Dezooming to {}", self.renderer.camera.scale);
                        }

                        self.window.request_redraw();
                    }
                    _ => (),
                },
                _ => (),
            }

            Self::handle_close(event, control_flow);
        })
    }

    fn handle_close(event: Event<()>, control_flow: &mut ControlFlow) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    info!("There is an Escape D:");
                    control_flow.set_exit();
                }
                _ => (),
            }
        }
    }
}
