extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate rand;
extern crate toml;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;


mod boids;
mod glx;
mod render;
mod fps;
mod system;
mod config;

use config::build_config;
use boids::run_simulation;

use std::process;


fn main() {
    let config = build_config().unwrap_or_else(|err| {
        err.exit()
    });
    run_simulation(&config).unwrap_or_else(|err| {
        //TODO: Have an err.exit()
        println!("Problem running simulation, {}", err);
        process::exit(1);
    });
}


