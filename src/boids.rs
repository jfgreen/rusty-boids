use std::f32::consts::PI;

use cgmath::{Point2, Vector2, InnerSpace};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

//TODO: Have some sort of control for these
//Could have a config file, with a flag to reload on change
const MAX_SPEED: f32 = 2.0;
const MAX_FORCE: f32 = 0.3;
const SEP_WEIGHT: f32 = 1.5;
const ALI_WEIGHT: f32 = 1.0;
const COH_WEIGHT: f32 = 1.0;
const SEP_RADIUS: f32 = 25.0;
const ALI_RADIUS: f32 = 50.0;

const TWO_PI: f32 = 2. * PI;

//TODO: Maybe alias some types?

struct Boid {
    position: Point2<f32>,
    velocity: Vector2<f32>,
}

impl Boid {
    fn apply_force(&mut self, force: Vector2<f32>) {
        //TODO: Limit velocity to MAX_SPEED
        self.velocity += force;
        self.position += self.velocity;
    }

    //TODO: Could we bounce, or halt instead of wrap
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
        for i in 0..self.boids.len() {
            let force = self.react_to_neighbours(i);
            self.apply_force(i, force);
        }
    }

    fn apply_force(&mut self, id: usize, force: Vector2<f32>) {
        let boid = &mut self.boids[id];
        boid.apply_force(force);
        boid.wrap_to(self.width, self.height);
    }

    fn react_to_neighbours(&self, i: usize) -> Vector2<f32> {
        //TODO: Can we use magnitude squared instead to speed up things
        let boid = &self.boids[i];
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.); 
        let mut ali_vel_count = 0;
        for j in 0..self.boids.len() {
            if i != j {
                let other = &self.boids[j];
                let from_neighbour = boid.position - other.position;
                let d = from_neighbour.magnitude();
                if d > 0. {
                    if d < SEP_RADIUS {
                       dodge += from_neighbour.normalize_to(1./d);
                    }
                    if d < ALI_RADIUS {
                        ali_vel_acc += other.velocity;
                        ali_vel_count += 1;
                    }
                }
            }
        }
        let mut force = Vector2::new(0., 0.);
        if dodge.magnitude() > 0. {
            let d_steer = steer(boid, dodge.normalize_to(MAX_SPEED));
            force += SEP_WEIGHT * d_steer;
        }
        if ali_vel_count > 0 {
            let align = ali_vel_acc / ali_vel_count as f32;
            let a_steer = steer(boid, align.normalize_to(MAX_SPEED));
            force += ALI_WEIGHT * a_steer;
        }
        force
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


fn steer(boid: &Boid, target_vel: Vector2<f32>) -> Vector2<f32> {
    let force = target_vel - boid.velocity;
    limit(force, MAX_FORCE)
}

fn limit(vec: Vector2<f32>, max: f32) -> Vector2<f32> {
    if vec.magnitude2() > max*max {
        vec.normalize_to(max)
    } else {
        vec
    }
}

