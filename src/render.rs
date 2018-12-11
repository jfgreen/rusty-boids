use std::{mem, ptr};

use cgmath::{Matrix, Matrix3, Point2};
use gl::{self, types::*};
use crate::system::Boid;

use crate::glx::{self, Buffer, ShaderProgram, VertexArray};

// Shader sources
static VS_SRC: &'static str = "
    #version 330 core
    layout (location = 0) in vec2 position;
    layout (location = 1) in vec2 velocity;

    uniform mat3 transform;
    uniform float pointSize;
    uniform float maxSpeedSquared;

    out vec4 pointColor;

    float two_pi = 6.2831853072;

    vec3 rgb_from_hsb(in vec3 c){
        vec3 rgb = clamp(abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),
                                 6.0)-3.0)-1.0,
                         0.0,
                         1.0 );
        rgb = rgb*rgb*(3.0-2.0*rgb);
        return c.z * mix(vec3(1.0), rgb, c.y);
    }

    float mag_2 = pow(velocity.x, 2) + pow(velocity.y, 2);

    float a = atan(velocity.y, velocity.x);
    void main() {
        pointColor = vec4(rgb_from_hsb(vec3(a/two_pi, 1 - (mag_2 / maxSpeedSquared), 1.0)), 1.0);
        gl_PointSize = pointSize;
        gl_Position = vec4(transform * vec3(position, 1.0), 1.0);
    }";

static FS_SRC: &'static str = "
    #version 330 core
    out vec4 frag_colour;

    in vec4 pointColor;

    void main() {
        frag_colour = pointColor;
    }";

//TODO: Handle resizing of screen
//TODO: Use hidpi_factor to scale gl_PointSize
//TODO: How to run at different resolutions

pub struct RendererConfig {
    pub width: f32,
    pub height: f32,
    pub boid_size: f32,
    pub max_speed: f32,
}

pub struct Renderer {
    transform: Matrix3<f32>,
    boid_size: f32,
    max_speed: f32,
    program: ShaderProgram,
    vao: VertexArray,
    vbo: Buffer,
}

impl Renderer {
    pub fn new(config: RendererConfig) -> Renderer {
        let program = ShaderProgram::new(VS_SRC, FS_SRC).expect("Problem creating shader program");

        Renderer {
            transform: glx::vtx_transform_2d(config.width, config.height),
            boid_size: config.boid_size,
            max_speed: config.max_speed,
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
            let trans_loc = self
                .program
                .get_uniform_location("transform")
                .expect("Could not find uniform");
            gl::UniformMatrix3fv(trans_loc, 1, gl::FALSE, self.transform.as_ptr());

            // Set the point size
            let size_loc = self
                .program
                .get_uniform_location("pointSize")
                .expect("Could not find uniform");
            gl::Uniform1f(size_loc, self.boid_size as GLfloat);

            // Set max speed
            let max_speed_loc = self
                .program
                .get_uniform_location("maxSpeedSquared")
                .expect("Could not find uniform");
            gl::Uniform1f(max_speed_loc, self.max_speed.powi(2) as GLfloat);

            // Specify the layout of the vertex data
            let pos_loc = self
                .program
                .get_atrib_location("position")
                .expect("could not find position");
            gl::EnableVertexAttribArray(pos_loc);
            gl::VertexAttribPointer(
                pos_loc,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Boid>() as GLsizei,
                ptr::null(),
            );

            let vel_loc = self
                .program
                .get_atrib_location("velocity")
                .expect("could not find velocity");
            gl::EnableVertexAttribArray(vel_loc);
            gl::VertexAttribPointer(
                vel_loc,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Boid>() as GLsizei,
                mem::size_of::<Point2<f32>>() as *const GLvoid,
            );

            // Allow shader to specify point size
            gl::Enable(gl::PROGRAM_POINT_SIZE);
        }
    }

    pub fn render(&self, boids: &[Boid]) {
        glx::clear_screen(0.1, 0.1, 0.1);
        unsafe {
            // This _should_ implement buffer orphaning
            gl::BufferData(gl::ARRAY_BUFFER, 0, ptr::null(), gl::STREAM_DRAW);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (boids.len() * mem::size_of::<Boid>()) as GLsizeiptr,
                boids.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::DrawArrays(gl::POINTS, 0, boids.len() as i32);
        }
    }
}
