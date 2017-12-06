// Glx = Gl extras (helpers)
use std::ffi::{CStr, CString};
use std::ptr;

use gl;
use gl::types::*;
use cgmath::Matrix3;

// FIXME (maybe): Are the strings created here really static?

pub fn get_gl_extensions() -> Vec<&'static str> {
    let mut results = vec![];
    for i in 0..get_gl_int(gl::NUM_EXTENSIONS) {
        results.push(get_gl_stri(gl::EXTENSIONS, i as u32));
    }
    results
}

pub fn get_gl_int(name: GLenum) -> i32 {
    let mut i = 0;
    unsafe { gl::GetIntegerv(name, &mut i); }
    i
}

pub fn get_gl_str(name: GLenum) -> &'static str {
    unsafe { read_gl_str(gl::GetString(name)) }
}

pub fn get_gl_stri(name: GLenum, i: GLuint) -> &'static str {
    unsafe { read_gl_str(gl::GetStringi(name, i)) }
}

unsafe fn read_gl_str(ptr: *const u8) -> &'static str {
    CStr::from_ptr(ptr as *const _)
        .to_str().expect("OpenGL returned invalid utf8")
}

pub fn vtx_transform_2d(width: f32, height:f32) -> Matrix3<f32> {
    Matrix3::new(
        2./width, 0., 0.,
        0., -2./height, 0.,
        -1., 1., 1.
    )
}

pub fn clear_screen(r: GLfloat, g: GLfloat, b: GLfloat) {
    unsafe {
        gl::ClearColor(r, g, b, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

// TODO: Implement propper error type for shader

pub struct ShaderProgram{
    program_id: GLuint,
}

impl ShaderProgram {
    pub fn new(vrtx_src: &str, frag_src: &str) -> Result<ShaderProgram, String> {
        unsafe {
            let vrtx_shader = compile_shader(vrtx_src, gl::VERTEX_SHADER)?;
            let frag_shader = compile_shader(frag_src, gl::FRAGMENT_SHADER)?;
            let program_id = link_program(vrtx_shader, frag_shader)?;
            gl::DeleteShader(vrtx_shader);
            gl::DeleteShader(frag_shader);
            let program = ShaderProgram {
                program_id: program_id,
            };
            Ok(program)
        }
    }

    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }
}

unsafe fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint, String> {
    let shader = gl::CreateShader(shader_type);

    // Attempt to compile shader
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    // Get the compile status
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        // subract 1 to skip the trailing null character
        buf.set_len((len as usize) - 1);
        gl::GetShaderInfoLog(shader,
                             len,
                             ptr::null_mut(),
                             buf.as_mut_ptr() as *mut GLchar);

        Err(String::from_utf8(buf)
                   .expect("ShaderInfoLog not valid utf8"))
    } else {
        Ok(shader)
    }
}

unsafe fn link_program(vrtx_shader: GLuint, frag_shader: GLuint)-> Result<GLuint, String> {
    let program = gl::CreateProgram();

    gl::AttachShader(program, vrtx_shader);
    gl::AttachShader(program, frag_shader);

    // Attempt to link program
    gl::LinkProgram(program);

    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        // subract 1 to skip the trailing null character
        buf.set_len((len as usize) - 1);
        gl::GetProgramInfoLog(program,
                              len,
                              ptr::null_mut(),
                              buf.as_mut_ptr() as *mut GLchar);

        Err(String::from_utf8(buf)
                   .expect("ProgramInfoLog not valid utf8"))
    } else {
        Ok(program)
    }
}
