use core::cell::Cell;

use embassy_time::Duration;
use libm::sinf;

use super::{Animation, Color, LedStateBuffer, Priority};

/// Valentine's Day animation that creates a pulsing heartbeat effect around the track
pub struct ValentineHeartbeat {
    phase: Cell<f32>,
}

unsafe impl Sync for ValentineHeartbeat {}

impl ValentineHeartbeat {
    pub const fn new() -> Self {
        Self {
            phase: Cell::new(0.0),
        }
    }

    /// Creates a heartbeat-like curve using two offset sine waves
    fn heartbeat_curve(x: f32) -> f32 {
        // Create two peaks close together for heartbeat effect
        let peak1 = sinf(x * core::f32::consts::PI * 2.0);
        let peak2 = sinf((x + 0.15) * core::f32::consts::PI * 2.0);
        (peak1 + peak2) * 0.5
    }

    /// Creates a heartbeat-like curve using two offset sine waves
    fn heartbeat_curve2(x: f32) -> f32 {
        // Create two peaks close together for heartbeat effect
        let peak1 = sinf(x * core::f32::consts::PI * 2.0);
        let peak2 = sinf((x + 0.15) * core::f32::consts::PI * 2.0);

        // Square the peaks to make them sharper
        let sharp1 = peak1 * peak1 * if peak1 >= 0.0 { 1.0 } else { -1.0 };
        let sharp2 = peak2 * peak2 * if peak2 >= 0.0 { 1.0 } else { -1.0 };

        (sharp1 + sharp2) * 0.4 // Scale to keep within reasonable bounds
    }
}

impl<const N: usize> Animation<N> for ValentineHeartbeat {
    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        // Update phase (controls movement speed)
        let t = timestamp.as_micros() as f32 * 1e-6;
        let current_phase = (t * 0.5) % 1.0; // Complete cycle every 2 seconds
        self.phase.set(current_phase);

        for i in 0..N {
            // Create wave position based on LED index and current phase
            let pos = ((i as f32 / N as f32) + current_phase) % 1.0;

            // Calculate intensity using heartbeat curve
            let intensity = Self::heartbeat_curve2(pos);

            // Map intensity from [-1, 1] to [0, 1] and apply gamma correction
            let normalized = (intensity + 1.0) * 0.5;

            // Apply a power curve to make low values even lower
            let dimmed = libm::powf(normalized, 2.5);

            // Then apply gamma correction
            let gamma = 2.2;
            let brightness = libm::powf(dimmed, 1.0 / gamma);

            // Create a deep red color with varying brightness
            // let red = (brightness * 155.0) as u8;
            let min_red = 0.0; // Minimum red value for a subtle glow
            let red = (min_red + (brightness * (155.0 - min_red))) as u8;
            let green = (brightness * 15.0) as u8; // Slight green for richness
            let blue = (brightness * 15.0) as u8; // Slight blue for richness

            buffer.set_led(i, Color(red, green, blue), Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
/// A very special Valentine animation that combines a base pulse with sparkle effects.
pub struct ValentineSpecial {
    phase: Cell<f32>,
}

unsafe impl Sync for ValentineSpecial {}

impl ValentineSpecial {
    /// Create a new ValentineSpecial animation.
    pub const fn new() -> Self {
        Self {
            phase: Cell::new(0.0),
        }
    }

    /// Base pulse effect using a simple sine wave.
    fn base_pulse(x: f32) -> f32 {
        // Sine wave giving values in [-1, 1]
        sinf(x * core::f32::consts::PI * 2.0)
    }

    /// Sparkle effect: returns a smooth value if the LED should sparkle, otherwise 0.0.
    /// This is based on LED position `x` and the current time `t`.
    fn sparkle(x: f32, t: f32) -> f32 {
        let pattern = (x * 10.0 + t * 1.0) % 1.0;
        if pattern < 0.3 {
            0.5 * (1.0 + libm::cosf(core::f32::consts::PI * pattern / 0.3))
        } else {
            0.0
        }
    }
}

impl<const N: usize> Animation<N> for ValentineSpecial {
    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        // Update phase based on the elapsed time.
        let t = timestamp.as_micros() as f32 * 1e-6; // Convert to seconds
        let current_phase = (t * 0.05) % 1.0; // Cycle every 2 seconds
        self.phase.set(current_phase);

        for i in 0..N {
            // Determine LED's position along the circuit [0,1)
            let pos = ((i as f32 / N as f32) + current_phase) % 1.0;
            // Compute base pulse intensity.
            let base_intensity = Self::base_pulse(pos);
            // Compute sparkle intensity.
            let sparkle_intensity = Self::sparkle(pos, t);
            // Combine the two to get a composite intensity.
            // We give the sparkle half-weight and clamp the values between -1 and 1.
            let combined = (base_intensity + 0.5 * sparkle_intensity)
                .max(-1.0)
                .min(1.0);
            // Map the combined value from [-1, 1] to [0, 1].
            let normalized = (combined + 1.0) * 0.5;
            // Apply gamma correction for a more natural brightness curve.
            let brightness = libm::powf(normalized, 1.0 / 2.2);

            // Base color: deep red.
            let mut red = (brightness * 200.0) as u8;
            let mut green = (brightness * 10.0) as u8;
            let mut blue = (brightness * 10.0) as u8;

            // If sparkle is active, boost the colors to create a burst effect.
            if sparkle_intensity > 0.0 {
                red = core::cmp::min(255, red as u16 + 80) as u8;
                green = core::cmp::min(255, green as u16 + 80) as u8;
                blue = core::cmp::min(255, blue as u16 + 80) as u8;
            }

            buffer.set_led(i, Color(red, green, blue), Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
