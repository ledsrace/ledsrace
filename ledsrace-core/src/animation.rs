use embassy_time::{Duration, Instant};
use heapless::Vec as HeaplessVec;
use libm::sinf;

use crate::{Circuit, Color, Priority};

mod advanced;
mod basic;
mod circuit_pulse;
mod dutch_flag;
mod ghost_car;
mod growing_trail;
mod lightning_sprint;
mod mexican_wave;
mod overtake;
mod party;
mod raindrop;
mod unicorn_rainbow;

pub use advanced::*;
pub use basic::*;
pub use circuit_pulse::*;
pub use dutch_flag::*;
pub use ghost_car::*;
pub use growing_trail::*;
pub use lightning_sprint::*;
pub use mexican_wave::*;
pub use overtake::*;
pub use party::*;
pub use raindrop::*;
pub use unicorn_rainbow::*;

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
    animations: HeaplessVec<&'static Animations, 12>, // Fixed max number of animations
    current_index: usize,
    /// Start time of the current animation
    start_time: Instant,
    /// Maximum duration of an animation
    max_duration: Duration,
}

impl AnimationQueue {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            animations: HeaplessVec::new(),
            current_index: 0,
            start_time: Instant::now(),
            max_duration,
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
            if animation.is_finished() || self.start_time.elapsed() > self.max_duration {
                self.next_animation();
                // Get the new animation after advancing
                if let Some(new_animation) = self.animations.get(self.current_index) {
                    new_animation.render(
                        circuit,
                        current_time.saturating_duration_since(self.start_time),
                    );
                }
            } else {
                animation.render(
                    circuit,
                    current_time.saturating_duration_since(self.start_time),
                );
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

macro_rules! impl_animation_for_enum {
    ($enum_name:ident, $($variant:ident),+) => {
        impl Animation for $enum_name {
            fn render<const N: usize, C: Circuit<N>>(&self, circuit: &mut C, timestamp: Duration) {
                match self {
                    $( $enum_name::$variant(animation) => animation.render(circuit, timestamp), )+
                }
            }

            fn is_finished(&self) -> bool {
                match self {
                    $( $enum_name::$variant(animation) => animation.is_finished(), )+
                }
            }

            fn priority(&self) -> Priority {
                match self {
                    $( $enum_name::$variant(animation) => animation.priority(), )+
                }
            }

            fn reset(&self) {
                match self {
                    $( $enum_name::$variant(animation) => animation.reset(), )+
                }
            }
        }
    };
}

pub enum Animations {
    Sunset(SunsetGlow),
    Static(StaticColor),
    ShowSectors(ShowSectors),
    SectorFrames(SectorFrames),
    RainDrop(RainDropRace),
    Party(Party),
    OvertakeDuel(OvertakeDuel),
    GhostCar(GhostCar),
    LightningSprint(LightningSprint),
    MexicanWave(MexicanWave),
    UnicornRainbow(UnicornRainbow),
    DutchFlag(DutchFlag),
    CircuitPulse(CircuitPulse),
    GrowingTrail(GrowingTrail),
}

impl_animation_for_enum!(
    Animations,
    Sunset,
    Static,
    ShowSectors,
    SectorFrames,
    RainDrop,
    Party,
    OvertakeDuel,
    GhostCar,
    LightningSprint,
    MexicanWave,
    UnicornRainbow,
    DutchFlag,
    CircuitPulse,
    GrowingTrail
);

// Helper function to scale a Color by the given brightness factor (0.0 to 1.0).
pub fn scale_color(color: Color, brightness: f32) -> Color {
    // Assuming Color is defined as Color(u8, u8, u8)
    let r = (color.0 as f32 * brightness).min(255.0) as u8;
    let g = (color.1 as f32 * brightness).min(255.0) as u8;
    let b = (color.2 as f32 * brightness).min(255.0) as u8;
    Color(r, g, b)
}
