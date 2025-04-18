#![no_std]

pub mod animation;
pub mod data_frame;

/// Represents a point on the circuit
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        libm::sqrtf(dx * dx + dy * dy)
    }
}

/// Enum representing the different sectors of the circuit
#[derive(Clone, Copy)]
pub enum Sector {
    _1,
    _2,
    _3,
}

/// Trait representing a circuit of LEDs
pub trait Circuit<const N: usize> {
    const LED_COUNT: usize;

    fn led_count(&self) -> usize;

    /// Returns the positions of all LEDs on the circuit
    fn led_positions(&self) -> &'static [Point];

    /// Returns the positions of all LEDs in a specific sector
    fn sectors(&self, sector: Sector) -> &'static [Point];

    /// Returns the indices of all LEDs in a specific sector
    fn sector_indices(&self, sector: Sector) -> core::ops::Range<usize>;

    /// Returns a mutable reference to the LED buffer
    fn led_buffer(&mut self) -> &mut LedStateBuffer<N>;

    /// Set a specific LED's color and priority
    fn set_led(&mut self, index: usize, color: Color, priority: Priority);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Ord, PartialOrd, Eq)]
pub enum Priority {
    Background = 0,
    Normal = 1,
    Warning = 2,
    Critical = 3,
}

/// Represents a buffer of LED states that animations can write to
pub struct LedStateBuffer<const N: usize> {
    states: [(Color, Priority); N],
}

impl<const N: usize> LedStateBuffer<N> {
    pub fn new() -> Self {
        Self {
            states: [(Color(0, 0, 0), Priority::Background); N],
        }
    }

    /// Set LED state if priority is higher than existing
    pub fn set_led(&mut self, index: usize, color: Color, priority: Priority) {
        if index >= N {
            return;
        }

        // Only update if new priority is higher
        if priority >= self.states[index].1 {
            self.states[index] = (color, priority);
        }
    }

    /// Clear the buffer to default state
    pub fn clear(&mut self) {
        self.states = [(Color(0, 0, 0), Priority::Background); N];
    }

    /// Get final LED colors for rendering
    pub fn get_colors(&self) -> &[(Color, Priority)] {
        &self.states
    }
}
