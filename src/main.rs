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

mod boids;
mod config;
mod fps;
mod glx;
mod render;
mod system;

use crate::boids::run_simulation;
use crate::config::build_config;

fn main() {
    let config = build_config().unwrap_or_else(|err| {
        println!("{}", "Failure building configuration:");
        err.exit()
    });

    run_simulation(config).unwrap_or_else(|err| {
        println!("{}", "Failure running simulation");
        err.exit()
    });
}
