use std::fmt;
use std::error;
use std::time::{Duration, Instant};

use gl;
use glutin;
use glutin::{
    GlRequest, Api, GlProfile,
    CreationError, ContextError,
    EventsLoop, WindowBuilder, ContextBuilder,
    GlWindow, GlContext
};
use cgmath::Point2;

use glx;
use render::Renderer;
use fps::FpsCounter;
use boids::Simulation;

const TITLE: &'static str = "rusty-boids";
const UPDATE_FPS_MS: u64 = 500;

#[derive(Debug)]
pub enum AppError {
    GlCreation(CreationError),
    GlContext(ContextError),
    Window(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::GlCreation(ref err) =>
                write!(f, "GL creation error, {}", err),
            AppError::GlContext(ref err) =>
                write!(f, "GL context error, {}", err),
            AppError::Window(ref err) =>
                write!(f, "Window error, {}", err),
        }
    }
}

impl error::Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::GlCreation(ref err) => err.description(),
            AppError::GlContext(ref err) => err.description(),
            AppError::Window(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AppError::GlCreation(ref err) => Some(err),
            AppError::GlContext(ref err) => Some(err),
            AppError::Window(..) => None,
        }
    }
}

impl From<CreationError> for AppError {
    fn from(err: CreationError) -> AppError {
        AppError::GlCreation(err)
    }
}

impl From<ContextError> for AppError {
    fn from(err: ContextError) -> AppError {
        AppError::GlContext(err)
    }
}

pub struct BoidsApp {
    running: bool,
    mouse_pos: Point2<f32>,
    last_updated_fps: Instant,
    last_shown_fps: u32,
    simulation: Simulation,
}

//TODO: Add key for randomisation
//


impl BoidsApp {
    pub fn new() -> Self {
        BoidsApp {
            running: false,
            mouse_pos: Point2::new(0.,0.),
            last_shown_fps: 0,
            last_updated_fps: Instant::now(),
            //TODO: Can we do better than zero sized as initial?
            //Maybe get the relevant part of window construction up here ?
            simulation: Simulation::new((0., 0.)),
        }
    }

    //FIXME: Seems like vsync stops applying when window off screen
    // ... so don't rely on it to limit fps
    pub fn run(&mut self) -> Result<(), AppError>{
        let mut events_loop = EventsLoop::new();
        let window = AppWindow::new(&events_loop)?;
        window.activate()?;
        let size = window.get_size()?;
        let renderer = Renderer::new(size);
        self.simulation.resize(size);
        self.simulation.add_boids(2000); //TODO: Parameterise / cli arg
        renderer.init_gl_pipeline();
        let mut fps_counter = FpsCounter::new();
        self.running = true;
        while self.running {
            self.simulation.update();
            events_loop.poll_events(|e| self.handle_event(e));
            renderer.render(&self.simulation.positions());
            window.swap_buffers()?;
            fps_counter.tick();
            self.update_fps(&window, fps_counter.current());
        }
        Ok(())
    }

    fn handle_event(&mut self, event: glutin::Event) {
        match event {
            glutin::Event::WindowEvent {
                event: e, ..
            } => self.handle_window_event(e),
            _ => ()
        }
    }

    fn handle_window_event(&mut self, event: glutin::WindowEvent) {
        use glutin::{WindowEvent, KeyboardInput, ElementState};
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(k), ..
                }, ..
            } => self.handle_keypress(k),

            WindowEvent::MouseMoved {
                position: (x, y), ..
            } => self.handle_mouse_move(x as f32, y as f32),

            WindowEvent::Closed => self.stop(),

            _ => ()
        }
    }

    fn handle_keypress(&mut self, key: glutin::VirtualKeyCode) {
        use glutin::VirtualKeyCode;
        match key {
            VirtualKeyCode::Escape | VirtualKeyCode::Q
                => self.stop(),
            _ => ()
        }
    }

    fn handle_mouse_move(&mut self, x: f32, y:f32) {
        self.mouse_pos.x = x as f32;
        self.mouse_pos.y = y as f32;
    }

    fn update_fps(&mut self, window: &AppWindow, fps: u32) {
        if self.last_updated_fps.elapsed() > Duration::from_millis(UPDATE_FPS_MS) {
            self.last_updated_fps = Instant::now();
            if fps != self.last_shown_fps {
                window.display_fps(fps);
                self.last_shown_fps = fps;
            }
        }
    }

    fn stop(&mut self) {
        self.running = false
    }

}


struct AppWindow {
    window: GlWindow,
}

impl AppWindow {
    fn new(events_loop: &EventsLoop) -> Result<AppWindow, AppError> {
        //TODO: Pass in size & fullscreen settings via CLI
        //let monitor = events_loop.get_primary_monitor();
        let window_builder = WindowBuilder::new()
            .with_title(TITLE)
            .with_dimensions(800, 800);
            //.with_fullscreen(monitor));
        let context_builder = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true);
        Ok(AppWindow{window: GlWindow::new(
            window_builder,
            context_builder,
            events_loop
        )?})
    }

    fn activate(&self) -> Result<(), AppError> {
            unsafe { self.window.make_current()?; }
            gl::load_with(|symbol| {
                self.window.get_proc_address(symbol) as *const _
            });
            //TODO: Only print opengl info if debug is set
            print_opengl_info();
            Ok(())
    }

    fn swap_buffers(&self) -> Result<(), AppError> {
        self.window.swap_buffers()?;
        Ok(())
    }

    fn get_size(&self) -> Result<(f32, f32), AppError> {
        self.window.get_inner_size_points()
            .map(|(w, h)| (w as f32, h as f32))
            .ok_or(AppError::Window(
                    "Tried to get size of closed window".to_string()))
    }

    fn display_fps(&self, fps: u32) {
        self.window.set_title(&format!("{} - {} fps", TITLE, fps));
    }
}

fn print_opengl_info() {
    println!("Vendor: {}", glx::get_gl_str(gl::VENDOR));
    println!("Renderer: {}", glx::get_gl_str(gl::RENDERER));
    println!("Version: {}", glx::get_gl_str(gl::VERSION));
    println!("GLSL version: {}", glx::get_gl_str(gl::SHADING_LANGUAGE_VERSION));
    println!("Extensions: {}", glx::get_gl_extensions().join(","));
}

