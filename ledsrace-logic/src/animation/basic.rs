use core::cell::Cell;

use embassy_time::Duration;
use heapless::Vec;

use crate::{Circuit, Sector};

use super::{Animation, Color, Priority};

pub struct StaticColor {
    color: Color,
}

impl StaticColor {
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Animation for StaticColor {
    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Background
    }

    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, _timestamp: Duration) {
        for i in 0..circuit.led_count() {
            circuit.set_led(i, self.color, Priority::Background);
        }
    }
}

// pub struct Runner {
//     index: Cell<usize>,
//     last_update: RefCell<Duration>,
// }

// unsafe impl Sync for Runner {}

// impl Runner {
//     pub const fn new(index: usize) -> Self {
//         Self {
//             index: Cell::new(index),
//             last_update: RefCell::new(Duration::from_millis(0)),
//         }
//     }
// }

// impl<const N: usize> Animation<N> for Runner {
//     fn render(&self, _circuit: &impl Circuit, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
//         let mut last = self.last_update.borrow_mut();
//         let t_diff = timestamp - *last;

//         if t_diff >= Duration::from_millis(100) {
//             *last = timestamp;
//             buffer.set_led(self.index.get(), Color(100, 00, 0), Priority::Normal);
//             self.index.set((self.index.get() + 1) % N);
//         } else {
//             buffer.set_led(self.index.get(), Color(100, 00, 0), Priority::Normal);
//         }
//     }

//     fn is_finished(&self) -> bool {
//         false
//     }

//     fn priority(&self) -> Priority {
//         Priority::Normal
//     }
// }

pub struct ShowSectors {
    sectors: [Color; 3],
}

impl ShowSectors {
    pub const fn new(sector1: Color, sector2: Color, sector3: Color) -> Self {
        Self {
            sectors: [sector1, sector2, sector3],
        }
    }
}

impl Animation for ShowSectors {
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, _timestamp: Duration) {
        for (c, sector) in [Sector::_1, Sector::_2, Sector::_3].iter().enumerate() {
            for led in circuit.sector_indices(*sector) {
                circuit.set_led(led, self.sectors[c], Priority::Background);
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Background
    }
}

pub struct SectorFrames {
    frames: Vec<[Color; 3], 10>,
    current_frame: Cell<usize>,
    last_update: Cell<Duration>,
    interval: Duration,
}

unsafe impl Sync for SectorFrames {}

impl SectorFrames {
    pub const fn new(interval: Duration) -> Self {
        Self {
            frames: Vec::new(),
            current_frame: Cell::new(0),
            last_update: Cell::new(Duration::from_millis(0)),
            interval,
        }
    }

    pub fn add_frame(&mut self, frame: [Color; 3]) {
        self.frames.push(frame).unwrap();
    }
}

impl Animation for SectorFrames {
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        for (c, sector) in [Sector::_1, Sector::_2, Sector::_3].iter().enumerate() {
            for led in circuit.sector_indices(*sector) {
                circuit.set_led(
                    led,
                    self.frames[self.current_frame.get()][c],
                    Priority::Background,
                );
            }
        }

        let last = self.last_update.get();
        let t_diff = timestamp - last;

        if t_diff >= self.interval {
            self.last_update.set(timestamp);
            self.current_frame
                .set((self.current_frame.get() + 1) % self.frames.len());
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn priority(&self) -> Priority {
        Priority::Background
    }
}
