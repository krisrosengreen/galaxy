use std::any::Any;

pub const SCREEN_SIZE_X: usize = 150;
pub const SCREEN_SIZE_Y: usize = 45;
pub const Y_SQUISH: f32 = 0.6;

pub const G_CONSTANT: f32 = 0.02;

pub const DELTA_TIME: f32 = 1.0 / 60.0;
pub const TIME_SPEED: f64 = 1.0;

const LUMINANCE: [char; 7] = ['.', ',', ':', 'i', 'l', 'w', '@'];
const LUMINANCE_COUNT: u8 = 7;
const LUMINANCE_DENSITY_FACTOR: f32 = 0.9;

#[allow(unused)]
pub fn screen_loop(mut screen: Screen, mut time: Time) {
    let time_delta: f64 = time.delta_time() as f64;

    // For FPS
    let mut time_last = std::time::Instant::now();

    loop {
        time.tick();
        screen.tick(&time);
        screen.draw();
        std::thread::sleep(std::time::Duration::from_secs_f64(time_delta * TIME_SPEED));
        screen.clear();

        // For fps
        //let difference = time_last.elapsed();
        //time_last = std::time::Instant::now();
        //println!("fps {}", 1000.0 / (difference.as_millis() as f32));
    }
}

pub trait PhysicsObject: Any {
    fn get_position(&self) -> [f32; 2];

    fn set_position(&mut self, x: f32, y: f32);

    fn add_position(&mut self, x: f32, y: f32);

    fn add_velocity(&mut self, vx: f32, vy: f32);

    fn tick(&mut self, time: &&Time);

    fn get_force(&mut self) -> &mut Force;

    fn apply_grav_attraction(&mut self, other_pos: [f32; 3]);

    fn get_mass(&self) -> f32;

    fn as_any(&mut self) -> &mut dyn Any;
}

pub struct Time {
    pub current_time: f32,
}

impl Time {
    pub fn tick(&mut self) {
        self.current_time += self.delta_time();
    }

    pub fn delta_time(&self) -> f32 {
        DELTA_TIME
    }
}

pub struct Screen {
    pub screen: [[char; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
    pub objs: Vec<Box<dyn PhysicsObject>>,
    pub obj_buffer: [[u32; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
}

impl Screen {
    pub fn draw(&self) {
        for y_i in 0..SCREEN_SIZE_Y {
            for x_i in 0..SCREEN_SIZE_X {
                if self.screen[y_i][x_i] != ' ' {
                    // Print star density in tile
                    let luminance = (self.obj_buffer[y_i][x_i] as f32 * LUMINANCE_DENSITY_FACTOR)
                        .clamp(0.0, LUMINANCE_COUNT as f32 - 1.0)
                        .floor() as usize;
                    print!("{}", LUMINANCE[luminance]);
                } else {
                    // No object in this tile
                    print!(" ");
                }
            }

            println!(""); // Print new line
        }
    }

    pub fn clear(&mut self) {
        for y_i in 0..SCREEN_SIZE_Y {
            for x_i in 0..SCREEN_SIZE_X {
                self.screen[y_i][x_i] = ' ';
                self.obj_buffer[y_i][x_i] = 0;
            }
        }
        print!("\x1b[H");
    }

    pub fn add_body(&mut self, body: Box<dyn PhysicsObject>) {
        self.objs.push(body);
    }

    fn set_position_buffer(&mut self, x_index: usize, y_index: usize, position_mass: f32) {
        self.obj_buffer[y_index][x_index] += position_mass.floor() as u32;
    }

    pub fn set_position(&mut self, x_index: usize, y_index: usize, luminance: char) {
        self.screen[y_index][x_index] = luminance;
    }

    fn is_drawable(&self, x: f32, y: f32) -> bool {
        x > 0.0 && x < (SCREEN_SIZE_X as f32) && y > 0.0 && y * Y_SQUISH < (SCREEN_SIZE_Y as f32)
    }

    pub fn draw_position(&mut self, x: f32, y: f32, mass: f32) {
        if !self.is_drawable(x, y) {
            return;
        }

        let x_index = x.floor() as usize;
        let y_index: usize = (y * Y_SQUISH).floor() as usize;

        self.set_position(x_index, y_index, '@');
        self.set_position_buffer(x_index, y_index, mass);
    }

    pub fn tick(&mut self, time: &Time) {
        let mut body_positions: Vec<[f32; 3]> = Vec::new();

        // Get position of all objects and tick the object
        for obj in self.objs.iter_mut() {
            // Save object position in vector
            let pairs = obj.get_position();
            let threes = [pairs[0], pairs[1], obj.get_mass()];
            body_positions.push(threes);
        }

        // Loop over objects again and calculate force
        for primary_obj in self.objs.iter_mut() {
            for secondary_pos in body_positions.iter() {
                let primary_pos = primary_obj.get_position();
                let difference_pos = (secondary_pos[0] - primary_pos[0]).abs()
                    + (secondary_pos[1] - primary_pos[1]).abs();

                if difference_pos > 1.0 {
                    primary_obj.apply_grav_attraction(secondary_pos.clone());
                }
            }
        }

        // Tick objects
        for obj in self.objs.iter_mut() {
            obj.tick(&time);
        }

        // Draw positions
        for position in body_positions {
            self.draw_position(position[0], position[1], position[2]);
        }
    }
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
    fn get_position(&self) -> [f32; 2] {
        return [self.x, self.y];
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

    fn apply_grav_attraction(&mut self, other_pos: [f32; 3]) {
        let other_mass = other_pos[2];

        let mut x_hat = other_pos[0] - self.x;
        let mut y_hat = other_pos[1] - self.y;

        let size = (x_hat.powf(2.0) + y_hat.powf(2.0)).sqrt();

        x_hat /= size;
        y_hat /= size;

        let force = G_CONSTANT * self.mass * other_mass / size.powf(2.0);
        self.force.N_x += force * x_hat;
        self.force.N_y += force * y_hat;
    }
}
