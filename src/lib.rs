extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate rand;
extern crate toml;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod boids;
pub mod config;
pub mod system;

mod fps;
mod glx;
mod render;
