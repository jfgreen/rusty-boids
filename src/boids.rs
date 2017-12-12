use std::f32::consts::PI;

use cgmath::{Point2, Vector2};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

//TODO: Have some sort of control for max speed
const MAX_SPEED: f32 = 2.0;
const TWO_PI: f32 = 2. * PI;

struct Boid {
    position: Point2<f32>,
    velocity: Vector2<f32>,
}

impl Boid {
    fn wrap_to(&mut self, width: f32, height: f32) {
        if self.position.x < 0. { self.position.x = width };
        if self.position.y < 0. { self.position.y = height };
        if self.position.x > width { self.position.x = 0. };
        if self.position.y > height { self.position.y = 0. };
    }
}

pub struct Simulation {
    boids: Vec<Boid>,
    width: f32,
    height: f32,
    rng: ThreadRng,
}

impl Simulation {
    pub fn new(size: (f32, f32)) -> Simulation {
        Simulation {
            boids: vec![],
            width: size.0,
            height: size.1,
            rng: rand::thread_rng(),
        }
    }

    pub fn add_boids(&mut self, count: usize) {
        for _ in 0..count {
            let pos = self.random_position();
            let vel = self.random_velocity();
            self.boids.push(Boid{
                position: pos,
                velocity: vel,
            });
        }

    }

    //TODO: Introduce dt to smooth the simulation
    pub fn update(&mut self) {
        //TODO: Add boid behaviours
        for b in &mut self.boids {
            let mut force = Vector2::new(0., 0.,);
            //TODO: Limit velocity to MAX_SPEED
            b.velocity += force;
            b.position += b.velocity;
            //TODO: Could we bounce, or halt instead of wrap
            b.wrap_to(self.width, self.height);
        }
    }

    //TODO: Instead do this with zero copy somehow?
    // Maybe just make renderer accept boids...
    // use two vertex atribs for vel and pos
    // do something pretty with vel...?
    pub fn positions(&self) -> Vec<Point2<f32>> {
        self.boids.iter()
            .map(|b| b.position)
            .collect()
    }

    fn random_position(&mut self) -> Point2<f32> {
        let sim_space_x = Range::new(0., self.width);
        let sim_space_y = Range::new(0., self.height);
        let x = sim_space_x.ind_sample(&mut self.rng);
        let y = sim_space_y.ind_sample(&mut self.rng);
        Point2::new(x, y)
    }

    fn random_velocity(&mut self) -> Vector2<f32> {
        let vel_space = Range::new(0., MAX_SPEED);
        let ang_space = Range::new(0., TWO_PI);
        let s = vel_space.ind_sample(&mut self.rng);
        let a = ang_space.ind_sample(&mut self.rng);
        Basis2::from_angle(Rad(a))
            .rotate_vector(Vector2::new(0., s))
    }
}


