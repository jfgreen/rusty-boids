extern crate glutin;
extern crate gl;
extern crate cgmath;

mod app;
mod glx;
mod render;

use std::process;

fn main() {
    let mut app = app::BoidsApp::new();
    app.run().unwrap_or_else(|err| {
        println!("Problem running app, {}", err);
        process::exit(1);
    });
}
