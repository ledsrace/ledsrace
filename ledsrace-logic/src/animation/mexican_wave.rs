use core::cell::Cell;
use embassy_time::Duration;
use libm::{cosf, sinf};

use crate::{animation::Animation, Circuit, Color, Priority};

#[derive(Debug)]
pub struct MexicanWave {
    pub speed: f32,                   // Wave movement speed
    pub wave_width: f32,              // Width of the wave in LEDs
    pub base_color: Color,            // Base color of the wave
    pub time: Cell<f32>,              // Track animation time
    pub sparkle_positions: Cell<u32>, // Bit field for sparkle positions
}

unsafe impl Sync for MexicanWave {}

impl MexicanWave {
    pub const fn new(speed: f32, wave_width: f32, base_color: Color) -> Self {
        Self {
            speed,
            wave_width,
            base_color,
            time: Cell::new(0.0),
            sparkle_positions: Cell::new(0),
        }
    }
}

impl Animation for MexicanWave {
    fn reset(&self) {
        self.time.set(0.0);
        self.sparkle_positions.set(0);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let t = timestamp.as_micros() as f32 * 1e-6;
        self.time.set(t);

        let led_count = circuit.led_count();

        // Update sparkle positions first
        let new_sparkles = (sinf(t * 3.0)).abs() > 0.7; // More frequent sparkles
        if new_sparkles {
            let mut current = self.sparkle_positions.get();
            // Create more sparkle bits and make them move faster
            current = (current.rotate_left(2) | (cosf(t * 17.0) as u32)) & 0x55555555; // Alternate bits
            self.sparkle_positions.set(current);
        }

        // Create the wave pattern with sparkles
        let sparkle_pattern = self.sparkle_positions.get();
        for i in 0..led_count {
            let phase = ((i as f32 / led_count as f32) + t * self.speed) % 1.0;
            let wave = (sinf(phase * core::f32::consts::PI * 2.0) + 1.0) / 2.0;

            let base_color = Color(
                ((self.base_color.0 as f32 * wave) as u8).max(20),
                ((self.base_color.1 as f32 * wave) as u8).max(20),
                ((self.base_color.2 as f32 * wave) as u8).max(20),
            );

            if (sparkle_pattern & (1 << (i % 32))) != 0 {
                // Brighter sparkles that blend with the wave
                let sparkle = Color(
                    (base_color.0 as u16 + 100).min(255) as u8,
                    (base_color.1 as u16 + 100).min(255) as u8,
                    (base_color.2 as u16 + 100).min(255) as u8,
                );
                circuit.set_led(i, sparkle, Priority::Normal);
            } else {
                circuit.set_led(i, base_color, Priority::Normal);
            }
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
