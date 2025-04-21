use embassy_time::{Duration, Instant};
use heapless::Vec as HeaplessVec;
use libm::sinf;

use crate::{Circuit, Color, Priority};

mod advanced;
mod basic;
mod ghost_car;
mod lightning_sprint;
mod overtake;
mod raindrop;

pub use advanced::*;
pub use basic::*;
pub use ghost_car::*;
pub use lightning_sprint::*;
pub use overtake::*;
pub use raindrop::*;

/// Core trait for all animations
pub trait Animation {
    /// Render the current animation frame into the provided buffer
    /// circuit: Circuit to render on
    /// timestamp: Current animation time
    /// buffer: Buffer to write LED states into
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration);

    /// Returns true if the animation has finished
    fn is_finished(&self) -> bool;

    /// Get the animation's base priority
    fn priority(&self) -> Priority;

    fn reset(&self) {}
}

// /// Manages multiple animations and renders them efficiently
// pub struct AnimationManager<const N: usize> {
//     animations: HeaplessVec<&'static dyn Animation, 8>, // Fixed max number of animations
//     buffer: LedStateBuffer<N>,
//     start_time: Instant,
// }

// impl<const N: usize> AnimationManager<N> {
//     pub fn new() -> Self {
//         Self {
//             animations: HeaplessVec::new(),
//             buffer: LedStateBuffer::new(),
//             start_time: Instant::now(),
//         }
//     }

//     pub fn add_animation(&mut self, animation: &'static dyn Animation) {
//         self.animations.push(animation).ok(); // Ignore if full
//     }

//     /// Render current frame of all animations
//     pub fn render(&mut self, current_time: Instant) -> &[(Color, Priority)] {
//         let timestamp = current_time - self.start_time;

//         // Clear buffer for new frame
//         self.buffer.clear();

//         // Render each animation in order (higher priority animations render last)
//         for animation in self.animations.iter().filter(|a| !a.is_finished()) {
//             animation.render(timestamp, &mut self.buffer);
//         }

//         self.buffer.get_colors()
//     }
// }

/// Manages a queue of animations and cycles through them
pub struct AnimationQueue {
    animations: HeaplessVec<&'static Animations, 8>, // Fixed max number of animations
    current_index: usize,
    start_time: Instant,
}

impl AnimationQueue {
    pub fn new() -> Self {
        Self {
            animations: HeaplessVec::new(),
            current_index: 0,
            start_time: Instant::now(),
        }
    }

    pub fn add_animation(&mut self, animation: &'static Animations) {
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
    pub fn render<const N: usize, C: Circuit<N>>(
        &mut self,
        circuit: &mut C,
        current_time: Instant,
    ) {
        // Clear buffer for new frame
        circuit.led_buffer().clear();

        if let Some(animation) = self.animations.get(self.current_index) {
            // Auto-advance if current animation is finished
            if animation.is_finished() {
                self.next_animation();
                // Get the new animation after advancing
                if let Some(new_animation) = self.animations.get(self.current_index) {
                    new_animation.render(circuit, current_time - self.start_time);
                }
            } else {
                animation.render(circuit, current_time - self.start_time);
            }
        }
    }
}

/// Example: Background wave animation that generates colors on the fly
pub struct WaveAnimation {
    pub speed: f32,
    pub wavelength: f32,
}

impl Animation for WaveAnimation {
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        let t = timestamp.as_micros() as f32 * 1e-6;

        // Generate wave pattern on the fly
        for i in 0..N {
            let phase = (i as f32 / self.wavelength + t * self.speed) % 1.0;
            // Map sine wave (-1 to 1) to brightness (0 to 150)
            let brightness = ((sinf(phase * core::f32::consts::PI * 2.0) + 1.0) * 75.0) as u8;
            circuit.led_buffer().set_led(
                i,
                Color(brightness, brightness / 10, brightness / 10), // Use all channels for better visibility
                <WaveAnimation as Animation>::priority(&self),
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

pub enum Animations {
    Sunset(SunsetGlow),
    Static(StaticColor),
    ShowSectors(ShowSectors),
    SectorFrames(SectorFrames),
    RainDrop(RainDropRace),
    OvertakeDuel(OvertakeDuel),
    GhostCar(GhostCar),
    LightningSprint(LightningSprint),
}

impl Animation for Animations {
    fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
        match self {
            Animations::Sunset(animation) => animation.render(circuit, timestamp),
            Animations::Static(animation) => animation.render(circuit, timestamp),
            Animations::ShowSectors(animation) => animation.render(circuit, timestamp),
            Animations::SectorFrames(animation) => animation.render(circuit, timestamp),
            Animations::RainDrop(animation) => animation.render(circuit, timestamp),
            Animations::OvertakeDuel(animation) => animation.render(circuit, timestamp),
            Animations::GhostCar(animation) => animation.render(circuit, timestamp),
            Animations::LightningSprint(animation) => animation.render(circuit, timestamp),
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Animations::Sunset(animation) => animation.is_finished(),
            Animations::Static(animation) => animation.is_finished(),
            Animations::ShowSectors(animation) => animation.is_finished(),
            Animations::SectorFrames(animation) => animation.is_finished(),
            Animations::RainDrop(animation) => animation.is_finished(),
            Animations::OvertakeDuel(animation) => animation.is_finished(),
            Animations::GhostCar(animation) => animation.is_finished(),
            Animations::LightningSprint(animation) => animation.is_finished(),
        }
    }

    fn priority(&self) -> Priority {
        match self {
            Animations::Sunset(animation) => animation.priority(),
            Animations::Static(animation) => animation.priority(),
            Animations::ShowSectors(animation) => animation.priority(),
            Animations::SectorFrames(animation) => animation.priority(),
            Animations::RainDrop(animation) => animation.priority(),
            Animations::OvertakeDuel(animation) => animation.priority(),
            Animations::GhostCar(animation) => animation.priority(),
            Animations::LightningSprint(animation) => animation.priority(),
        }
    }
}
