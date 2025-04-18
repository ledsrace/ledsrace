use core::{cell::Cell, f32};
use embassy_time::Duration;
use libm::sinf;

use crate::{Circuit, Point};

use super::{Animation, Color, Priority};

/// Calculate the geometric center of the circuit
pub fn calculate_center(led_positions: &[Point]) -> Point {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;

    for point in led_positions.iter() {
        sum_x += point.x;
        sum_y += point.y;
    }

    Point::new(
        sum_x / led_positions.len() as f32,
        sum_y / led_positions.len() as f32,
    )
}

pub fn calculate_center_middle(led_positions: &[Point]) -> Point {
    // find minimum and maximum for x and y.
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for point in led_positions.iter() {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
}

/// Get the maximum distance from the center to any LED
pub fn max_distance_from_center(led_positions: &[Point]) -> f32 {
    let center = calculate_center(led_positions);
    let mut max_distance = 0.0;

    for point in led_positions.iter() {
        let distance = point.distance_to(&center);
        if distance > max_distance {
            max_distance = distance;
        }
    }

    max_distance
}

/// Animation that creates a warm, pulsing glow reminiscent of a sunset
pub struct SunsetGlow {
    time_offset: Cell<Duration>,
    base_color: Color,
    highlight_color: Color,
    pulse_speed: f32,
}

unsafe impl Sync for SunsetGlow {}

impl SunsetGlow {
    pub const fn new() -> Self {
        // Warm orange base with a redder highlight for the "sun" effect
        let base_color = Color(255, 100, 0); // Warm orange
        let highlight_color = Color(255, 30, 0); // Reddish

        Self {
            time_offset: Cell::new(Duration::from_millis(0)),
            base_color,
            highlight_color,
            pulse_speed: 3.0, // Faster to make movement more visible
        }
    }

    fn calculate_color(&self, distance_ratio: f32, pulse_intensity: f32) -> Color {
        // Make the blend more dramatic for visible changes
        let blend = (1.0 - distance_ratio * distance_ratio) * pulse_intensity;

        // Interpolate between colors with more contrast
        let base = if distance_ratio < 0.5 {
            self.highlight_color // Use highlight color in center
        } else {
            self.base_color // Use base color towards edges
        };

        let target = if distance_ratio < 0.5 {
            self.base_color // Pulse to base color in center
        } else {
            Color(255, 0, 0) // Pulse to deep red towards edges
        };

        Color(
            base.0
                .saturating_add(((target.0 as f32 - base.0 as f32) * blend) as u8),
            base.1.saturating_sub(((base.1 as f32) * blend) as u8),
            base.2
                .saturating_add(((target.2 as f32 - base.2 as f32) * blend) as u8),
        )
    }
}

impl Animation for SunsetGlow {
    fn reset(&self) {
        self.time_offset.set(Duration::from_millis(0));
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let positions = circuit.led_positions();

        // Update time offset
        self.time_offset.set(timestamp);
        let time = timestamp.as_millis() as f32 / 1000.0;

        // Create two overlapping waves with different frequencies
        let wave1 = sinf(time * self.pulse_speed) * 0.5 + 0.5;
        let wave2 = sinf(time * self.pulse_speed * 0.7 + 1.0) * 0.5 + 0.5;

        let center = calculate_center(positions);
        let max_distance = max_distance_from_center(positions);

        // Update all LEDs
        for (i, pos) in positions.iter().enumerate() {
            let distance = pos.distance_to(&center);
            let distance_ratio = distance / max_distance;

            // Combine waves with distance for more dynamic effect
            let pulse = wave1 * (1.0 - distance_ratio) + wave2 * distance_ratio;

            // Calculate color based on distance and pulse
            let color = self.calculate_color(distance_ratio, pulse);

            // Apply brightness with more contrast and minimum brightness
            let base_brightness = ((1.0 - distance_ratio) * 225.0) as u8;
            let brightness = base_brightness.saturating_add(30); // Ensure minimum brightness of 30

            // Apply brightness to each color component
            let dimmed_color = Color(
                ((color.0 as u32 * brightness as u32) / 255) as u8,
                ((color.1 as u32 * brightness as u32) / 255) as u8,
                ((color.2 as u32 * brightness as u32) / 255) as u8,
            );

            circuit.set_led(i, dimmed_color, Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
