use gl;
use gl::types;
use cgmath::Matrix3;

use glx;

pub struct Renderer {
    transform: Matrix3<f32>,
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Renderer {
        Renderer{
            transform: glx::vtx_transform_2d(width, height),
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(0.8, 0.8, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // TODO: Implement full renderer
    }
}

