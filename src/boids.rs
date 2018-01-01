use std::fmt;
use std::error;

use gl;
use glutin;
use glutin::{
    GlRequest, Api, GlProfile,
    CreationError, ContextError,
    EventsLoop, WindowBuilder, ContextBuilder,
    GlWindow, GlContext,
    VirtualKeyCode,
};

use glx;
use render::Renderer;
use fps::CachedFpsCounter;
use system::FlockingSystem;

const TITLE: &'static str = "rusty-boids";
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
            SimulatorError::GlCreation(ref err) =>
                write!(f, "GL creation error, {}", err),
            SimulatorError::GlContext(ref err) =>
                write!(f, "GL context error, {}", err),
            SimulatorError::Window(ref err) =>
                write!(f, "Window error, {}", err),
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

pub struct SimulationConfig {
    pub boid_count: usize,
    pub window_size: WindowSize,
    pub debug: bool,
}

pub enum WindowSize {
    Fullscreen,
    Dimensions((u32, u32)),
}

//TODO: Slim down the size of this func
pub fn run_simulation(config: &SimulationConfig) -> Result<(), SimulatorError> {
    let mut events_loop = EventsLoop::new();
    let window = build_window(&events_loop, &config.window_size)?;
    gl_init(&window)?;
    if config.debug { print_debug_info(); }
    let size = get_window_size(&window)?;
    let mut simulation = FlockingSystem::new(size);
    simulation.add_boids(config.boid_count);
    let renderer = Renderer::new(size);
    renderer.init_pipeline();
    let mut fps_counter = CachedFpsCounter::new(CACHE_FPS_MS);
    let mut running = true;
    while running {
        simulation.update();
        events_loop.poll_events(|e| match process_event(e) {
            Some(ControlEvent::Stop)   => running = false,
            Some(ControlEvent::Key(k)) => handle_key(&mut simulation, k),
            _ => ()
        });
        renderer.render(&simulation.positions());
        window.swap_buffers()?;
        fps_counter.tick();
        // TODO: Consider pulling caching out of fps counter
        // Could we use a functional/closure thing instead
        fps_counter.poll_change(|fps| {
            let title = format!("{} - {} fps", TITLE, fps);
            window.set_title(&title);
        });
    }
    Ok(())
}

fn handle_key(simulation: &mut FlockingSystem, key: VirtualKeyCode) {
    match key {
        VirtualKeyCode::R => simulation.randomise(),
        VirtualKeyCode::F => simulation.zeroise(),
        VirtualKeyCode::C => simulation.centralise(),
        _ => ()
    }
}

enum ControlEvent {
    Stop,
    Key(VirtualKeyCode),
}

fn process_event(event: glutin::Event) -> Option<ControlEvent> {
    match event {
        glutin::Event::WindowEvent {
            event: e, ..
        } => process_window_event(e),
        _ => None
    }
}

fn process_window_event(event: glutin::WindowEvent) -> Option<ControlEvent> {
    use glutin::{WindowEvent, KeyboardInput, ElementState};
    match event {
        WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(k), ..
            }, ..
        } => process_keypress(k),

        //TODO: React to mouse
        //WindowEvent::MouseMoved {
        //    position: (x, y), ..
        //} => ...,

        WindowEvent::Closed => Some(ControlEvent::Stop),
        _ => None
    }
}

fn process_keypress(key: VirtualKeyCode) -> Option<ControlEvent> {
    match key {
        VirtualKeyCode::Escape | VirtualKeyCode::Q => Some(ControlEvent::Stop),
        _ => Some(ControlEvent::Key(key)),
    }
}

fn build_window(events_loop: &EventsLoop, window_size: &WindowSize)
    -> Result<GlWindow, SimulatorError> {

    let window_builder = WindowBuilder::new().with_title(TITLE);
    let window_builder = match window_size {
        &WindowSize::Fullscreen => {
            let screen = Some(events_loop.get_primary_monitor());
            window_builder.with_fullscreen(screen)
        },
        &WindowSize::Dimensions((width, height)) => {
            window_builder.with_dimensions(width, height)
        }
    };

    let context_builder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true);

    Ok(GlWindow::new(
        window_builder,
        context_builder,
        events_loop
    )?)
}

fn gl_init(window: &GlWindow) -> Result<(), SimulatorError> {
        unsafe { window.make_current()?; }
        gl::load_with(|symbol| {
            window.get_proc_address(symbol) as *const _
        });
        Ok(())
}

fn get_window_size(window: &GlWindow) -> Result<(f32, f32), SimulatorError> {
    window.get_inner_size_points()
        .map(|(w, h)| (w as f32, h as f32))
        .ok_or(SimulatorError::Window(
                "Tried to get size of closed window".to_string()))
}

fn print_debug_info() {
    println!("Vendor: {}", glx::get_gl_str(gl::VENDOR));
    println!("Renderer: {}", glx::get_gl_str(gl::RENDERER));
    println!("Version: {}", glx::get_gl_str(gl::VERSION));
    println!("GLSL version: {}", glx::get_gl_str(gl::SHADING_LANGUAGE_VERSION));
    println!("Extensions: {}", glx::get_gl_extensions().join(","));
}

