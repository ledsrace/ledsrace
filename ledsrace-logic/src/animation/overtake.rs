use crate::{animation::Animation, Circuit, Color, Priority};
use core::cell::Cell;
use embassy_time::Duration;

/// Animation: Two comets chasing and overtaking each other around the circuit
pub struct OvertakeDuel {
    pub a_pos: Cell<usize>,    // Car A position
    pub b_pos: Cell<usize>,    // Car B position
    pub a_fast: Cell<bool>,    // Is A in catch-up mode?
    pub timer: Cell<u32>,      // Animation time in frames
    pub flash_timer: Cell<u8>, // Flash effect timer
}

unsafe impl Sync for OvertakeDuel {}

const COMET_LEN: usize = 7;
const FLASH_LEN: u8 = 8;
const A_COLOR: Color = Color(200, 30, 30); // Red
const B_COLOR: Color = Color(30, 30, 200); // Blue
const FLASH_COLOR: Color = Color(255, 255, 80); // Yellowish
const WAKE_COLOR: Color = Color(20, 10, 20); // Subtle background

impl OvertakeDuel {
    pub const fn new(led_count: usize) -> Self {
        Self {
            a_pos: Cell::new(0),
            b_pos: Cell::new(led_count / 3), // Start B 1/3 lap ahead
            a_fast: Cell::new(true),
            timer: Cell::new(0),
            flash_timer: Cell::new(0),
        }
    }

    fn step_positions(&self, led_count: usize) {
        let (a_speed, b_speed) = if self.a_fast.get() { (2, 1) } else { (1, 2) };
        self.a_pos.set((self.a_pos.get() + a_speed) % led_count);
        self.b_pos.set((self.b_pos.get() + b_speed) % led_count);
    }

    fn distance(a: usize, b: usize, led_count: usize) -> usize {
        if b >= a {
            b - a
        } else {
            led_count - a + b
        }
    }
}

impl Animation for OvertakeDuel {
    fn reset(&self) {
        self.timer.set(0);
        self.flash_timer.set(0);
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, _timestamp: Duration) {
        update_overtake_duel(self, N);

        let led_count = N;
        // Clear to wake color
        for i in 0..led_count {
            circuit
                .led_buffer()
                .set_led(i, WAKE_COLOR, Priority::Background);
        }
        // Draw comets
        for i in 0..COMET_LEN {
            // Prevent overflow in fade calculation
            let fade = if COMET_LEN > 0 {
                ((COMET_LEN - i) as u8)
                    .saturating_mul(200)
                    .checked_div(COMET_LEN as u8)
                    .unwrap_or(30)
                    .max(30)
            } else {
                30
            };
            let a_idx = (self.a_pos.get() + led_count - i) % led_count;
            let b_idx = (self.b_pos.get() + led_count - i) % led_count;
            circuit
                .led_buffer()
                .set_led(a_idx, Color(fade, fade / 8, fade / 8), Priority::Normal);
            circuit
                .led_buffer()
                .set_led(b_idx, Color(fade / 8, fade / 8, fade), Priority::Normal);
        }
        // Flash on overtake
        if self.flash_timer.get() > 0 {
            let idx = self.a_pos.get();
            for d in 0..4 {
                let fidx = (idx + d) % led_count;
                circuit
                    .led_buffer()
                    .set_led(fidx, FLASH_COLOR, Priority::Normal);
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

/// State update function for OvertakeDuel (call in main loop or timer)
pub fn update_overtake_duel(anim: &OvertakeDuel, led_count: usize) {
    anim.timer.set(anim.timer.get() + 1);
    if anim.flash_timer.get() > 0 {
        anim.flash_timer.set(anim.flash_timer.get() - 1);
    }
    // Check for overtake
    let dist = OvertakeDuel::distance(anim.a_pos.get(), anim.b_pos.get(), led_count);
    if dist < 3 && anim.a_fast.get() {
        anim.flash_timer.set(FLASH_LEN);
        anim.a_fast.set(false); // B now chases A
    } else if dist > led_count / 2 && !anim.a_fast.get() {
        anim.a_fast.set(true); // A resumes chase
    }
    anim.step_positions(led_count);
}
