use std::ptr;
use std::mem;

use gl;
use gl::types::*;
use cgmath::{Matrix, Matrix3, Point2};

use glx::{self, ShaderProgram, VertexArray, Buffer};

// Shader sources
static VS_SRC: &'static str = "
    #version 330 core
    layout (location = 0) in vec2 position;

    uniform mat3 transform;

    void main() {
        gl_PointSize = 3.0;
        gl_Position = vec4(transform * vec3(position, 1.0), 1.0);
    }";

static FS_SRC: &'static str = "
    #version 330 core
    out vec4 frag_colour;

    void main() {
        frag_colour = vec4(0.7, 0.7, 0.7, 1.0);
    }";

//TODO: Handle resizing of screen
//TODO: Should we be explicity setting glViewport()?
//TODO: Use hidpi_factor to scale gl_PointSize
//TODO: How to run at different resolutions

pub struct Renderer {
    transform: Matrix3<f32>,
    program: ShaderProgram,
    vao: VertexArray,
    vbo: Buffer,
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Renderer {
        let program = ShaderProgram::new(VS_SRC, FS_SRC)
            .expect("Problem creating shader program");

        Renderer {
            transform: glx::vtx_transform_2d(width, height),
            program,
            vao: VertexArray::new(),
            vbo: Buffer::new(),
        }
    }

    pub fn init_pipeline(&self) {

        unsafe {
            self.vao.bind();
            self.vbo.bind(gl::ARRAY_BUFFER);
            self.program.activate();

            // Set the tranform uniform
            let trans_loc = self.program.get_uniform_location("transform")
                .expect("Could not find uniform");
            gl::UniformMatrix3fv(trans_loc, 1, gl::FALSE, self.transform.as_ptr());

            // Specify the layout of the vertex data
            let pos_loc = self.program.get_atrib_location("position")
                .expect("could not find position");
            gl::EnableVertexAttribArray(pos_loc);
            gl::VertexAttribPointer(pos_loc,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    0,
                                    ptr::null());

            // Allow shader to specify point size
            gl::Enable(gl::PROGRAM_POINT_SIZE);
        }

    }

    pub fn render(&self, points: &[Point2<f32>]) {
        glx::clear_screen(0.1, 0.1, 0.1);
        unsafe {
            // This _should_ implement buffer orphaning
            gl::BufferData(gl::ARRAY_BUFFER, 0, ptr::null(), gl::STREAM_DRAW);

            gl::BufferData(gl::ARRAY_BUFFER,
                           (points.len() * mem::size_of::<Point2<f32>>()) as GLsizeiptr,
                           points.as_ptr() as *const _,
                           gl::STREAM_DRAW);

            gl::DrawArrays(gl::POINTS, 0, points.len() as i32);
        }
    }

}

