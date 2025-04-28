// LightningSprint animation for a PCB with 216 LEDs laid out like the Zandvoort Racing Circuit
// This animation produces a lightning bolt effect that streaks along the track with a fading trail.

use crate::animation::Animation;
use crate::Color;
use crate::Priority;
use core::cell::Cell;
use embassy_time::Duration;
use once_cell::sync::Lazy;

const NUM_LEDS: usize = 216;
const MAX_TRAIL: usize = 10;

// Precomputed data for the animation; for example, mapping LED indices if needed.
// For now, we just store a sequential mapping as a demonstration. In a real system you could
// compute per-sector LED indices using Circuit trait methods.

pub struct PrecomputedData {
    pub led_indices: [usize; NUM_LEDS],
}

static PRECOMPUTED_DATA: Lazy<PrecomputedData> = Lazy::new(|| PrecomputedData {
    led_indices: {
        let mut arr = [0; NUM_LEDS];
        for i in 0..NUM_LEDS {
            arr[i] = i;
        }
        arr
    },
});

// LightningSprint stores the current head position of the bolt and a fixed-length trail
// showing the previous positions. Interior mutability via core::cell::Cell is used to update state.

pub struct LightningSprint {
    current_pos: Cell<usize>,
    // Each entry holds an Option: Some(led_index) if a trail exists at that position
    trail: [Cell<Option<usize>>; MAX_TRAIL],
    base_color: Color,
}

unsafe impl Sync for LightningSprint {}

impl LightningSprint {
    pub const fn new(base_color: Color) -> Self {
        Self {
            current_pos: Cell::new(0),
            trail: [const { Cell::new(None) }; MAX_TRAIL],
            base_color,
        }
    }
}

impl Animation for LightningSprint {
    fn reset(&self) {
        self.current_pos.set(0);
    }

    // The render method will be called on each frame. It moves the lightning bolt along the LED strip.
    // The current head becomes the brightest LED while previous positions form a fading trail.
    fn render<const N: usize, C: crate::Circuit<N>>(&self, circuit: &mut C, _timestamp: Duration) {
        // Advance the bolt along the LED indices
        let pos = self.current_pos.get();
        let next = (pos + 1) % NUM_LEDS;
        self.current_pos.set(next);

        // Update trail: shift existing trail entries one step back and insert current position at the front
        for i in (1..MAX_TRAIL).rev() {
            self.trail[i].set(self.trail[i - 1].get());
        }
        self.trail[0].set(Some(pos));

        // Render the animation on the circuit: for each LED, set intensity based on whether it's the head or part of the trail
        for i in 0..NUM_LEDS {
            let intensity = if i == next {
                1.0
            } else {
                let mut val = 0.0;
                for (j, cell) in self.trail.iter().enumerate() {
                    if let Some(trail_led) = cell.get() {
                        if trail_led == i {
                            val += exp_decay(0.8, (j as i32) + 1);
                        }
                    }
                }
                val
            };
            let scaled = crate::animation::scale_color(self.base_color, intensity);
            circuit.set_led(i, scaled, Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// Helper function to compute exponentiation by an integer power
fn exp_decay(base: f32, exp: i32) -> f32 {
    let mut result = 1.0;
    for _ in 0..exp {
        result *= base;
    }
    result
}
