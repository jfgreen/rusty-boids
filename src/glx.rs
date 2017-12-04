// Glx = Gl extras (helpers)
use std::ffi::CStr;

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
