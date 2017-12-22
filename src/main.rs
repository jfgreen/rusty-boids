extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate rand;
extern crate clap;

mod boids;
mod glx;
mod render;
mod fps;
mod system;

use std::process;

fn main() {
    let mut boids = boids::BoidSimulator::new();
    boids.run().unwrap_or_else(|err| {
        println!("Problem running simulation, {}", err);
        process::exit(1);
    });
}
