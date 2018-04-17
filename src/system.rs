use std::f32::consts::PI;

use cgmath::{Point2, Vector2, InnerSpace};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

type Position = Point2<f32>;
type Velocity = Vector2<f32>;
type Force = Vector2<f32>;

const TWO_PI: f32 = 2. * PI;

pub struct FlockingSystemParameters {
    max_speed: f32,
}

impl Default for FlockingSystemParameters {
    fn default() -> Self {
        FlockingSystemParameters {
            max_speed: 2.5,
        }
    }
}

pub struct FlockingSystem {
    width: f32, 
    height: f32,
    boid_count: usize,
    dim_x: usize,
    dim_y: usize,
    grid: Vec<usize>,
    positions: Vec<Position>,
    velocities: Vec<Velocity>,
    forces: Vec<Force>,
    parameters: FlockingSystemParameters,
    mouse_position: Position,
    rng: ThreadRng,
}

impl FlockingSystem {
    // TODO: Allow FlockingSystemParameters to be passed in
    // TODO: Use builder pattern for FlockingSystem 
    pub fn new(width: f32, height: f32, req_boid_count: usize) -> Self {

        let (dim_x, dim_y) = calculate_grid_size(width, height, req_boid_count);
        let grid_capacity = dim_x * dim_y;

        // TODO: Use sentinal values so boid count can be exactly as requested
        // Could have a sentinal boid at position 0
        let boid_count = grid_capacity;

        FlockingSystem {
            width,
            height,
            boid_count,
            dim_x,
            dim_y,
            grid: (0..grid_capacity).collect(),
            positions: vec![Position::new(0., 0.); boid_count],
            velocities: vec![Velocity::new(0., 0.); boid_count],
            forces: vec![Force::new(0., 0.); boid_count],
            parameters: FlockingSystemParameters::default(),
            mouse_position: Position::new(0., 0.),
            rng: rand::thread_rng(),
        }
    }


    pub fn randomise(&mut self) {
        self.randomise_positions();
        self.randomise_velocities();
    }

    fn randomise_positions(&mut self) {
        let sim_space_x = Range::new(0., self.width);
        let sim_space_y = Range::new(0., self.height);
        for i in 0..self.boid_count {
            let x = sim_space_x.ind_sample(&mut self.rng);
            let y = sim_space_y.ind_sample(&mut self.rng);
            self.positions[i] = Point2::new(x, y);
        }

    }

    fn randomise_velocities(&mut self) {
        let vel_space = Range::new(0., self.parameters.max_speed);
        let ang_space = Range::new(0., TWO_PI);
        for i in 0..self.boid_count {
            let s = vel_space.ind_sample(&mut self.rng);
            let a = ang_space.ind_sample(&mut self.rng);
            self.velocities[i] = Basis2::from_angle(Rad(a)).rotate_vector(Vector2::new(0., s));
        }

    }

    pub fn centralise(&mut self) {
        let center = Position::new(self.width/2., self.height/2.);
        for i in 0..self.boid_count {
            self.positions[i] = center;
        }
        self.randomise_velocities();
    }

    pub fn zeroise(&mut self) {
        for i in 0..self.boid_count {
            self.positions[i] = Position::new(0., 0.);
        }
        self.randomise_velocities();
    }

    // TODO: Supply a time delta to update so simulation can be frame independant
    pub fn update(&mut self) {

    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_position = Position::new(x, y); 
    }
    
    pub fn positions(&self) -> &[Position] {
        &self.positions
    }
}

fn calculate_grid_size(width: f32, height: f32, desired_count:usize) -> (usize, usize) {
    let aspect_ratio = height / width;
    let mut dim_x = 0;
    let mut dim_y = 0;
    while dim_x * dim_y < desired_count {
        dim_x +=1;
        dim_y = (dim_x as f32 * aspect_ratio) as usize;
    }
    (dim_x, dim_y)
}
