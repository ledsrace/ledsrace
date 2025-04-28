// ghost_car.rs

use core::cell::Cell;
use embassy_time::Duration;

use crate::{animation::Animation, Circuit, Color, Priority, Sector};

#[derive(Debug)]
pub struct GhostCar {
    pub speed: f32,
    pub car_length: usize,
    pub color: Color,
    pub current_position: Cell<f32>, // Position as a float to allow sub-LED movement
}

unsafe impl Sync for GhostCar {}

impl GhostCar {
    pub const fn new(speed: f32, car_length: usize, color: Color) -> Self {
        Self {
            speed,
            car_length,
            color,
            current_position: Cell::new(0.0),
        }
    }
}

impl Animation for GhostCar {
    fn reset(&self) {
        self.current_position.set(0.0);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let led_count = circuit.led_count();
        let t = timestamp.as_secs() as f32;
        let position = (self.current_position.get() + self.speed * t) % led_count as f32;
        let start_index = position as usize;

        // Get current sector
        let sector = match start_index {
            i if i < 77 => Sector::_1,
            i if i < 153 => Sector::_2,
            _ => Sector::_3,
        };

        // Get LEDs in the current sector
        let sector_leds = circuit.sectors(sector);

        // Set the color of the LEDs in the sector
        let sector_color = Color(self.color.2, self.color.1, self.color.0); // Complementary color
        for led in sector_leds {
            if let Some(index) = circuit
                .led_positions()
                .iter()
                .position(|&p| p.x == led.x && p.y == led.y)
            {
                circuit.set_led(index, sector_color, Priority::Background);
            }
        }

        // Render the ghost car
        for i in 0..self.car_length {
            let led_index = (start_index + i) % led_count;
            let brightness = (255.0 * (1.0 - (i as f32 / self.car_length as f32))) as u8;
            let color = Color(brightness, brightness, brightness); // White color with fading
            circuit.set_led(led_index, color, Priority::Normal);
        }

        self.current_position
            .set(self.current_position.get() + self.speed * t);
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
