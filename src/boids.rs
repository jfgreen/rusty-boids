use cgmath::Point2;

pub struct Simulation {
    pub last_positions : Vec<Point2<f32>>,
}

impl Simulation {
    pub fn new() -> Simulation {
        let dummy_positions = vec![
            Point2::new(300., 200.),
            Point2::new(200., 700.),
            Point2::new(100., 100.),
        ];
        Simulation {
            last_positions: dummy_positions,
        }
    }

    pub fn update(&mut self) {
        for i in 0..self.last_positions.len() {
            self.last_positions[i].x += 0.01;
        }
    }

    pub fn positions(&self) -> &Vec<Point2<f32>> {
        return &self.last_positions;
    }
}

