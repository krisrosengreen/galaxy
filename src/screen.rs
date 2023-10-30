use crate::physics::{PhysicsObject, PosMass};

pub const SCREEN_SIZE_X: usize = 150;
pub const SCREEN_SIZE_Y: usize = 45;
pub const Y_SQUISH: f32 = 0.6;

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
    let mut time_last_sleep = std::time::Instant::now();

    loop {
        time.tick();
        screen.tick(&time);
        screen.draw();

        let calc_time = time_last_sleep.elapsed().as_secs_f64();

        std::thread::sleep(std::time::Duration::from_secs_f64(
            (time_delta - calc_time).max(0.0) * TIME_SPEED,
        ));
        screen.clear();

        // For fps
        let difference = time_last.elapsed();
        time_last = std::time::Instant::now();
        time_last_sleep = std::time::Instant::now();
        println!("fps {}", 1000.0 / (difference.as_millis() as f32));
    }
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
    screen: [[char; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
    objs: Vec<Box<dyn PhysicsObject>>,
    obj_buffer: [[u32; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            screen: [[' '; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
            objs: Vec::new(),
            obj_buffer: [[0; SCREEN_SIZE_X]; SCREEN_SIZE_Y],
        }
    }

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

    fn set_position(&mut self, x_index: usize, y_index: usize, luminance: char) {
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
            let pos = obj.get_position();
            let threes = [pos.x, pos.y, obj.get_mass()];

            body_positions.push(threes);
        }

        // Loop over objects again and calculate force
        for primary_obj in self.objs.iter_mut() {
            for secondary_pos in body_positions.iter() {
                // Only calculate forces if the mass present is significant

                #[cfg(not(feature = "starsforces"))]
                if secondary_pos[2] < 100.0 {
                    continue;
                }

                let primary_pos = primary_obj.get_position();
                let difference_pos = (secondary_pos[0] - primary_pos.x).abs()
                    + (secondary_pos[1] - primary_pos.y).abs();

                if difference_pos > 1.0 {
                    primary_obj.apply_grav_attraction(PosMass {
                        x: secondary_pos[0],
                        y: secondary_pos[1],
                        mass: secondary_pos[2],
                    });
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
