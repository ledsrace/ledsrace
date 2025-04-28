use crate::{animation::Animation, Circuit, Color, Priority};
use embassy_time::Duration;

pub struct DutchFlag;

impl DutchFlag {
    pub const fn new() -> Self {
        Self
    }
}

impl Animation for DutchFlag {
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, _timestamp: Duration) {
        let led_count = circuit.led_count();
        let led_positions = circuit.led_positions();

        // Determine y-coordinate boundaries for each color section
        let y_min = led_positions
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min);
        let y_max = led_positions
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        let section_height = (y_max - y_min) / 3.0;

        for i in 0..led_count {
            let y = led_positions[i].y;
            let color = if y < y_min + section_height {
                Color(0, 0, 255) // Blue
            } else if y < y_min + 2.0 * section_height {
                Color(255, 255, 255) // White
            } else {
                Color(255, 0, 0) // Red
            };
            circuit.set_led(i, color, Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
