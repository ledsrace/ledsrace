use core::cell::Cell;
use embassy_time::Duration;
use heapless::Vec as HeaplessVec;

use crate::Circuit;

use super::{Animation, Color, Priority};

/// Maximum number of raindrops that can be active at once
const MAX_RAINDROPS: usize = 8;

/// Represents a single raindrop moving along the circuit
struct Raindrop {
    /// Current position (LED index)
    position: usize,
    /// Speed in LEDs per second
    speed: f32,
    /// Length of the tail (in LEDs)
    tail_length: usize,
    /// Hue value (0.0 - 1.0) for color cycling
    hue: f32,
    /// How fast the hue changes
    hue_shift_rate: f32,
}

/// Animation that simulates raindrops racing around the circuit
pub struct RainDropRace {
    /// Active raindrops
    raindrops: HeaplessVec<Raindrop, MAX_RAINDROPS>,
    /// Time of last update
    last_update: Cell<Duration>,
    /// Time of last raindrop creation
    last_raindrop: Cell<Duration>,
    /// Time between raindrop spawns (randomized)
    spawn_interval: Cell<Duration>,
    /// Time of last lightning flash
    last_flash: Cell<Duration>,
    /// Whether a lightning flash is currently active
    flash_active: Cell<bool>,
    /// Duration of the animation (or 0 for infinite)
    duration: Duration,
    /// Start time
    start_time: Cell<Duration>,
}

unsafe impl Sync for RainDropRace {}

impl RainDropRace {
    pub const fn new(duration: Duration) -> Self {
        Self {
            raindrops: HeaplessVec::new(),
            last_update: Cell::new(Duration::from_millis(0)),
            last_raindrop: Cell::new(Duration::from_millis(0)),
            spawn_interval: Cell::new(Duration::from_millis(500)),
            last_flash: Cell::new(Duration::from_millis(0)),
            flash_active: Cell::new(false),
            duration,
            start_time: Cell::new(Duration::from_millis(0)),
        }
    }

    /// Convert HSV to RGB color
    /// h: 0.0 - 1.0 (hue)
    /// s: 0.0 - 1.0 (saturation)
    /// v: 0.0 - 1.0 (value/brightness)
    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> Color {
        let h = h % 1.0;
        let h6 = h * 6.0;
        let i = h6 as u8;
        let f = h6 - i as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Color((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    /// Create a new raindrop at a random position
    fn spawn_raindrop(&mut self, led_count: usize) {
        // Only spawn if we haven't reached the maximum
        if self.raindrops.len() >= MAX_RAINDROPS {
            return;
        }

        // Use a simple hash of the current time to generate pseudo-random values
        let time_hash = self.last_raindrop.get().as_micros() as u32;
        let position = (time_hash % led_count as u32) as usize;

        // Vary speed between 10-30 LEDs per second
        let speed_factor = (time_hash % 20) as f32 + 10.0;

        // Vary tail length between 3-8 LEDs
        let tail_length = ((time_hash % 6) + 3) as usize;

        // Start with a random hue
        let hue = (time_hash % 100) as f32 / 100.0;

        // Vary hue shift rate
        let hue_shift_rate = (time_hash % 10) as f32 / 100.0 + 0.02;

        let raindrop = Raindrop {
            position,
            speed: speed_factor,
            tail_length,
            hue,
            hue_shift_rate,
        };

        self.raindrops.push(raindrop).ok();

        // Set next spawn interval (500-2000ms)
        let next_interval = 500 + (time_hash % 1500);
        self.spawn_interval
            .set(Duration::from_millis(next_interval as u64));
    }

    /// Update raindrop positions based on elapsed time
    fn update_raindrops(&mut self, timestamp: Duration, led_count: usize) {
        let last_time = self.last_update.get();
        let elapsed = timestamp - last_time;
        let elapsed_secs = elapsed.as_micros() as f32 * 1e-6;

        // Update each raindrop
        let mut i = 0;
        while i < self.raindrops.len() {
            let raindrop = &mut self.raindrops[i];

            // Update position
            let distance = (raindrop.speed * elapsed_secs) as usize;
            raindrop.position = (raindrop.position + distance) % led_count;

            // Update hue
            raindrop.hue = (raindrop.hue + raindrop.hue_shift_rate * elapsed_secs) % 1.0;

            i += 1;
        }

        self.last_update.set(timestamp);
    }

    /// Check if it's time to spawn a new raindrop
    fn check_spawn_raindrop(&mut self, timestamp: Duration, led_count: usize) {
        let last_spawn = self.last_raindrop.get();
        let interval = self.spawn_interval.get();

        if timestamp - last_spawn >= interval {
            self.spawn_raindrop(led_count);
            self.last_raindrop.set(timestamp);
        }
    }

    /// Check if it's time for a lightning flash
    fn check_lightning(&self, timestamp: Duration) -> bool {
        let last_flash = self.last_flash.get();

        // Lightning occurs roughly every 5-10 seconds
        if self.flash_active.get() {
            // Flash lasts for 100ms
            if timestamp - last_flash >= Duration::from_millis(100) {
                self.flash_active.set(false);
            }
            true
        } else {
            // Check if it's time for a new flash
            let time_hash = timestamp.as_millis() as u32;
            if timestamp - last_flash >= Duration::from_secs(5) && time_hash % 100 < 2 {
                self.last_flash.set(timestamp);
                self.flash_active.set(true);
                true
            } else {
                false
            }
        }
    }

    /// Render a raindrop with its tail
    fn render_raindrop<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, raindrop: &Raindrop) {
        let led_count = circuit.led_count();

        // Render the head of the raindrop
        let head_color = self.hsv_to_rgb(raindrop.hue, 0.8, 1.0);
        circuit.set_led(raindrop.position, head_color, Priority::Normal);

        // Render the tail with fading brightness
        for i in 1..=raindrop.tail_length {
            let pos = (led_count + raindrop.position - i) % led_count;
            let fade_factor = 1.0 - (i as f32 / (raindrop.tail_length as f32 + 1.0));
            let tail_color = self.hsv_to_rgb(
                raindrop.hue,
                0.8,
                fade_factor * 0.8, // Fade out brightness
            );
            circuit.set_led(pos, tail_color, Priority::Normal);
        }
    }
}

impl Animation for RainDropRace {
    fn reset(&self) {
        self.start_time.set(Duration::from_millis(0));
        self.last_update.set(Duration::from_millis(0));
        self.last_raindrop.set(Duration::from_millis(0));
        self.last_flash.set(Duration::from_millis(0));
        self.flash_active.set(false);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        // Initialize start time if this is the first render
        if self.start_time.get().as_millis() == 0 {
            self.start_time.set(timestamp);
            self.last_update.set(timestamp);
            self.last_raindrop.set(timestamp);
        }

        // Since we can't mutate self directly, we'll update the Cell values and
        // render based on the current state

        // Check for lightning flash
        let lightning_active = self.check_lightning(timestamp);

        // If lightning is active, flash the entire circuit
        if lightning_active {
            // Create a bright white flash
            let flash_color = Color(255, 255, 255);
            for i in 0..circuit.led_count() {
                circuit.set_led(i, flash_color, Priority::Normal);
            }
        } else {
            // Create a static set of raindrops for demonstration
            // Since we can't modify self.raindrops directly in the embedded context,
            // we'll create raindrops on the fly based on the current time

            let time_hash = timestamp.as_micros() as u32;
            let led_count = circuit.led_count();

            // Create 8 raindrops with different positions and colors
            for i in 0..8 {
                // Use a different seed for each raindrop to create variety
                let seed = time_hash.wrapping_add(i as u32 * 12345);

                // Make the raindrops move at different speeds
                let speed = 20.0 + (seed % 40) as f32; // 20-60 LEDs per second

                // Calculate position based on time and speed to create movement
                let base_position = (seed % led_count as u32) as usize;
                let elapsed_secs = timestamp.as_secs() as f32;
                let position = (base_position + (elapsed_secs * speed) as usize) % led_count;

                // Use a different hue for each raindrop, and make it cycle over time
                let hue_offset = (i as f32 * 0.125) + (elapsed_secs * 0.1);
                let hue = hue_offset % 1.0;

                // Create a raindrop at this position with a bright color
                let head_color = self.hsv_to_rgb(hue, 1.0, 1.0);
                circuit.set_led(position, head_color, Priority::Normal);

                // Create a longer, more visible tail
                let tail_length = 6 + (seed % 5) as usize; // 6-10 LEDs
                for j in 1..=tail_length {
                    let pos = (led_count + position - j) % led_count;
                    let fade_factor = 1.0 - (j as f32 / (tail_length as f32 + 1.0));

                    // Keep the same hue but reduce brightness for the tail
                    let tail_color = self.hsv_to_rgb(
                        hue,
                        1.0,
                        fade_factor * 0.9, // Fade out brightness but keep it visible
                    );
                    circuit.set_led(pos, tail_color, Priority::Normal);
                }
            }
        }

        // Update state for next frame
        self.last_update.set(timestamp);

        // Check if it's time to spawn a new raindrop
        let last_spawn = self.last_raindrop.get();
        let interval = self.spawn_interval.get();

        if timestamp - last_spawn >= interval {
            // Update the last raindrop time
            self.last_raindrop.set(timestamp);

            // Set next spawn interval (500-2000ms)
            let time_hash = timestamp.as_micros() as u32;
            let next_interval = 500 + (time_hash % 1500);
            self.spawn_interval
                .set(Duration::from_millis(next_interval as u64));
        }
    }

    fn is_finished(&self) -> bool {
        if self.duration.as_millis() == 0 {
            // Infinite duration
            false
        } else {
            // Check if we've exceeded the duration
            let elapsed = self.last_update.get() - self.start_time.get();
            elapsed >= self.duration
        }
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
