#![allow(unused)]

pub mod gl_buffer;
pub mod shader;
pub mod texture;

use std::cell::RefCell;
use std::ops::Deref;

use glam::{uvec2, UVec2};
use glow::HasContext;

use crate::camera::Camera;
use shader::ShaderCompileError;

/// Blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal blending mode.
    Normal,
}

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize OpenGL renderer: {0}")]
pub enum OpenglRendererError {
    ShaderCompile(#[from] ShaderCompileError),
    Opengl(String),
}

#[derive(Default, Clone)]
pub struct GlCache {
    pub camera: Option<Camera>,
    pub blend_mode: Option<BlendMode>,
    pub program: Option<glow::NativeProgram>,
    pub albedo: Option<usize>,
}

impl GlCache {
    pub fn update_camera(&mut self, camera: &Camera) -> bool {
        if let Some(prev_camera) = &mut self.camera {
            let mut changed = false;

            if prev_camera.position != camera.position {
                prev_camera.position = camera.position;
                changed = true;
            }
            if prev_camera.rotation != camera.rotation {
                prev_camera.rotation = camera.rotation;
                changed = true;
            }
            if prev_camera.scale != camera.scale {
                prev_camera.scale = camera.scale;
                changed = true;
            }

            changed
        } else {
            self.camera = Some(camera.clone());
            true
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) -> bool {
        if let Some(prev_mode) = self.blend_mode.replace(blend_mode) {
            prev_mode != blend_mode
        } else {
            true
        }
    }

    pub fn update_program(&mut self, program: glow::NativeProgram) -> bool {
        if let Some(prev_program) = self.program.replace(program) {
            prev_program != program
        } else {
            true
        }
    }

    pub fn update_albedo(&mut self, albedo: usize) -> bool {
        if let Some(prev_texture) = self.albedo.replace(albedo) {
            prev_texture != albedo
        } else {
            true
        }
    }
}

pub struct Renderer {
    pub gl: glow::Context,
    pub camera: Camera,
    pub viewport: UVec2,
    cache: RefCell<GlCache>,
}

impl Renderer {
    pub fn new(gl: glow::Context, viewport: UVec2) -> Result<Self, OpenglRendererError> {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.viewport(0, 0, viewport.x as i32, viewport.y as i32);
        }

        Ok(Self {
            gl,
            camera: Camera::default(),
            viewport,
            cache: RefCell::new(GlCache::default()),
        })
    }

    pub fn resize(&mut self, x: u32, y: u32) {
        self.viewport = uvec2(x, y);
        unsafe { self.gl.viewport(0, 0, x as i32, y as i32) };
    }

    pub fn clear(&self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.1, 0.2, 0.3, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    #[inline]
    pub fn bind_shader<S: Deref<Target = glow::NativeProgram>>(&self, shader: &S) {
        let program = **shader;
        unsafe { self.gl.use_program(Some(program)) };
    }

    /// Pushes an OpenGL debug group.
    /// This is very useful to debug OpenGL calls per node with `apitrace`, as it will nest calls inside of labels,
    /// making it trivial to know which calls correspond to which nodes.
    ///
    /// It is a no-op on platforms that don't support it (only MacOS so far).
    #[inline]
    pub fn push_debug_group(&self, name: &str) {
        #[cfg(not(target_os = "macos"))]
        unsafe {
            self.gl
                .push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, name);
        }
    }

    /// Pops the last OpenGL debug group.
    ///
    /// It is a no-op on platforms that don't support it (only MacOS so far).
    #[inline]
    pub fn pop_debug_group(&self) {
        #[cfg(not(target_os = "macos"))]
        unsafe {
            self.gl.pop_debug_group();
        }
    }

    /// Updates the camera in the GL cache and returns whether it changed.
    pub fn update_camera(&self) -> bool {
        if !self.cache.borrow_mut().update_camera(&self.camera) {
            return false;
        }

        let matrix = self.camera.matrix(self.viewport.as_vec2());

        true
    }

    /// Set blending mode. See `BlendMode` for supported blend modes.
    pub fn set_blend_mode(&self, blend_mode: BlendMode) {
        if !self.cache.borrow_mut().update_blend_mode(blend_mode) {
            return;
        }

        let gl = &self.gl;
        unsafe {
            match blend_mode {
                BlendMode::Normal => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                }
            }
        }
    }
}
