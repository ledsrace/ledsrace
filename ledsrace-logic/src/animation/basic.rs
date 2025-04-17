use core::cell::{Cell, RefCell};

use embassy_time::Duration;
use libm::sinf;

use super::{Animation, Color, LedStateBuffer, Priority};

pub struct StaticColor {
    color: Color,
}

impl StaticColor {
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl<const N: usize> Animation<N> for StaticColor {
    fn render(&self, _timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        for i in 0..N {
            buffer.set_led(i, self.color, Priority::Background);
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Background
    }
}

pub struct Runner {
    index: Cell<usize>,
    last_update: RefCell<Duration>,
}

unsafe impl Sync for Runner {}

impl Runner {
    pub const fn new(index: usize) -> Self {
        Self {
            index: Cell::new(index),
            last_update: RefCell::new(Duration::from_millis(0)),
        }
    }
}

impl<const N: usize> Animation<N> for Runner {
    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        let mut last = self.last_update.borrow_mut();
        let t_diff = timestamp - *last;

        if t_diff >= Duration::from_millis(100) {
            *last = timestamp;
            buffer.set_led(self.index.get(), Color(100, 00, 0), Priority::Normal);
            self.index.set((self.index.get() + 1) % N);
        } else {
            buffer.set_led(self.index.get(), Color(100, 00, 0), Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
