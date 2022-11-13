extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate rand;
extern crate raw_window_handle;
extern crate toml;
extern crate winit;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod boids;
pub mod config;
pub mod system;

mod event;
mod fps;
mod glx;
mod render;
