use std::f32::consts::PI;

use cgmath::{Basis2, InnerSpace, Point2, Rad, Rotation, Rotation2, Vector2};
use rand::{
    self,
    distributions::{IndependentSample, Range},
    ThreadRng,
};

type Position = Point2<f32>;
type Velocity = Vector2<f32>;
type Force = Vector2<f32>;

const TWO_PI: f32 = 2. * PI;
const SHELL_GAPS: [usize; 9] = [1750, 701, 301, 132, 57, 23, 10, 4, 1];

pub struct FlockingConfig {
    pub boid_count: u32,
    pub width: f32,
    pub height: f32,
    pub max_speed: f32,
    pub max_force: f32,
    pub mouse_weight: f32,
    pub sep_weight: f32,
    pub ali_weight: f32,
    pub coh_weight: f32,
    pub sep_radius: f32,
    pub ali_radius: f32,
    pub coh_radius: f32,
}

struct FlockingConstants {
    max_speed: f32,
    max_force: f32,
    mouse_weight: f32,
    sep_radius_2: f32,
    ali_radius_2: f32,
    coh_radius_2: f32,
    sep_weight: f32,
    ali_weight: f32,
    coh_weight: f32,
}

impl FlockingConstants {
    fn from_config(conf: FlockingConfig) -> Self {
        FlockingConstants {
            max_speed: conf.max_speed,
            max_force: conf.max_force,
            mouse_weight: conf.mouse_weight,
            sep_radius_2: conf.sep_radius.powi(2),
            ali_radius_2: conf.ali_radius.powi(2),
            coh_radius_2: conf.coh_radius.powi(2),
            sep_weight: conf.sep_weight,
            ali_weight: conf.ali_weight,
            coh_weight: conf.coh_weight,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Boid {
    position: Position,
    velocity: Velocity,
}

impl Boid {
    fn new() -> Self {
        Boid {
            position: Position::new(0., 0.),
            velocity: Velocity::new(0., 0.),
        }
    }
}

pub struct FlockingSystem {
    width: f32,
    height: f32,
    dim_x: usize,
    dim_y: usize,
    boid_grid: Vec<Boid>,
    forces: Vec<Force>,
    params: FlockingConstants,
    mouse_position: Position,
    mouse_multiplier: f32,
    rng: ThreadRng,
}

impl FlockingSystem {
    pub fn new(conf: FlockingConfig) -> Self {
        // TODO: conf.grid_size()
        let (dim_x, dim_y) = grid_size(conf.width, conf.height, conf.boid_count);
        let grid_capacity = dim_x * dim_y;

        // TODO: Use sentinal values so boid count can be exactly as requested
        // Could have a sentinal boid at position 0
        let boid_count = grid_capacity;

        FlockingSystem {
            width: conf.width,
            height: conf.height,
            dim_x,
            dim_y,
            boid_grid: (0..boid_count).map(|_| Boid::new()).collect(),
            forces: vec![Force::new(0., 0.); boid_count],
            params: FlockingConstants::from_config(conf),
            mouse_position: Position::new(0., 0.),
            mouse_multiplier: 1.,
            rng: rand::thread_rng(),
        }
    }

    pub fn randomise(&mut self) {
        self.randomise_positions();
        self.randomise_velocities();
    }

    pub fn centralise(&mut self) {
        let center = Position::new(self.width / 2., self.height / 2.);
        for boid in &mut self.boid_grid {
            boid.position = center
        }
        self.randomise_velocities();
    }

    pub fn zeroise(&mut self) {
        for boid in &mut self.boid_grid {
            boid.position = Position::new(0., 0.);
        }
        self.randomise_velocities();
    }

    // TODO: Supply a time delta to update so simulation can be frame independant
    pub fn update(&mut self) {
        self.sort_boids();
        self.calculate_forces();
        self.update_boids();
    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_position = Position::new(x, y);
    }

    pub fn enable_mouse_attraction(&mut self) {
        self.mouse_multiplier = -1.;
    }

    pub fn enable_mouse_repulsion(&mut self) {
        self.mouse_multiplier = 1.;
    }

    pub fn boids(&self) -> &[Boid] {
        &self.boid_grid
    }

    fn randomise_positions(&mut self) {
        let sim_space_x = Range::new(0., self.width);
        let sim_space_y = Range::new(0., self.height);
        for boid in &mut self.boid_grid {
            let x = sim_space_x.ind_sample(&mut self.rng);
            let y = sim_space_y.ind_sample(&mut self.rng);
            boid.position = Point2::new(x, y);
        }
    }

    fn randomise_velocities(&mut self) {
        let vel_space = Range::new(0., self.params.max_speed);
        let ang_space = Range::new(0., TWO_PI);
        for boid in &mut self.boid_grid {
            let a = ang_space.ind_sample(&mut self.rng);
            let m = vel_space.ind_sample(&mut self.rng);
            boid.velocity = velocity_from_polar(a, m);
        }
    }

    fn sort_boids(&mut self) {
        //TODO: Could we pick the right starting gap such that we dont need these checks?
        for &gap in SHELL_GAPS.iter() {
            if gap < self.dim_x {
                self.spatial_shell_pass_rows(gap);
            }
            if gap < self.dim_y {
                self.spatial_shell_pass_columns(gap);
            }
        }
    }

    fn spatial_shell_pass_rows(&mut self, gap: usize) {
        for row in 0..self.dim_y {
            for col in gap..self.dim_x {
                let temp_boid = unsafe { self.query_boid_grid(col, row) }.clone();
                let mut j = col;
                while j >= gap {
                    let curr_boid = unsafe { self.query_boid_grid(j - gap, row) };
                    if curr_boid.position.x < temp_boid.position.x {
                        unsafe {
                            self.update_boid_grid(j, row, curr_boid.clone());
                        }
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != col {
                    unsafe {
                        self.update_boid_grid(j, row, temp_boid);
                    }
                }
            }
        }
    }

    fn spatial_shell_pass_columns(&mut self, gap: usize) {
        for col in 0..self.dim_x {
            for row in gap..self.dim_y {
                let temp_boid = unsafe { self.query_boid_grid(col, row) }.clone();
                let mut j = row;
                while j >= gap {
                    let curr_boid = unsafe { self.query_boid_grid(col, j - gap) };
                    if curr_boid.position.y < temp_boid.position.y {
                        unsafe {
                            self.update_boid_grid(col, j, curr_boid.clone());
                        }
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != row {
                    unsafe {
                        self.update_boid_grid(col, j, temp_boid);
                    }
                }
            }
        }
    }

    //TODO: Try and lose these - replace with counter?
    // could have iterators for row wise and column wise? (row, col, index)
    #[inline(always)]
    unsafe fn query_boid_grid(&self, column: usize, row: usize) -> &Boid {
        self.boid_grid.get_unchecked(column + (row * self.dim_x))
    }

    //TODO: As above
    #[inline(always)]
    unsafe fn update_boid_grid(&mut self, column: usize, row: usize, boid: Boid) {
        *self
            .boid_grid
            .get_unchecked_mut(column + (row * self.dim_x)) = boid
    }

    fn calculate_forces(&mut self) {
        //TODO: ROLLY THING
        let mut neighbours = Vec::with_capacity(10); // FIXME: remove hardcoded
        for row in 0..self.dim_y {
            for col in 0..self.dim_x {
                let boid_index = col + (row * self.dim_x);
                let boid = unsafe { self.boid_grid.get_unchecked(boid_index) };
                let mut force = Vector2::new(0., 0.);
                neighbours.clear();
                self.find_neighbours(col, row, boid, &mut neighbours);
                let boid = boid.clone();
                force += self.react_to_neighbours(&boid, &neighbours);
                force += self.react_to_mouse(&boid);
                unsafe { *self.forces.get_unchecked_mut(boid_index) = force };
            }
        }
    }

    fn react_to_mouse(&mut self, boid: &Boid) -> Force {
        let from_mouse = boid.position - self.mouse_position;
        let dist_sq = from_mouse.magnitude2();
        if dist_sq > 0. {
            let repulse = self.params.mouse_weight / dist_sq;
            from_mouse.normalize_to(repulse) * self.mouse_multiplier
        } else {
            Force::new(0., 0.)
        }
    }

    fn find_neighbours(&self, col: usize, row: usize, boid: &Boid, neighbourhood: &mut Vec<Boid>) {
        //TODO: Could try other "kernals"
        //TODO Remove use of i32, use usize instead

        let v = boid.velocity;

        // This is essentially a look up table to determine which flockmates the boid is facing
        #[rustfmt::skip]
        let neighbours = match (v.x > 0., v.y > 0., v.x.abs() > v.y.abs()) {
            (true, true, true) => &[
                (1, -1), (1, 0), (1, 1), (2, 0), (2, 1),
                (0, 1), (2, 2), (0, -1), (2, -1), (1, 2),
            ],

            (true, true, false) => &[
                (1, 1), (0, 1), (-1, 1), (0, 2), (1, 2),
                (1, 0), (2, 2), (-1, 0), (2, 1), (-1, 2),
            ],

            (false, true, false) => &[
                (1, 1), (0, 1), (-1, 1), (0, 2), (-1, 2),
                (-1, 0), (-2, 2), (1, 0), (1, 2), (-2, 1),
            ],

            (false, true, true) => &[
                (-1, 1), (-1, 0), (-1, -1), (-2, 0), (-2, 1),
                (0, 1), (-2, 2), (0, -1), (-1, 2), (-2, -1),
            ],

            (false, false, true) => &[
                (-1, 1), (-1, 0), (-1, -1), (-2, 0), (-2, -1),
                (0, -1), (-2, -2), (0, 1), (-2, 1), (-1, -2),
            ],

            (false, false, false) => &[
                (-1, -1), (0, -1), (1, -1), (0, -2), (-1, -2),
                (-1, 0), (-2, -2), (1, 0), (-2, -1), (1, -2),
            ],

            (true, false, false) => &[
                (-1, -1), (0, -1), (1, -1), (0, -2), (1, -2),
                (1, 0), (2, -2), (-1, 0), (-1, -2), (2, -1),
            ],

            (true, false, true) => &[
                (1, -1), (1, 0), (1, 1), (2, 0), (2, -1),
                (0, -1), (2, -2), (0, 1), (1, -2), (2, 1),
            ],
        };

        //TODO: Try and remove extra references and casting
        for &(x, y) in neighbours.iter() {
            let nx = (col as i32 + x) as usize;
            let ny = (row as i32 + y) as usize;
            if nx > 0 && nx < self.dim_x && ny > 0 && ny < self.dim_y {
                let neighbour = unsafe { self.boid_grid.get_unchecked(nx + (ny * self.dim_x)) };
                neighbourhood.push(neighbour.clone());
            }
        }
    }

    fn react_to_neighbours(&self, boid: &Boid, neighbours: &[Boid]) -> Force {
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.);
        let mut ali_vel_count = 0;
        let mut coh_pos_acc = Vector2::new(0., 0.);
        let mut coh_pos_count = 0;

        for other in neighbours {
            let from_neighbour = boid.position - other.position;
            let dist_squared = from_neighbour.magnitude2();
            if dist_squared > 0. {
                if dist_squared < self.params.sep_radius_2 {
                    let repulse = 1. / dist_squared.sqrt();
                    dodge += from_neighbour.normalize_to(repulse);
                }
                if dist_squared < self.params.ali_radius_2 {
                    ali_vel_acc += other.velocity;
                    ali_vel_count += 1;
                }
                if dist_squared < self.params.coh_radius_2 {
                    coh_pos_acc.x += other.position.x;
                    coh_pos_acc.y += other.position.y;
                    coh_pos_count += 1;
                }
            }
        }
        //TODO: Using MAX_SPEED to steer all the things might not be the most pleasing to look at?
        let mut force = Vector2::new(0., 0.);
        if dodge.magnitude2() > 0. {
            let target_d_vel = dodge.normalize_to(self.params.max_speed);
            let d_steer = limit(target_d_vel - boid.velocity, self.params.max_force);
            force += self.params.sep_weight * d_steer;
        }
        if ali_vel_count > 0 {
            let align = ali_vel_acc / ali_vel_count as f32;
            let target_a_vel = align.normalize_to(self.params.max_speed);
            let a_steer = limit(target_a_vel - boid.velocity, self.params.max_force);
            force += self.params.ali_weight * a_steer;
        }
        if coh_pos_count > 0 {
            let avg_pos = coh_pos_acc / coh_pos_count as f32;
            let boid_pos = Vector2::new(boid.position.x, boid.position.y);
            let cohesion = avg_pos - boid_pos;
            let target_c_vel = cohesion.normalize_to(self.params.max_speed);
            let c_steer = limit(target_c_vel - boid.velocity, self.params.max_force);
            force += self.params.coh_weight * c_steer;
        }
        force
    }

    fn update_boids(&mut self) {
        for (mut boid, force) in self.boid_grid.iter_mut().zip(self.forces.iter()) {
            // Update velocity
            let vel = boid.velocity + force;
            boid.velocity = limit(vel, self.params.max_speed);

            // Update position
            let mut new_pos = boid.position + boid.velocity;
            if new_pos.x <= 0. {
                new_pos.x += self.width;
            }
            if new_pos.y <= 0. {
                new_pos.y += self.height;
            }
            if new_pos.x >= self.width {
                new_pos.x -= self.width;
            }
            if new_pos.y >= self.height {
                new_pos.y -= self.height;
            }
            boid.position = new_pos
        }
    }
}

fn grid_size(width: f32, height: f32, desired_count: u32) -> (usize, usize) {
    let aspect_ratio = width / height;
    let dim_y_unrounded = (desired_count as f32 / aspect_ratio).sqrt();
    let dim_y = dim_y_unrounded.ceil();
    let dim_x = (dim_y_unrounded * aspect_ratio).ceil();
    (dim_x as usize, dim_y as usize)
}

fn velocity_from_polar(a: f32, m: f32) -> Velocity {
    Basis2::from_angle(Rad(a)).rotate_vector(Vector2::new(0., m))
}

fn limit(force: Force, max: f32) -> Force {
    if force.magnitude2() > max * max {
        force.normalize_to(max)
    } else {
        force
    }
}
