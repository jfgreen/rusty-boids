use std::{error, fmt, process};

use gl;
use glutin::{
    self, dpi, Api, ContextBuilder, ContextError, CreationError, EventsLoop, GlContext, GlProfile,
    GlRequest, GlWindow, VirtualKeyCode, WindowBuilder,
};

use crate::event::{BoidControlEvent, EventFilter};
use crate::fps::{FpsCache, FpsCounter};
use crate::glx;
use crate::render::{Renderer, RendererConfig};
use crate::system::{FlockingConfig, FlockingSystem};

const TITLE: &str = "rusty-boids";
const CACHE_FPS_MS: u64 = 500;

#[derive(Debug)]
pub enum SimulatorError {
    GlCreation(CreationError),
    GlContext(ContextError),
    Window(String),
}

impl fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimulatorError::GlCreation(ref err) => write!(f, "GL creation error, {}", err),
            SimulatorError::GlContext(ref err) => write!(f, "GL context error, {}", err),
            SimulatorError::Window(ref err) => write!(f, "Window error, {}", err),
        }
    }
}

impl error::Error for SimulatorError {
    fn description(&self) -> &str {
        match *self {
            SimulatorError::GlCreation(ref err) => err.description(),
            SimulatorError::GlContext(ref err) => err.description(),
            SimulatorError::Window(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SimulatorError::GlCreation(ref err) => Some(err),
            SimulatorError::GlContext(ref err) => Some(err),
            SimulatorError::Window(..) => None,
        }
    }
}

impl From<CreationError> for SimulatorError {
    fn from(err: CreationError) -> SimulatorError {
        SimulatorError::GlCreation(err)
    }
}

impl From<ContextError> for SimulatorError {
    fn from(err: ContextError) -> SimulatorError {
        SimulatorError::GlContext(err)
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
        //TODO: Does the update syntax work here?
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

pub fn run_simulation(config: SimulationConfig) -> Result<(), SimulatorError> {
    let mut events_loop = EventsLoop::new();
    let window = build_window(&events_loop, &config.window_size)?;
    gl_init(&window, config.debug)?;
    let window_size = get_window_size_info(&window)?;
    let flock_conf = build_flocking_config(&config, &window_size);
    let render_conf = build_render_config(&config, &window_size);
    let mut simulation = FlockingSystem::new(flock_conf);
    simulation.randomise();
    let renderer = Renderer::new(render_conf);
    renderer.init_pipeline();
    let mut fps_counter = FpsCounter::new();
    let mut fps_cacher = FpsCache::new(CACHE_FPS_MS);
    let mut running = true;
    let mut paused = false;
    let event_filter = EventFilter::new(window_size.hidpi_factor);
    while running {
        if !paused {
            simulation.update();
        }
        events_loop.poll_events(|e| match event_filter.process(e) {
            Some(BoidControlEvent::Stop) => running = false,
            Some(BoidControlEvent::Pause) => paused = !paused,
            Some(event) => handle_event(&mut simulation, event),
            _ => (),
        });
        renderer.render(&simulation.boids());
        window.swap_buffers()?;
        fps_counter.tick();
        fps_cacher.poll(&fps_counter, |new_fps| {
            let title = format!("{} - {:02} fps", TITLE, new_fps);
            window.set_title(&title);
        });
    }
    Ok(())
}

struct WindowSizeInfo {
    width: f32,
    height: f32,
    hidpi_factor: f64,
}

fn get_window_size_info(window: &GlWindow) -> Result<WindowSizeInfo, SimulatorError> {
    let hidpi_factor = window.get_hidpi_factor();
    let logical_size = window
        .get_inner_size()
        .ok_or_else(|| SimulatorError::Window("Tried to get size of closed window".to_string()))?;

    let physical_size = logical_size.to_physical(hidpi_factor);

    Ok(WindowSizeInfo{
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

fn build_window(
    events_loop: &EventsLoop,
    window_size: &WindowSize,
) -> Result<GlWindow, SimulatorError> {
    let window_builder = WindowBuilder::new().with_title(TITLE);
    let window_builder = match window_size {
        WindowSize::Fullscreen => {
            let screen = Some(events_loop.get_primary_monitor());
            window_builder.with_fullscreen(screen)
        }
        WindowSize::Dimensions((width, height)) => window_builder
            .with_dimensions(dpi::LogicalSize::new(f64::from(*width), f64::from(*height))),
    };

    let context_builder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true);

    Ok(GlWindow::new(window_builder, context_builder, events_loop)?)
}

fn gl_init(window: &GlWindow, debug: bool) -> Result<(), SimulatorError> {
    unsafe {
        window.make_current()?;
    }
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    if debug {
        print_debug_info(&window);
    }

    Ok(())
}

fn print_debug_info(window: &GlWindow) {
    println!("Vendor: {}", glx::get_gl_str(gl::VENDOR));
    println!("Renderer: {}", glx::get_gl_str(gl::RENDERER));
    println!("Version: {}", glx::get_gl_str(gl::VERSION));
    println!(
        "GLSL version: {}",
        glx::get_gl_str(gl::SHADING_LANGUAGE_VERSION)
    );
    println!("Extensions: {}", glx::get_gl_extensions().join(","));
    println!("Hidpi factor: {}", window.get_hidpi_factor());
}
