extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate rand;

#[macro_use]
extern crate clap;

mod boids;
mod glx;
mod render;
mod fps;
mod system;

use boids::{
    run_simulation,
    SimulationConfig,
    WindowSize
};
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
             .help("Sets the config file to read simulation parameters from"))
        .arg(Arg::with_name("size")
             .short("s")
             .long("size")
             .value_names(&["width", "height"])
             .use_delimiter(true)
             .default_value("800,800")
             .help("Sets the simultion window to specified width & height"))
        .arg(Arg::with_name("fullscreen")
             .short("f")
             .long("fullscreen")
             .help("Display fullscreen (overrides size argument)")
             .conflicts_with("size"))
        .arg(Arg::with_name("boids")
             .short("b")
             .long("boid-count")
             .default_value("1000")
             .help("Sets the number of boids to simulate"))
        .arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .help("print opengl debug information"))
        .get_matches();

    let window_size = if matches.is_present("fullscreen") {
            WindowSize::Fullscreen
        } else {
            let size = values_t!(matches, "size", u32)
                .unwrap_or_else(|e| e.exit());
            WindowSize::Dimensions((size[0], size[1]))
        };

    let boid_count = value_t!(matches, "boids", usize)
            .unwrap_or_else(|e| e.exit());

    let debug = matches.is_present("debug");

    SimulationConfig { window_size, boid_count, debug }
}
