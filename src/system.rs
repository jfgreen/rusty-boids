//TODO: Compile all the TODOs, have a little think, and rewrite this mess

use std::f32::consts::PI;

use cgmath::{Point2, Vector2, InnerSpace};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

//TODO: See if 64bit is faster?

//FIXME: The most 'pleasing' options depends on the simulation size
//Could use the size of the simulation to come up with pleasing defaults.

//TODO: Have some sort of control for these
//Could have a config file, with a flag to reload on change
const MAX_SPEED: f32 = 2.0;
const MAX_FORCE: f32 = 0.1;
const SEP_WEIGHT: f32 = 1.5;
const ALI_WEIGHT: f32 = 1.0;
const COH_WEIGHT: f32 = 1.0;
const SEP_RADIUS: f32 = 10.0;
const ALI_RADIUS: f32 = 15.5;
const COH_RADIUS: f32 = 15.5;
//const SEP_RADIUS: f32 = 17.0;
//const ALI_RADIUS: f32 = 25.0;
//const COH_RADIUS: f32 = 25.0;

// Maintain squared versions to speed up calculation
const SEP_RADIUS_2: f32 = SEP_RADIUS * SEP_RADIUS;
const ALI_RADIUS_2: f32 = ALI_RADIUS * ALI_RADIUS;
const COH_RADIUS_2: f32 = COH_RADIUS * COH_RADIUS;

//const MOUSE_WEIGHT: f32 = 1000.0;
const MOUSE_WEIGHT: f32 = 600.0;

const TWO_PI: f32 = 2. * PI;

const SHELL_GAPS: [usize; 9] = [1750, 701, 301, 132, 57, 23, 10, 4, 1];

type Position = Point2<f32>;
type Velocity = Vector2<f32>;
type Force = Vector2<f32>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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
        //TODO: Can probably remove this hack now we are using a direct grid structure
        //FIXME: horrible hack, find a better way
        if self.position.x <= 0. { self.position.x = space.width - 0.1 };
        if self.position.y <= 0. { self.position.y = space.height - 0.1 };
        if self.position.x >= space.width { self.position.x = 0.1 };
        if self.position.y >= space.height { self.position.y = 0.1 };
    }
}

//TODO: Make mouse avoid only apply when pressing

pub struct FlockingSystem {
    grid: BoidGrid,
    reactor: BoidReactor,
    space: SimulationSpace,
    mouse_position: Position,
}

impl FlockingSystem {
    pub fn new(width: f32, height: f32, desired_boids: usize) -> FlockingSystem {
        let mut space = SimulationSpace::new(width, height);
        let center = space.center();
        FlockingSystem {
            grid: BoidGrid::new(&mut space, desired_boids),
            reactor: BoidReactor::new(),
            space: space,
            mouse_position: center,
        }
    }

    pub fn resize(&mut self, width: f32, height:f32) {
        self.space.resize(width, height);
        unimplemented!("Need to resize grid"); 
    }

    // TODO: Should these 3 methods really manipulate inside BoidGrid?
    // Have it return an iterator, use a closure or something?
    
    pub fn randomise(&mut self) {
        for boid in &mut self.grid.grid {
           boid.position = self.space.random_position();
           boid.velocity = self.space.random_velocity();
        }
        self.grid.partial_reorder();
    }

    pub fn centralise(&mut self) {
        let center = self.space.center();
        for boid in &mut self.grid.grid {
            boid.position = center;
            boid.velocity = self.space.random_velocity();
        }
        self.grid.partial_reorder();
    }

    pub fn zeroise(&mut self) {
        for boid in &mut self.grid.grid {
            boid.position = Point2::new(0., 0.);
            boid.velocity = self.space.random_velocity();
        }
        self.grid.partial_reorder();
    }

    // TODO: Remove strange density artefacts?
    // Observation, when boids are dense, a larger search radius removes strange artefacts
    //
    // Idea: throw in random "panic" force when they get close - stop these resonances
    // Idea: be dynamic with neighbourhood look up range?
    // Idea: use d^3 instead of d^2 for repulsion

    //TODO: Make simulation frame independant
    pub fn update(&mut self) {

        //TODO: Do away with the need to store intermediate forces by using a double buffer?
        let mut forces = Vec::with_capacity(self.grid.grid.len());
        for _ in 0..self.grid.grid.len() {
            forces.push(Vector2::new(0., 0.));
        }

        //TODO: Can we create an iterator that returns ref to each boid and it's neighbourhood
        for row in 0..self.grid.dim_y {
            for col in 0..self.grid.dim_x {
            let i = self.grid.grid_index(col, row);
            let mut force = Vector2::new(0., 0.);
            let boid = &self.grid.grid[i]; 
            //TODO: Lose box?
            let others = self.grid.neighbourhood(col, row, 1);
            force += self.reactor.react_to_neighbours(boid, &others);
            force += self.reactor.react_to_mouse(boid, self.mouse_position);
            forces[i] = force;
            }
        }


        //TODO: Could we add this to the above functional code?
        for i in 0..self.grid.grid.len() {
            let boid = &mut self.grid.grid[i];
            boid.apply_force(forces[i]);
            boid.wrap_to(&self.space);
        }

        self.grid.partial_reorder();
    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_position = Position::new(x, y);
    }


    //TODO: Instead do this with zero copy somehow?
    // Maybe just make renderer accept boids...
    // use two vertex atribs for vel and pos
    // do something pretty with vel...?
    pub fn positions(&self) -> Vec<Position> {
        self.grid.grid.iter()
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

//TODO: Reconsider where we use usize
struct BoidGrid {
    dim_x: usize,
    dim_y: usize,
    grid: Vec<Boid>,
}

//TODO: Probably flatten things out, then refactor back into nice structs

impl BoidGrid {
    
    //TODO: Find a way to have sentinal values and hence an exact size
    // If we have an invariant that they will always be sorted to the end, then cut short the
    // length of the returned slice
    fn new(space: &mut SimulationSpace, desired_boids: usize) -> Self {
        // Create a grid of that fits at least desired_boids,
        // while fitting the aspect ratio of space
        
        let aspect_ratio = space.aspect_ratio();
        let mut dim_x = 0; 
        let mut dim_y = 0; 
        while dim_x * dim_y < desired_boids {
            dim_x += 1;
            dim_y = (dim_x as f32 * aspect_ratio) as usize;
        }

        let grid_capacity = dim_x*dim_y;
        let mut grid = Vec::with_capacity(grid_capacity);
        for _ in 0..grid_capacity {
            grid.push(Boid{
                position: space.random_position(),
                velocity: space.random_velocity(),
            });
        }

        BoidGrid {
            dim_x,
            dim_y,
            grid,
        }
    }

    //TODO: Experiment with full ordering?
    fn partial_reorder(&mut self) {
        self.spatial_shell_sort(&SHELL_GAPS);
    }

    fn spatial_shell_sort(&mut self, gaps: &[usize]) {
        //TODO: Could we pick the right starting gap such that we dont need these checks?
        for &gap in gaps {
            if gap < self.dim_x {
               self.spatial_shell_pass_rows(gap);
            }
            if gap < self.dim_y {
               self.spatial_shell_pass_columns(gap);
            }
        }

    }

    //TODO: Is relying on copy types ok for perf?
    
    fn spatial_shell_pass_rows(&mut self, gap: usize) {
        for row in 0..self.dim_y {
            for col in gap..self.dim_x {
                let temp = self.get(col, row);
                let mut j = col;
                while j >= gap {
                    let curr = self.get(j-gap, row);
                    if curr.position.x > temp.position.x {
                       self.set(j, row, curr); 
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != col {
                   self.set(j, row, temp); 
                }
            }
        }
    }

    fn spatial_shell_pass_columns(&mut self, gap: usize) {
        for col in 0..self.dim_x {
            for row in gap..self.dim_y {
                let temp = self.get(col, row);
                let mut j = row;
                while j >= gap {
                    let curr = self.get(col, j-gap);
                    if curr.position.y > temp.position.y {
                       self.set(col, j, curr); 
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != row {
                   self.set(col, j, temp); 
                }
            }
        }
    }

    fn grid_index(&self, column: usize, row: usize) -> usize {
       return column + (row * self.dim_x) 
    }

    // TODO: Maybe dont need this?
    fn get(&self, column: usize, row: usize) -> Boid {
        let index = self.grid_index(column, row);
        return self.grid[index];
    }

    // TODO: Maybe dont need this?
    fn get_ref(&self, column: usize, row: usize) -> &Boid {
        let index = self.grid_index(column, row);
        return &self.grid[index];
    }

    // TODO: Maybe dont need this?
    fn set(&mut self, column: usize, row: usize, boid: Boid){
        let index = self.grid_index(column, row);
        self.grid[index] = boid;
    }

    //TODO: Probs lose this, dont need to be boxy really
    fn neighbourhood(&self, col: usize, row: usize, n: usize) -> Box<[&Boid]>{
        let mut neighbourhood = vec![];
        for j in usize::max(row-n, 0)..usize::min(row+n+1, self.dim_y) {
            for i in usize::max(col-n, 0)..usize::min(col+n+1, self.dim_x) {
                if j == row && i == col { continue; }
                neighbourhood.push(self.get_ref(i, j));
            }
        }
        neighbourhood.into_boxed_slice()
    }

    //fn resize(space: &SimulationSpace, desired_size: u32) {
    //    unimplemented!();
    //}
    
    // TODO: Once we do have a more dynamic grid, could implement add/remove?
    // TODO: Implement iterator?
    //
    //TODO: method that takes a closure for updating a boid, handles double buffering and so on
}

//TODO: Class for double buffered grid?

//TOOD: Should space have a rng? Maybe pass in instead.
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

    //TODO Should this really belong to space
    fn random_velocity(&mut self) -> Velocity {
        let vel_space = Range::new(0., MAX_SPEED);
        let ang_space = Range::new(0., TWO_PI);
        let s = vel_space.ind_sample(&mut self.rng);
        let a = ang_space.ind_sample(&mut self.rng);
        Basis2::from_angle(Rad(a))
            .rotate_vector(Vector2::new(0., s))
    }

    fn aspect_ratio(&mut self) -> f32 {
        self.height / self.width
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
    fn react_to_neighbours(&self, boid: &Boid, others: &[&Boid]) -> Force {
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.);
        let mut ali_vel_count = 0;
        let mut coh_pos_acc = Vector2::new(0., 0.);
        let mut coh_pos_count = 0;
        //TODO: Re-implement this? Think - how it might work with a spacial index
        //if i != j {

        //TODO: What we actually want is the KNN (within a radius)
        // this will speed up the sim when boids are closely packed

        let mut n = 0;
        for other in others {
            let from_neighbour = boid.position - other.position;
            let dist_squared = from_neighbour.magnitude2();
            if dist_squared > 0. {
                if dist_squared < SEP_RADIUS_2 {
                    n += 1;
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


