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
const SHELL_GAPS: [usize; 9] = [1750, 701, 301, 132, 57, 23, 10, 4, 1];

pub struct FlockingSystemParameters {
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

//TODO: Put boid count into parameters
impl Default for FlockingSystemParameters {
    fn default() -> Self {
        FlockingSystemParameters {
            // Commented out settings are good for a big flock
            max_speed: 2.5,
            //max_force: 0.4,
            max_force: 0.2,
            mouse_weight: 600.,
            //sep_radius_2: 6_f32.powi(2),
            sep_radius_2: 20_f32.powi(2),
            //ali_radius_2: 11.5_f32.powi(2),
            ali_radius_2: 40_f32.powi(2),
            //coh_radius_2: 11.5_f32.powi(2),
            coh_radius_2: 40_f32.powi(2),
            sep_weight: 1.5,
            ali_weight: 1.0,
            coh_weight: 1.0,
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
        self.refresh_index();
        self.calculate_forces();
        self.update_velocities();
        self.update_positions();
    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_position = Position::new(x, y);
    }

    pub fn positions(&self) -> &[Position] {
        &self.positions
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
            let a = ang_space.ind_sample(&mut self.rng);
            let m = vel_space.ind_sample(&mut self.rng);
            self.velocities[i] = velocity_from_polar(a, m);
        }
    }

    fn refresh_index(&mut self) {
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
                let temp_boid = self.query_boid_index(col, row);
                let temp_pos = self.positions[temp_boid];
                let mut j = col;
                while j >= gap {
                    let curr_boid = self.query_boid_index(j-gap, row);
                    let curr_pos = self.positions[curr_boid];
                    if curr_pos.x < temp_pos.x {
                       self.update_boid_index(j, row, curr_boid);
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != col {
                   self.update_boid_index(j, row, temp_boid);
                }
            }
        }
    }

    fn spatial_shell_pass_columns(&mut self, gap: usize) {
        for col in 0..self.dim_x {
            for row in gap..self.dim_y {
                let temp_boid = self.query_boid_index(col, row);
                let temp_pos = self.positions[temp_boid];
                let mut j = row;
                while j >= gap {
                    let curr_boid = self.query_boid_index(col, j-gap);
                    let curr_pos = self.positions[curr_boid];
                    if curr_pos.y < temp_pos.y {
                       self.update_boid_index(col, j, curr_boid);
                    } else {
                        break;
                    }
                    j -= gap;
                }
                if j != row {
                   self.update_boid_index(col, j, temp_boid);
                }
            }
        }
    }

    fn query_boid_index(&self, column: usize, row: usize) -> usize {
        self.grid[column + (row * self.dim_x)]
    }

    fn update_boid_index(&mut self, column:usize, row:usize, boid: usize) {
        self.grid[column + (row * self.dim_x)] = boid;
    }

    fn calculate_forces(&mut self) {
        // TODO: Pull out position and velocity here and feed into react funcs?
        // It seems that maybe array of structs might be better for this use case
        // TODO: refactor react code to avoid redundant looking up of things
        for row in 0..self.dim_y {
            for col in 0..self.dim_x {
                let boid = self.query_boid_index(col, row);
                let current_vel = self.velocities[boid];
                let current_pos = self.positions[boid];
                let mut force = Vector2::new(0., 0.);
                let neighbours = self.find_neighbours(col, row, current_vel);
                force += self.react_to_neighbours(current_pos, current_vel, &neighbours);
                force += self.react_to_mouse(current_pos);
                self.forces[boid] = force;
            }
        }
    }

    fn react_to_mouse(&mut self, position: Position) -> Force {
        let from_mouse = position - self.mouse_position;
        let dist_sq = from_mouse.magnitude2();
        if dist_sq > 0. {
            let repulse = self.parameters.mouse_weight / dist_sq;
            from_mouse.normalize_to(repulse)
        } else {
            Force::new(0., 0.)
        }
    }

    // TODO: See if we can lose box?
    fn find_neighbours(&self, col: usize, row: usize, vel: Velocity) -> Box<[usize]> {
        let mut neighbourhood = vec![];

        //TODO: Sort the neighbours below into memory access patter order
        //TODO: Could try other "kernals"

        //TODO Remove use of i32, use usize instead
        let neighbours: &[(i32, i32)] = match (vel.x > 0., vel.y > 0., vel.x.abs() > vel.y.abs() ) {
            (true, true, true) =>
                &[(1, -1), (1, 0), (1, 1), (2, 0), (2, 1), (0, 1), (2, 2), (0, -1), (2, -1), (1, 2)],

            (true, true, false) =>
                &[(1, 1), (0, 1), (-1, 1), (0, 2), (1, 2), (1, 0), (2, 2), (-1, 0), (2, 1), (-1, 2)],

            (false, true, false) =>
                &[(1, 1), (0, 1), (-1, 1), (0, 2), (-1, 2), (-1, 0), (-2, 2), (1, 0), (1, 2), (-2, 1)],

            (false, true, true) =>
                &[(-1, 1), (-1, 0), (-1, -1), (-2, 0), (-2, 1), (0, 1), (-2, 2), (0, -1), (-1, 2), (-2, -1)],

            (false, false, true) =>
                &[(-1, 1), (-1, 0), (-1, -1), (-2, 0), (-2, -1), (0, -1), (-2, -2), (0, 1), (-2, 1), (-1, -2)],

            (false, false, false) =>
                &[(-1, -1), (0, -1), (1, -1), (0, -2), (-1, -2), (-1, 0), (-2, -2), (1, 0), (-2, -1), (1, -2)],

            (true, false, false) =>
                &[(-1, -1), (0, -1), (1, -1),  (0, -2), (1, -2), (1, 0), (2, -2), (-1, 0), (-1, -2), (2, -1)],

            (true, false, true) =>
                &[(1, -1), (1, 0), (1, 1), (2, 0), (2, -1), (0, -1), (2, -2), (0, 1), (1, -2), (2, 1)],
        };

        //TODO: Try and remove extra references and casting
        for &(x, y) in neighbours.iter() {
            let nx = (col as i32 + x) as usize;
            let ny = (row as i32 + y) as usize;
            if nx > 0 && nx < self.dim_x && ny > 0 && ny < self.dim_y {
                let neighbour = self.grid[nx + (ny * self.dim_x)];
                neighbourhood.push(neighbour);
            }
        }

        neighbourhood.into_boxed_slice()
    }

    fn react_to_neighbours(&mut self, pos: Position, vel: Velocity, neighbours: &[usize]) -> Force {
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.);
        let mut ali_vel_count = 0;
        let mut coh_pos_acc = Vector2::new(0., 0.);
        let mut coh_pos_count = 0;

        for &other in neighbours {
            let other_pos = self.positions[other];
            let other_vel = self.velocities[other];
            let from_neighbour = pos - other_pos;
            let dist_squared = from_neighbour.magnitude2();
            if dist_squared > 0. {
                if dist_squared < self.parameters.sep_radius_2 {
                    let repulse = 1./dist_squared.sqrt();
                    dodge += from_neighbour.normalize_to(repulse);
                }
                if dist_squared < self.parameters.ali_radius_2 {
                    ali_vel_acc += other_vel;
                    ali_vel_count += 1;
                }
                if dist_squared < self.parameters.coh_radius_2 {
                    coh_pos_acc.x += other_pos.x;
                    coh_pos_acc.y += other_pos.y;
                    coh_pos_count += 1;
                }
            }
        }
        //TODO: Using MAX_SPEED to steer all the things might not be the most pleasing to look at?
        let mut force = Vector2::new(0., 0.);
        if dodge.magnitude2() > 0. {
            let target_d_vel = dodge.normalize_to(self.parameters.max_speed);
            let d_steer = limit(target_d_vel - vel, self.parameters.max_force);
            force += self.parameters.sep_weight * d_steer;
        }
        if ali_vel_count > 0 {
            let align = ali_vel_acc / ali_vel_count as f32;
            let target_a_vel = align.normalize_to(self.parameters.max_speed);
            let a_steer = limit(target_a_vel - vel, self.parameters.max_force);
            force += self.parameters.ali_weight * a_steer;
        }
        if coh_pos_count > 0 {
            let avg_pos = coh_pos_acc / coh_pos_count as f32;
            let boid_pos = Vector2::new(pos.x, pos.y);
            let cohesion = avg_pos - boid_pos;
            let target_c_vel = cohesion.normalize_to(self.parameters.max_speed);
            let c_steer = limit(target_c_vel - vel, self.parameters.max_force);
            force += self.parameters.coh_weight * c_steer;
        }
        force
    }


    fn update_velocities(&mut self) {
        for i in 0..self.boid_count {
            let vel = self.velocities[i] + self.forces[i];
            self.velocities[i] = limit(vel, self.parameters.max_speed);
        }
    }

    fn update_positions(&mut self) {
        //TODO: Is it noticeably slower to do this in a seperate pass from vel update?
        for i in 0..self.boid_count {
            let mut new_pos = self.positions[i] + self.velocities[i];
            //FIXME: horrible hack, find a better way
            if new_pos.x <= 0. { new_pos.x = self.width - 0.1 };
            if new_pos.y <= 0. { new_pos.y = self.height - 0.1 };
            if new_pos.x >= self.width { new_pos.x = 0.1 };
            if new_pos.y >= self.height { new_pos.y = 0.1 };
            self.positions[i] = new_pos
        }
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

fn velocity_from_polar(a: f32, m: f32) -> Velocity {
    Basis2::from_angle(Rad(a)).rotate_vector(Vector2::new(0., m))
}

fn limit(force: Force, max: f32) -> Force {
    if force.magnitude2() > max*max {
        force.normalize_to(max)
    } else {
        force
    }
}
