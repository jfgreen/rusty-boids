use std::error::{self, Error};
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::{fmt, process};

use gl;

use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowBuilder};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::{
    Surface, SurfaceAttributes, SurfaceAttributesBuilder, SwapInterval, WindowSurface,
};

use crate::event::{BoidControlEvent, EventFilter};
use crate::fps::{FpsCache, FpsCounter};
use crate::glx; //TODO: Rename this module
use crate::render::{Renderer, RendererConfig};
use crate::system::{FlockingConfig, FlockingSystem};

const TITLE: &str = "rusty-boids";
const CACHE_FPS_MS: u64 = 500;

#[derive(Debug)]
pub enum SimulatorError {
    Window(String),
}

impl fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimulatorError::Window(ref err) => write!(f, "Window error, {}", err),
        }
    }
}

impl error::Error for SimulatorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            SimulatorError::Window(..) => None,
        }
    }
}

impl SimulatorError {
    pub fn exit(&self) -> ! {
        println!("{}", self);
        process::exit(1);
    }
}

pub struct SimulationConfig {
    pub boid_count: u32,
    pub window_size: WindowSize,
    pub debug: bool,
    pub max_speed: f32,
    pub max_force: f32,
    pub mouse_weight: f32,
    pub sep_weight: f32,
    pub ali_weight: f32,
    pub coh_weight: f32,
    pub sep_radius: f32,
    pub ali_radius: f32,
    pub coh_radius: f32,
    pub boid_size: f32,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            boid_count: 1000,
            window_size: WindowSize::Dimensions((800, 800)),
            debug: false,
            max_speed: 2.5,
            max_force: 0.4,
            mouse_weight: 600.,
            sep_radius: 6.,
            ali_radius: 11.5,
            coh_radius: 11.5,
            sep_weight: 1.5,
            ali_weight: 1.0,
            coh_weight: 1.0,
            boid_size: 3.0,
        }
    }
}

fn build_flocking_config(
    sim_config: &SimulationConfig,
    window_size: &WindowSizeInfo,
) -> FlockingConfig {
    FlockingConfig {
        boid_count: sim_config.boid_count,
        width: window_size.width,
        height: window_size.height,
        max_speed: sim_config.max_speed,
        max_force: sim_config.max_force,
        mouse_weight: sim_config.mouse_weight,
        sep_weight: sim_config.sep_weight,
        ali_weight: sim_config.ali_weight,
        coh_weight: sim_config.coh_weight,
        sep_radius: sim_config.sep_radius,
        ali_radius: sim_config.ali_radius,
        coh_radius: sim_config.coh_radius,
    }
}

fn build_render_config(
    sim_config: &SimulationConfig,
    window_size: &WindowSizeInfo,
) -> RendererConfig {
    RendererConfig {
        width: window_size.width,
        height: window_size.height,
        boid_size: sim_config.boid_size * (window_size.hidpi_factor as f32),
        max_speed: sim_config.max_speed,
    }
}

pub enum WindowSize {
    Fullscreen,
    Dimensions((u32, u32)),
}

pub fn run_simulation(sim_config: SimulationConfig) -> Result<(), SimulatorError> {
    let mut event_loop = EventLoop::new();

    let raw_display = event_loop.raw_display_handle();

    //TODO: Handle error
    let display = unsafe { Display::new(raw_display, DisplayApiPreference::Cgl).unwrap() };
    println!("Running on: {}", display.version_string());

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(false)
        .with_multisampling(0)
        .build();

    // TODO: Handle error
    let config = unsafe { display.find_configs(template) }
        .unwrap()
        .next()
        .unwrap();

    println!("Picked a config with {} samples", config.num_samples());
    println!("config {:?}", config);

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .with_profile(GlProfile::Core)
        .build(None);

    // TODO: Handle error
    // TODO: Can we do this later?
    let non_current_gl_context = unsafe {
        display
            .create_context(&config, &context_attributes)
            .unwrap()
    };

    let window_builder = WindowBuilder::new()
        .with_title(TITLE)
        .with_transparent(false);

    let window_builder = match sim_config.window_size {
        WindowSize::Fullscreen => {
            window_builder.with_fullscreen(Some(Fullscreen::Borderless(None)))
        }
        WindowSize::Dimensions((w, h)) => window_builder.with_inner_size(LogicalSize::new(w, h)),
    };

    // TODO: Handle error
    let window = window_builder.build(&event_loop).unwrap();

    let window_size = get_window_size_info(&window)?;

    //TODO: Use above window_size instead
    let (width, height): (u32, u32) = window.inner_size().into();
    let raw_window_handle = window.raw_window_handle();
    let attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    // TODO: Handle error
    let surface = unsafe { display.create_window_surface(&config, &attributes).unwrap() };

    // TODO: Handle error
    let gl_context = non_current_gl_context.make_current(&surface).unwrap();

    //TODO: Handle error
    surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .unwrap();

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        display.get_proc_address(symbol.as_c_str()) as *const _
    });

    if sim_config.debug {
        print_debug_info();
    }

    let flock_conf = build_flocking_config(&sim_config, &window_size);
    let render_conf = build_render_config(&sim_config, &window_size);
    let mut simulation = FlockingSystem::new(flock_conf);
    simulation.randomise();
    let renderer = Renderer::new(render_conf);
    renderer.init_pipeline();
    let mut fps_counter = FpsCounter::new();
    let mut fps_cacher = FpsCache::new(CACHE_FPS_MS);
    let mut running = true;
    let mut paused = false;
    let event_filter = EventFilter::new(window_size.hidpi_factor);
    event_loop.run(move |event, event_loop_window_target, control_flow| {
        control_flow.set_wait();
        if !paused {
            simulation.update();
        }

        //TODO: Hook into close requested

        match event_filter.process(event) {
            //TODO: Wire up exit
            //Some(BoidControlEvent::Stop) => running = false,
            Some(BoidControlEvent::Pause) => paused = !paused,
            Some(event) => handle_event(&mut simulation, event),
            _ => (),
        }
        renderer.render(&simulation.boids());
        //TODO: Deal with errors
        window.request_redraw();
        surface.swap_buffers(&gl_context).unwrap();
        fps_counter.tick();
        fps_cacher.poll(&fps_counter, |new_fps| {
            let title = format!("{} - {:02} fps", TITLE, new_fps);
            window.set_title(&title);
        });
    });
    Ok(())
}

struct WindowSizeInfo {
    width: f32,
    height: f32,
    hidpi_factor: f64,
}

fn get_window_size_info(window: &Window) -> Result<WindowSizeInfo, SimulatorError> {
    let hidpi_factor = window.scale_factor();
    let physical_size = window.inner_size();

    Ok(WindowSizeInfo {
        width: physical_size.width as f32,
        height: physical_size.height as f32,
        hidpi_factor,
    })
}

fn handle_event(simulation: &mut FlockingSystem, event: BoidControlEvent) {
    match event {
        BoidControlEvent::MouseMove(x, y) => simulation.set_mouse(x, y),
        BoidControlEvent::MousePress => simulation.enable_mouse_attraction(),
        BoidControlEvent::MouseRelease => simulation.enable_mouse_repulsion(),
        BoidControlEvent::Key(VirtualKeyCode::R) => simulation.randomise(),
        BoidControlEvent::Key(VirtualKeyCode::F) => simulation.zeroise(),
        BoidControlEvent::Key(VirtualKeyCode::C) => simulation.centralise(),
        _ => (),
    }
}

fn print_debug_info() {
    println!("Vendor: {}", glx::get_gl_str(gl::VENDOR));
    println!("Renderer: {}", glx::get_gl_str(gl::RENDERER));
    println!("Version: {}", glx::get_gl_str(gl::VERSION));
    println!(
        "GLSL version: {}",
        glx::get_gl_str(gl::SHADING_LANGUAGE_VERSION)
    );
    println!("Extensions: {}", glx::get_gl_extensions().join(","));
    //TODO: Print hidpi
    //println!("Hidpi factor: {}", window.get_hidpi_factor());
}
