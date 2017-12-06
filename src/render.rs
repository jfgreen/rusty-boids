use gl;
use gl::types;
use cgmath::{Matrix3, Point2};

use glx;

pub struct Renderer {
    transform: Matrix3<f32>,
}

// Shader sources
// TODO: Check this shader code is valid for 330
static VS_SRC: &'static str = "
    #version 330 core
    layout (location = 0) in vec2 position;

    uniform mat3 transform;

    void main() {
        gl_PointSize = 4.0;
        gl_Position = vec4(transform * vec3(position, 1.0), 1.0);
    }";

static FS_SRC: &'static str = "
    #version 330 core
    void main() {
        gl_FragColor = vec4(0.3, 0.1, 0.1, 1.0);
    }";

//TODO: Handle resizing of screen

impl Renderer {
    pub fn new(width: f32, height: f32) -> Renderer {
        Renderer {
            transform: glx::vtx_transform_2d(width, height),
        }
    }

    //TODO: Propigate errors properly
    pub fn init_gl_pipeline() {
        let program = glx::ShaderProgram::new(VS_SRC, FS_SRC)
            .expect("Problem creating shader program");
        program.activate();
        //TODO: Should we set glViewport()?
    }

    pub fn render(&self, points: &Vec<Point2<f32>>) {
        glx::clear_screen(0.8, 0.8, 0.8);
        // TODO: Implement full renderer
    }
}

