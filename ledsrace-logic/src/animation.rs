use core::cell::{Cell, RefCell};

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Duration, Instant};
use heapless::Vec as HeaplessVec;
use libm::sinf;

pub mod advanced;
pub mod basic;
pub mod valentine;

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

/// Core trait for all animations
pub trait Animation<const N: usize> {
    /// Render the current animation frame into the provided buffer
    /// timestamp: Current animation time
    /// buffer: Buffer to write LED states into
    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>);

    /// Returns true if the animation has finished
    fn is_finished(&self) -> bool;

    /// Get the animation's base priority
    fn priority(&self) -> Priority;

    fn reset(&self) {}
}

/// Manages multiple animations and renders them efficiently
pub struct AnimationManager<const N: usize> {
    animations: HeaplessVec<&'static dyn Animation<N>, 8>, // Fixed max number of animations
    buffer: LedStateBuffer<N>,
    start_time: Instant,
}

impl<const N: usize> AnimationManager<N> {
    pub fn new() -> Self {
        Self {
            animations: HeaplessVec::new(),
            buffer: LedStateBuffer::new(),
            start_time: Instant::now(),
        }
    }

    pub fn add_animation(&mut self, animation: &'static dyn Animation<N>) {
        self.animations.push(animation).ok(); // Ignore if full
    }

    /// Render current frame of all animations
    pub fn render(&mut self, current_time: Instant) -> &[(Color, Priority)] {
        let timestamp = current_time - self.start_time;

        // Clear buffer for new frame
        self.buffer.clear();

        // Render each animation in order (higher priority animations render last)
        for animation in self.animations.iter().filter(|a| !a.is_finished()) {
            animation.render(timestamp, &mut self.buffer);
        }

        self.buffer.get_colors()
    }
}

/// Manages a queue of animations and cycles through them
pub struct AnimationQueue<const N: usize> {
    animations: HeaplessVec<&'static dyn Animation<N>, 8>, // Fixed max number of animations
    current_index: usize,
    buffer: LedStateBuffer<N>,
    start_time: Instant,
}

impl<const N: usize> AnimationQueue<N> {
    pub fn new() -> Self {
        Self {
            animations: HeaplessVec::new(),
            current_index: 0,
            buffer: LedStateBuffer::new(),
            start_time: Instant::now(),
        }
    }

    pub fn add_animation(&mut self, animation: &'static dyn Animation<N>) {
        self.animations.push(animation).ok(); // Ignore if full
    }

    pub fn next_animation(&mut self) {
        if !self.animations.is_empty() {
            self.current_index = (self.current_index + 1) % self.animations.len();
            self.start_time = Instant::now(); // Reset time for new animation
            self.animations.get(self.current_index).unwrap().reset();
        }
    }

    /// Render current frame of the current animation
    pub fn render(&mut self, current_time: Instant) -> &[(Color, Priority)] {
        // Clear buffer for new frame
        self.buffer.clear();

        if let Some(animation) = self.animations.get(self.current_index) {
            // Auto-advance if current animation is finished
            if animation.is_finished() {
                self.next_animation();
                // Get the new animation after advancing
                if let Some(new_animation) = self.animations.get(self.current_index) {
                    new_animation.render(current_time - self.start_time, &mut self.buffer);
                }
            } else {
                animation.render(current_time - self.start_time, &mut self.buffer);
            }
        }

        self.buffer.get_colors()
    }
}

/// Example: Background wave animation that generates colors on the fly
pub struct WaveAnimation {
    pub speed: f32,
    pub wavelength: f32,
}

impl<const N: usize> Animation<N> for WaveAnimation {
    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        let t = timestamp.as_micros() as f32 * 1e-6;

        // Generate wave pattern on the fly
        for i in 0..N {
            let phase = (i as f32 / self.wavelength + t * self.speed) % 1.0;
            // Map sine wave (-1 to 1) to brightness (0 to 150)
            let brightness = ((sinf(phase * core::f32::consts::PI * 2.0) + 1.0) * 75.0) as u8;
            buffer.set_led(
                i,
                Color(brightness, brightness / 10, brightness / 10), // Use all channels for better visibility
                <WaveAnimation as Animation<N>>::priority(&self),
            );
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Background
    }
}
