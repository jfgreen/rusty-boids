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

use boids::{run_simulation, SimulationConfig};
use std::process;
use clap::{App, Arg};

fn main() {
    let config = build_config();
    run_simulation(&config).unwrap_or_else(|err| {
        println!("Problem running simulation, {}", err);
        process::exit(1);
    });
}

// Argument parsing
//TODO: Would be cool if there was an arg to print / generate an example config file
//TODO: Remember to push validation into clap e.g height/width are positive integers
//TODO: Where possible, set defaults upfront in clap
//TODO: Parse config in this order: cli-args > config > default

fn build_config() -> SimulationConfig {
    let matches = App::new("boid-simulator")
        .version("0.1")
        .author("James Green")
        .about("Simulates flocking behaviour of birds")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets the config file to read simulation parameters from")
             .takes_value(true))
        .arg(Arg::with_name("fullscreen")
             .short("f")
             .long("fullscreen")
             .help("Sets the simulation to display fullscreen on primary monitor"))
        .get_matches();

    SimulationConfig {
        fullscreen: matches.is_present("fullscreen"),
    }
}
