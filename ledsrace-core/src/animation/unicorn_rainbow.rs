use core::cell::Cell;
use embassy_time::Duration;
use libm::sinf;

use crate::{animation::Animation, Circuit, Color, Priority};

pub struct UnicornRainbow {
    speed: f32,
    phase: Cell<f32>,
    wavelength: f32,
}

unsafe impl Sync for UnicornRainbow {}

impl UnicornRainbow {
    pub const fn new(speed: f32, wavelength: f32) -> Self {
        Self {
            speed,
            phase: Cell::new(0.0),
            wavelength,
        }
    }
}

impl Animation for UnicornRainbow {
    fn reset(&self) {
        self.phase.set(0.0);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let t = timestamp.as_micros() as f32 * 1e-6;
        let phase_shift = self.speed * t;
        self.phase.set(self.phase.get() + phase_shift);

        for i in 0..N {
            let position = circuit.led_positions()[i];
            let phase = (position.x / self.wavelength + self.phase.get()) % 1.0;
            let red = ((sinf(phase * core::f32::consts::PI * 2.0) + 1.0) * 127.5) as u8;
            let green = ((sinf((phase + 0.33) * core::f32::consts::PI * 2.0) + 1.0) * 127.5) as u8;
            let blue = ((sinf((phase + 0.66) * core::f32::consts::PI * 2.0) + 1.0) * 127.5) as u8;
            circuit.set_led(i, Color(red, green, blue), Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
