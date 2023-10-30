use crate::screen::Time;
use std::any::Any;

pub const G_CONSTANT: f32 = 0.02;

pub struct PosMass {
    pub x: f32,
    pub y: f32,
    pub mass: f32,
}

pub struct Pos {
    pub x: f32,
    pub y: f32,
}

pub trait PhysicsObject: Any {
    fn get_position(&self) -> Pos;

    fn set_position(&mut self, x: f32, y: f32);

    fn add_position(&mut self, x: f32, y: f32);

    fn add_velocity(&mut self, vx: f32, vy: f32);

    fn tick(&mut self, time: &&Time);

    fn get_force(&mut self) -> &mut Force;

    fn apply_grav_attraction(&mut self, other_pos: PosMass);

    fn get_mass(&self) -> f32;

    fn as_any(&mut self) -> &mut dyn Any;
}

#[allow(non_snake_case)]
pub struct Force {
    pub N_x: f32,
    pub N_y: f32,
}

impl Force {
    pub fn new() -> Self {
        Force { N_x: 0.0, N_y: 0.0 }
    }

    pub fn reset(&mut self) {
        self.N_x = 0.0;
        self.N_y = 0.0;
    }

    pub fn to_velocity(&self, mass: f32, time: &Time) -> [f32; 2] {
        let x = self.N_x / mass * time.delta_time();
        let y = self.N_y / mass * time.delta_time();

        [x, y]
    }
}

pub struct Body {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub force: Force,
}

impl PhysicsObject for Body {
    fn get_position(&self) -> Pos {
        return Pos {
            x: self.x,
            y: self.y,
        };
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn add_position(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    fn add_velocity(&mut self, vx: f32, vy: f32) {
        self.vx += vx;
        self.vy += vy;
    }

    fn tick(&mut self, time: &&Time) {
        // Body physics
        let mass = self.mass;

        // Calculate new velocity
        let velocity_diff = self.force.to_velocity(mass, time);
        self.add_velocity(velocity_diff[0], velocity_diff[1]);

        // Calculate new position
        let position_diff = [self.vx * time.delta_time(), self.vy * time.delta_time()];
        self.add_position(position_diff[0], position_diff[1]);

        // After movement, reset forces
        self.force.reset();
    }

    fn get_force(&mut self) -> &mut Force {
        &mut self.force
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_mass(&self) -> f32 {
        self.mass
    }

    fn apply_grav_attraction(&mut self, other_pos: PosMass) {
        let other_mass = other_pos.mass;

        let mut x_hat = other_pos.x - self.x;
        let mut y_hat = other_pos.y - self.y;

        let size = (x_hat.powf(2.0) + y_hat.powf(2.0)).sqrt();

        x_hat /= size;
        y_hat /= size;

        let force = G_CONSTANT * self.mass * other_mass / size.powf(2.0);
        self.force.N_x += force * x_hat;
        self.force.N_y += force * y_hat;
    }
}
