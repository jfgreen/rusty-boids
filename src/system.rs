use std::f32::consts::PI;

use cgmath::{Point2, Vector2, InnerSpace};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

//FIXME: The most 'pleasing' options depends on the simulation size
//Could use the size of the simulation to come up with pleasing defaults.

//TODO: Have some sort of control for these
//Could have a config file, with a flag to reload on change
const MAX_SPEED: f32 = 2.0;
const MAX_FORCE: f32 = 0.1;
const SEP_WEIGHT: f32 = 1.5;
const ALI_WEIGHT: f32 = 1.0;
const COH_WEIGHT: f32 = 1.0;
const SEP_RADIUS: f32 = 25.0;
const ALI_RADIUS: f32 = 50.0;
const COH_RADIUS: f32 = 50.0;

// Maintain squared versions to speed up calculation
const SEP_RADIUS_2: f32 = SEP_RADIUS * SEP_RADIUS;
const ALI_RADIUS_2: f32 = ALI_RADIUS * ALI_RADIUS;
const COH_RADIUS_2: f32 = COH_RADIUS * COH_RADIUS;

const MOUSE_WEIGHT: f32 = 1000.0;

const TWO_PI: f32 = 2. * PI;

type Position = Point2<f32>;
type Velocity = Vector2<f32>;
type Force = Vector2<f32>;

#[repr(C)]
struct Boid {
    position: Position,
    velocity: Velocity,
}

impl Boid {
    fn apply_force(&mut self, force: Force) {
        self.velocity += force;
        self.velocity = limit(self.velocity, MAX_SPEED);
        self.position += self.velocity;
    }

    fn wrap_to(&mut self, space: &SimulationSpace) {
        if self.position.x < 0. { self.position.x = space.width };
        if self.position.y < 0. { self.position.y = space.height };
        if self.position.x > space.width { self.position.x = 0. };
        if self.position.y > space.height { self.position.y = 0. };
    }
}

//TODO: Make mouse avoid only apply when pressing

pub struct FlockingSystem {
    boids: Vec<Boid>,
    reactor: BoidReactor,
    space: SimulationSpace,
    mouse_position: Position,
}

impl FlockingSystem {
    pub fn new(width: f32, height: f32) -> FlockingSystem {
        let space = SimulationSpace::new(width, height);
        let center = space.center();
        FlockingSystem {
            boids: vec![],
            reactor: BoidReactor::new(),
            space: space,
            mouse_position: center,
        }
    }

    pub fn add_boids(&mut self, count: usize) {
        for _ in 0..count {
            let pos = self.space.random_position();
            let vel = self.space.random_velocity();
            self.boids.push(Boid{
                position: pos,
                velocity: vel,
            });
        }

    }

    pub fn resize(&mut self, width: f32, height:f32) {
        self.space.resize(width, height)
    }


    pub fn randomise(&mut self) {
        for boid in &mut self.boids {
           boid.position = self.space.random_position();
           boid.velocity = self.space.random_velocity();
        }
    }

    pub fn centralise(&mut self) {
        let center = self.space.center();
        for boid in &mut self.boids {
            boid.position = center;
            boid.velocity = self.space.random_velocity();
        }
    }

    pub fn zeroise(&mut self) {
        for boid in &mut self.boids {
            boid.position = Point2::new(0., 0.);
            boid.velocity = self.space.random_velocity();
        }
    }

    //TODO: Is RC faster than using a usize into an array?
    //TODO: Make simulation frame independant
    pub fn update(&mut self) {
        let forces: Vec<Force> = self.boids.iter()
            .map(|b| self.calculate_force(b))
            .collect();

        for i in 0..self.boids.len() {
            let boid = &mut self.boids[i];
            boid.apply_force(forces[i]);
            boid.wrap_to(&self.space);
        }
    }

    fn calculate_force(&self, boid: &Boid) -> Force {
        let mut force = Vector2::new(0., 0.);
        force += self.reactor.react_to_neighbours(boid, self.boids.as_slice());
        force += self.reactor.react_to_mouse(boid, self.mouse_position);
        force
    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_position = Position::new(x, y);
    }


    //TODO: Instead do this with zero copy somehow?
    // Maybe just make renderer accept boids...
    // use two vertex atribs for vel and pos
    // do something pretty with vel...?
    pub fn positions(&self) -> Vec<Position> {
        self.boids.iter()
            .map(|b| b.position)
            .collect()
    }
}


fn steer(boid: &Boid, target_vel: Velocity) -> Force {
    let force = target_vel - boid.velocity;
    limit(force, MAX_FORCE)
}

fn limit(force: Force, max: f32) -> Force {
    if force.magnitude2() > max*max {
        force.normalize_to(max)
    } else {
        force
    }
}


struct SimulationSpace {
    width: f32,
    height: f32,
    rng: ThreadRng,
}

impl SimulationSpace {

    fn new(width: f32, height: f32) -> SimulationSpace {
        SimulationSpace {
            width,
            height,
            rng: rand::thread_rng(),
        }
    }

    fn resize(&mut self, width: f32, height:f32) {
        self.width = width;
        self.height = height;
    }

    fn center(&self) -> Position {
        Point2::new(self.width/2., self.height/2.)
    }

    fn random_position(&mut self) -> Position {
        let sim_space_x = Range::new(0., self.width);
        let sim_space_y = Range::new(0., self.height);
        let x = sim_space_x.ind_sample(&mut self.rng);
        let y = sim_space_y.ind_sample(&mut self.rng);
        Point2::new(x, y)
    }

    fn random_velocity(&mut self) -> Velocity {
        let vel_space = Range::new(0., MAX_SPEED);
        let ang_space = Range::new(0., TWO_PI);
        let s = vel_space.ind_sample(&mut self.rng);
        let a = ang_space.ind_sample(&mut self.rng);
        Basis2::from_angle(Rad(a))
            .rotate_vector(Vector2::new(0., s))
    }

}

struct BoidReactor {
    //TODO: This is where the simulation params can go
}

impl BoidReactor {

    fn new() -> BoidReactor {
        //TODO: This is where config could be unpacked
        BoidReactor {}
    }

    //TODO: At some point, use spacial data structure
    //TODO: Break this up a bit
    fn react_to_neighbours(&self, boid: &Boid, boids: &[Boid]) -> Force {
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.);
        let mut ali_vel_count = 0;
        let mut coh_pos_acc = Vector2::new(0., 0.);
        let mut coh_pos_count = 0;
        for j in 0..boids.len() {
            //TODO: Re-implement this? Think - how it might work with a spacial index
            //if i != j {
                let other = &boids[j];
                let from_neighbour = boid.position - other.position;
                let dist_squared = from_neighbour.magnitude2();
                if dist_squared > 0. {
                    if dist_squared < SEP_RADIUS_2 {
                        let repulse = 1./dist_squared.sqrt();
                        dodge += from_neighbour.normalize_to(repulse);
                    }
                    if dist_squared < ALI_RADIUS_2 {
                        ali_vel_acc += other.velocity;
                        ali_vel_count += 1;
                    }
                    if dist_squared < COH_RADIUS_2 {
                        coh_pos_acc.x += other.position.x;
                        coh_pos_acc.y += other.position.y;
                        coh_pos_count += 1;
                    }
                }
            //}
        }
        let mut force = Vector2::new(0., 0.);
        if dodge.magnitude2() > 0. {
            let d_steer = steer(boid, dodge.normalize_to(MAX_SPEED));
            force += SEP_WEIGHT * d_steer;
        }
        if ali_vel_count > 0 {
            let align = ali_vel_acc / ali_vel_count as f32;
            let a_steer = steer(boid, align.normalize_to(MAX_SPEED));
            force += ALI_WEIGHT * a_steer;
        }
        if coh_pos_count > 0 {
            let avg_pos = coh_pos_acc / coh_pos_count as f32;
            let boid_pos = Vector2::new(boid.position.x, boid.position.y);
            let cohesion = avg_pos - boid_pos;
            let c_steer = steer(boid, cohesion.normalize_to(MAX_SPEED));
            force += COH_WEIGHT * c_steer;
        }
        force
    }

    fn react_to_mouse(&self, boid: &Boid, mouse_position: Position) -> Force {
        let from_mouse = boid.position - mouse_position;
        let dist_sq = from_mouse.magnitude2();
        if dist_sq > 0. {
            let repulse = MOUSE_WEIGHT / dist_sq;
            from_mouse.normalize_to(repulse)
        } else {
            Vector2::new(0., 0.)
        }
    }
}
