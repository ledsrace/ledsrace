use crate::{animation::Animation, Circuit, Color, Priority};
use core::cell::Cell;
use embassy_time::Duration;

pub struct GrowingTrail {
    current_pos: Cell<usize>,
    trail_length: Cell<usize>,
    base_color: Color,
    grow_speed: f32,
    finished: Cell<bool>,
}

impl GrowingTrail {
    pub const fn new(base_color: Color, grow_speed: f32) -> Self {
        Self {
            current_pos: Cell::new(0),
            trail_length: Cell::new(1),
            base_color,
            grow_speed,
            finished: Cell::new(false),
        }
    }
}

unsafe impl Sync for GrowingTrail {}

impl Animation for GrowingTrail {
    fn reset(&self) {
        self.current_pos.set(0);
        self.trail_length.set(1);
        self.finished.set(false);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let led_count = circuit.led_count();
        let pos = self.current_pos.get();
        let next = (pos + 3) % led_count;
        self.current_pos.set(next);

        // Increase trail length over time, slower than the movement of the first LED
        if self.trail_length.get() < led_count {
            let grow_increment = (self.grow_speed * timestamp.as_millis() as f32 / 1000.0) as usize;
            self.trail_length.set(1 + grow_increment);
        }

        if self.trail_length.get() >= led_count {
            self.finished.set(true);
        }

        // Render the growing trail
        for i in 0..self.trail_length.get() {
            let trail_pos = (next + led_count - i) % led_count;
            let intensity = 1.0 - (i as f32 / self.trail_length.get() as f32);
            let scaled = crate::animation::scale_color(self.base_color, intensity);
            circuit.set_led(trail_pos, scaled, Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        self.finished.get()
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
