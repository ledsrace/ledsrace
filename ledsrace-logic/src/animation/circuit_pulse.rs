use core::cell::Cell;

use embassy_time::Duration;
use libm::fabsf;

use crate::{Circuit, Color, Priority};

use super::{calculate_center, max_distance_from_center, Animation};

/// Animation that creates pulses moving outward from the center
pub struct CircuitPulse {
    /// How fast the pulse moves (units per second)
    pub speed: f32,
    /// Width of the pulse (in distance units)
    pub pulse_width: f32,
    /// Colors of the pulse
    pub colors: [Color; 3],
    // pub color: Color,
    /// Time between pulses (None for single pulse)
    pub repeat_interval: Option<Duration>,
    /// Time when the animation started
    start_time: Cell<Option<Duration>>,

    /// Whether to randomize the position of the pulse
    randomize: bool,
    /// Whether the animation has finished
    finished: Cell<bool>,
}

unsafe impl Sync for CircuitPulse {}

impl CircuitPulse {
    pub const fn new(
        speed: f32,
        pulse_width: f32,
        colors: [Color; 3],
        repeat_interval: Option<Duration>,
        randomize: bool,
    ) -> Self {
        Self {
            speed,
            pulse_width,
            colors,
            repeat_interval,
            start_time: Cell::new(None),
            randomize,
            finished: Cell::new(false),
        }
    }

    fn calculate_brightness(&self, distance: f32, pulse_distance: f32) -> u8 {
        // Distance from the pulse center
        let delta = fabsf(distance - pulse_distance);

        // If we're within the pulse width, calculate brightness
        if delta <= self.pulse_width {
            // Create a smooth falloff at the edges of the pulse
            let normalized = 1.0 - (delta / self.pulse_width);
            let brightness = normalized * normalized; // Square for smoother falloff
            (brightness * 150.0) as u8 // Max brightness of 150
        } else {
            0
        }
    }
}

impl Animation for CircuitPulse {
    fn reset(&self) {
        self.start_time.set(None);
        self.finished.set(false);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let led_positions = if self.randomize {
            circuit.led_positions_random()
        } else {
            circuit.led_positions()
        };

        // Initialize start time if not set
        if self.start_time.get().is_none() {
            self.start_time.set(Some(timestamp));
        }

        let elapsed = timestamp - self.start_time.get().unwrap();
        let center = calculate_center(led_positions);
        let max_distance = max_distance_from_center(led_positions);

        // Calculate how far the pulse has traveled
        let pulse_distance =
            (elapsed.as_micros() as f32 * 1e-6 * self.speed) % (max_distance * 2.0);

        // let colors = self.colors
        // : Vec<Color, 3> = Vec::from_iter([
        //     Color(150, 0, 100), // Purple color
        //     Color(150, 50, 50), // Purple color
        //     Color(150, 10, 10), // Purple color
        // ]);

        let num_colors = self.colors.len();

        // For each LED, calculate its brightness based on distance from pulse
        for (i, pos) in led_positions.iter().enumerate() {
            let distance = pos.distance_to(&center);
            let brightness = self.calculate_brightness(distance, pulse_distance);

            let color_index = (i % num_colors) as usize;

            if brightness > 0 {
                circuit.set_led(
                    i,
                    Color(
                        (self.colors[color_index].0 as u32 * brightness as u32 / 150) as u8,
                        (self.colors[color_index].1 as u32 * brightness as u32 / 150) as u8,
                        (self.colors[color_index].2 as u32 * brightness as u32 / 150) as u8,
                    ),
                    Priority::Normal,
                );
            }
        }

        // Finish when pulse has moved beyond max distance
        if elapsed.as_micros() as f32 * 1e-6 * self.speed
            > max_distance_from_center(led_positions) * 2.0
        {
            self.finished.set(true);
        }
    }

    fn is_finished(&self) -> bool {
        self.finished.get()
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
