use crate::physics::{Body, Force, G_CONSTANT};
use crate::screen::*;
use rand::prelude::*;

const STAR_MIN_R: f32 = 2.0;
const STAR_MAX_R: f32 = 22.0;
const STAR_MASS: f32 = 1.0;

pub fn get_orbit_speed(radius: f32, center_mass: f32) -> f32 {
    (G_CONSTANT * center_mass / radius).sqrt()
}

pub fn create_galaxy(
    screen: &mut Screen,
    center_x: f32,
    center_y: f32,
    center_mass: f32,
    num_stars: usize,
    galaxy_vel_x: f32,
    galaxy_vel_y: f32,
) {
    let mut rng = rand::thread_rng();

    // Create center mass
    let center_body = Body {
        mass: center_mass,
        force: Force::new(),
        vx: galaxy_vel_x,
        vy: galaxy_vel_y,
        x: center_x,
        y: center_y,
    };

    screen.add_body(Box::new(center_body));

    for _ in 0..num_stars {
        // Position of star
        let r: f32 = (STAR_MAX_R - STAR_MIN_R) * rng.gen::<f32>() + STAR_MIN_R;
        let theta: f32 = 2.0 * 3.14 * rng.gen::<f32>();

        let x = theta.cos() * r + center_x;
        let y = theta.sin() * r + center_y;

        // Velocity of star
        let speed = get_orbit_speed(r, center_mass);

        let mut x_hat = x - center_x;
        let mut y_hat = y - center_y;

        let size = (x_hat.powf(2.0) + y_hat.powf(2.0)).sqrt();
        x_hat /= size;
        y_hat /= size;

        let x_vel = x_hat * speed;
        let y_vel = y_hat * speed;

        // x_vel and y_vel components are swapped to get vel orth. to center mass
        // flip sign on one component

        let body = Body {
            force: Force::new(),
            mass: STAR_MASS,
            vx: -y_vel + galaxy_vel_x,
            vy: x_vel + galaxy_vel_y,
            x: x,
            y: y,
        };

        screen.add_body(Box::new(body));
    }
}

pub fn run_system() {
    let mut screen = Screen::new();

    let time = Time { current_time: 0.0 };

    create_galaxy(&mut screen, 20.0, 20.0, 100000.0, 600, 6.0, 0.0);

    create_galaxy(&mut screen, 80.0, 50.0, 100000.0, 200, -3.0, -5.5);

    screen_loop(screen, time);
}
