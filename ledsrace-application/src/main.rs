//! Ledsrace main application
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation},
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{GpioPin, Input, InputConfig, Pull},
    spi::{
        master::{Config, Spi, SpiDmaBus},
        Mode,
    },
    time::Rate,
    timer::timg::TimerGroup,
    Async, Blocking,
};
use heapless08::Vec;

mod driver_info;
mod hd108;
use crate::driver_info::DRIVERS;
use static_cell::StaticCell;

use ledsrace_logic::animation::advanced::SunsetGlow;
use ledsrace_logic::animation::advanced::{CircuitPulse, DistanceGradient};
use ledsrace_logic::animation::basic::Runner;
use ledsrace_logic::animation::basic::StaticColor;
use ledsrace_logic::animation::valentine::ValentineHeartbeat;
use ledsrace_logic::animation::valentine::ValentineSpecial;
use ledsrace_logic::animation::AnimationManager;
use ledsrace_logic::animation::AnimationQueue;
use ledsrace_logic::animation::Color;
use ledsrace_logic::animation::WaveAnimation;
use ledsrace_logic::data_frame::UpdateFrame;

use hd108::HD108;

#[cfg(feature = "board20x20")]
const LED_COUNT: usize = 216;

const LED_BUFFER_SIZE: usize = hd108::required_buffer_size::<LED_COUNT>();

type AdcCal = esp_hal::analog::adc::AdcCalLine<esp_hal::peripherals::ADC1>;

enum Message {
    ButtonPressed,
}

static SIGNAL_CHANNEL: StaticCell<Channel<NoopRawMutex, Message, 1>> = StaticCell::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let analog_pin = peripherals.GPIO1;
    let sclk = peripherals.GPIO6;
    let miso = peripherals.GPIO8;
    let mosi = peripherals.GPIO7;
    let cs = peripherals.GPIO9;

    let dma_channel = peripherals.DMA_CH0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(20))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .with_cs(cs)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    static LED_BUF: StaticCell<[u8; LED_BUFFER_SIZE]> = StaticCell::new();

    let hd108: HD108<_, LED_COUNT> = HD108::new(spi, LED_BUF.init([0; LED_BUFFER_SIZE]));
    let signal_channel = SIGNAL_CHANNEL.init(Channel::new());

    // Spawn the led task with the receiver
    spawner
        .spawn(led_task2(hd108, signal_channel.receiver()))
        .unwrap();

    let button_pin = Input::new(
        peripherals.GPIO10,
        InputConfig::default().with_pull(Pull::Up),
    );

    spawner
        .spawn(button_task(button_pin, signal_channel.sender()))
        .unwrap();

    let mut adc1_config = AdcConfig::new();
    let adc1_pin = adc1_config.enable_pin_with_cal::<_, AdcCal>(analog_pin, Attenuation::_11dB);
    let adc1 = Adc::new(peripherals.ADC1, adc1_config);

    spawner.spawn(temperature_task(adc1, adc1_pin)).unwrap();
}

#[embassy_executor::task]
async fn led_task2(
    mut hd108: HD108<SpiDmaBus<'static, Async>, LED_COUNT>,
    receiver: Receiver<'static, NoopRawMutex, Message, 1>,
) {
    static VALENTINE: ValentineHeartbeat = ValentineHeartbeat::new();
    static VALENTINE_SPECIAL: ValentineSpecial = ValentineSpecial::new();
    static WAVE: WaveAnimation = WaveAnimation {
        wavelength: 7.0,
        speed: 1.5,
    };
    // static PULSE: CircuitPulse = CircuitPulse::new(
    //     30.0, // Speed: 30 units per second
    //     10.0, // Pulse width: 10 units
    //     None,
    //     // Some(Duration::from_millis(2000)), // New pulse every 2 seconds
    // );

    static RANDOM_PULSE: CircuitPulse = CircuitPulse::new(
        30.0,                              // Speed: 30 units per second
        10.0,                              // Pulse width: 10 units
        Some(Duration::from_millis(5000)), // New pulse every 2 seconds
        true,                              // Randomize position
    );

    static PULSE: CircuitPulse = CircuitPulse::new(
        50.0,                              // Speed: 30 units per second
        10.0,                              // Pulse width: 10 units
        Some(Duration::from_millis(5000)), // New pulse every 2 seconds
        false,                             // Randomize position
    );

    static GRADIENT: DistanceGradient = DistanceGradient::new(
        Color(0, 150, 150), // Cyan color
        true,               // Fade out from center
        20,                 // Minimum brightness
        150,                // Maximum brightness
    );

    static SUNSET: SunsetGlow = SunsetGlow::new();

    let mut queue = AnimationQueue::<LED_COUNT>::new();

    queue.add_animation(&RANDOM_PULSE);
    queue.add_animation(&PULSE);
    queue.add_animation(&WAVE);
    queue.add_animation(&SUNSET);
    queue.add_animation(&VALENTINE);
    queue.add_animation(&VALENTINE_SPECIAL);

    receiver.receive().await;

    let frame_duration = Duration::from_millis(20);
    let start = Instant::now();
    let mut ticker = Ticker::every(frame_duration);

    loop {
        // Check for button press
        if let Ok(Message::ButtonPressed) = receiver.try_receive() {
            queue.next_animation();
        }

        ticker.next().await;
        let elapsed = start.elapsed();

        // Get current LED states from animation
        let led_states = queue.render(Instant::now());

        // Convert to LED updates
        let mut updates = Vec::<_, LED_COUNT>::new();
        for (i, (color, _)) in led_states.iter().enumerate() {
            updates.push((i, color.0, color.1, color.2)).unwrap();
        }

        // Update LEDs
        hd108.set_leds(&updates).await.unwrap();
    }
}

#[embassy_executor::task]
async fn button_task(
    mut button_pin: Input<'static>,
    sender: Sender<'static, NoopRawMutex, Message, 1>,
) {
    loop {
        // Wait for a button press
        button_pin.wait_for_falling_edge().await;
        esp_println::println!("Button pressed");
        sender.send(Message::ButtonPressed).await;
        Timer::after(Duration::from_millis(400)).await; // Debounce delay
    }
}

fn convert_voltage_to_temperature(pin_mv: u16) -> f32 {
    const V0C: f32 = 400.0; // Output voltage at 0°C in mV
    const TC: f32 = 19.5; // Temperature coefficient in mV/°C

    let voltage = pin_mv as f32; // Convert pin_mv to f32 for calculation
    let temperature_c = (voltage - V0C) / TC;

    temperature_c
}
#[embassy_executor::task]
async fn temperature_task(
    mut adc1: Adc<'static, esp_hal::peripherals::ADC1, Blocking>,
    mut adc1_pin: AdcPin<GpioPin<1>, esp_hal::peripherals::ADC1, AdcCal>,
) {
    loop {
        // Non-blocking read of ADC value
        let mut pin_mv = None;
        loop {
            match adc1.read_oneshot(&mut adc1_pin) {
                Ok(value) => {
                    pin_mv = Some(value);
                    break;
                }
                Err(nb::Error::WouldBlock) => {
                    // ADC is not ready, wait for a short duration to avoid busy-waiting
                    Timer::after(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    // Handle other errors if necessary
                    esp_println::println!("ADC read error: {:?}", e);
                    break;
                }
            }
        }

        if let Some(pin_mv) = pin_mv {
            // Convert to temperature
            let temperature_c = convert_voltage_to_temperature(pin_mv);
            // Print temperature
            esp_println::println!("Temperature: {:.2} °C", temperature_c);
        }

        // Wait for 1 second before the next reading
        Timer::after(Duration::from_secs(1)).await;
    }
}

// #![no_std]
// #![no_main]
// #![feature(type_alias_impl_trait)]

// mod driver_info;
// mod hd108;
// use crate::driver_info::DRIVERS;
// use embassy_executor::Spawner;
// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
// use embassy_sync::channel::Channel;
// use embassy_sync::channel::Receiver;
// use embassy_sync::channel::Sender;
// use embassy_time::Instant;
// use embassy_time::Ticker;
// use embassy_time::{Duration, Timer};
// use embedded_hal_async::spi::SpiBus;
// use esp_backtrace as _;
// use esp_hal::analog::adc::AdcPin;
// use esp_hal::dma::DmaDescriptor;
// use esp_hal::spi::master::prelude::_esp_hal_spi_master_dma_WithDmaSpi2;
// use esp_hal::{
//     analog::adc::{Adc, AdcConfig, Attenuation},
//     clock::ClockControl,
//     dma::{Dma, DmaPriority},
//     gpio::{Event, GpioPin, Input, Io, Pull},
//     peripherals::Peripherals,
//     prelude::*,
//     spi::{master::Spi, SpiMode},
//     system::SystemControl,
//     timer::timg::TimerGroup,
// };
// use esp_println::println;
// use hd108::HD108;
// use heapless08::Vec;
// use ledsrace_logic::animation::advanced::SunsetGlow;
// use ledsrace_logic::animation::advanced::{CircuitPulse, DistanceGradient};
// use ledsrace_logic::animation::basic::Runner;
// use ledsrace_logic::animation::basic::StaticColor;
// use ledsrace_logic::animation::valentine::ValentineHeartbeat;
// use ledsrace_logic::animation::valentine::ValentineSpecial;
// use ledsrace_logic::animation::AnimationManager;
// use ledsrace_logic::animation::AnimationQueue;
// use ledsrace_logic::animation::Color;
// use ledsrace_logic::animation::WaveAnimation;
// use ledsrace_logic::data_frame::UpdateFrame;
// use panic_halt as _;
// use static_cell::StaticCell;

// #[cfg(feature = "board20x20")]
// const LED_COUNT: usize = 216;

// #[cfg(feature = "board10x10")]
// const LED_COUNT: usize = 96;

// const LED_BUFFER_SIZE: usize = hd108::required_buffer_size::<LED_COUNT>();

// enum Message {
//     ButtonPressed,
// }

// static SIGNAL_CHANNEL: StaticCell<Channel<NoopRawMutex, Message, 1>> = StaticCell::new();

// type AdcCal = esp_hal::analog::adc::AdcCalLine<esp_hal::peripherals::ADC1>;

// #[embassy_executor::task]
// async fn button_task(
//     mut button_pin: Input<'static, GpioPin<10>>,
//     sender: Sender<'static, NoopRawMutex, Message, 1>,
// ) {
//     loop {
//         // Wait for a button press
//         button_pin.wait_for_falling_edge().await;
//         sender.send(Message::ButtonPressed).await;
//         Timer::after(Duration::from_millis(400)).await; // Debounce delay
//     }
// }

// #[embassy_executor::task]
// async fn temperature_task(
//     mut adc1: Adc<'static, esp_hal::peripherals::ADC1>,
//     mut adc1_pin: AdcPin<GpioPin<1>, esp_hal::peripherals::ADC1, AdcCal>,
// ) {
//     loop {
//         // Non-blocking read of ADC value
//         let mut pin_mv = None;
//         loop {
//             match adc1.read_oneshot(&mut adc1_pin) {
//                 Ok(value) => {
//                     pin_mv = Some(value);
//                     break;
//                 }
//                 Err(nb::Error::WouldBlock) => {
//                     // ADC is not ready, wait for a short duration to avoid busy-waiting
//                     Timer::after(Duration::from_millis(10)).await;
//                 }
//                 Err(e) => {
//                     // Handle other errors if necessary
//                     println!("ADC read error: {:?}", e);
//                     break;
//                 }
//             }
//         }

//         if let Some(pin_mv) = pin_mv {
//             // Convert to temperature
//             let temperature_c = convert_voltage_to_temperature(pin_mv);
//             // Print temperature
//             println!("Temperature: {:.2} °C", temperature_c);
//         }

//         // Wait for 1 second before the next reading
//         Timer::after(Duration::from_secs(1)).await;
//     }
// }

// // #[embassy_executor::task]
// // async fn led_task(
// //     mut hd108: HD108<impl SpiBus<u8> + 'static, LED_COUNT>,
// //     receiver: Receiver<'static, NoopRawMutex, Message, 1>,
// // ) {
// //     // Define the brightness levels
// //     let low_brightness = 10; // Low brightness for background LEDs

// //     // Start the train animation immediately
// //     let high_brightness = 255;
// //     let train_length = 15;
// //     let colors = [
// //         (high_brightness, 0, 0),
// //         (high_brightness, 0, 0),
// //         (high_brightness, 0, 0),
// //         (high_brightness, 0, 0),
// //         (high_brightness, 0, 0),
// //         (0, 0, high_brightness),
// //         (0, 0, high_brightness),
// //         (0, 0, high_brightness),
// //         (0, 0, high_brightness),
// //         (0, 0, high_brightness),
// //         (0, high_brightness, 0),
// //         (0, high_brightness, 0),
// //         (0, high_brightness, 0),
// //         (0, high_brightness, 0),
// //         (0, high_brightness, 0),
// //     ];

// //     let mut iteration_count = 0;

// //     while iteration_count < 1 {
// //         for i in 1..=LED_COUNT {
// //             let mut led_updates: heapless08::Vec<(usize, u8, u8, u8), LED_COUNT> =
// //                 heapless08::Vec::new();
// //             // Set all LEDs to low brightness
// //             for j in 1..=LED_COUNT {
// //                 led_updates
// //                     .push((j, low_brightness, low_brightness, low_brightness))
// //                     .unwrap();
// //             }

// //             // Update the train LEDs with high brightness colors
// //             for j in 0..train_length {
// //                 let pos = (i + j) % LED_COUNT;
// //                 let color = colors[j];
// //                 led_updates[pos] = (pos, color.0, color.1, color.2);
// //             }

// //             hd108.set_leds(&led_updates).await.unwrap();
// //             Timer::after(Duration::from_millis(20)).await;
// //         }
// //         iteration_count += 1;
// //     }

// //     println!("Startup animation complete...");

// //     // Set all leds off
// //     hd108.set_off().await.unwrap();

// //     loop {
// //         // Wait for the start message
// //         receiver.receive().await;

// //         println!("Starting race...");
// //         let mut ticker = Ticker::every(Duration::from_millis(50));

// //         // Start deserialization in chunks
// //         let data_bin = include_bytes!("zandvoort_2024_20x20_5hz.bin");
// //         let mut remaining_data = &data_bin[..];

// //         while !remaining_data.is_empty() {
// //             // Attempt to deserialize a single frame from the data using `try_from_bytes`
// //             match UpdateFrame::try_from_bytes(remaining_data) {
// //                 Ok(frame) => {
// //                     // Move the remaining_data pointer forward by the size of the serialized frame
// //                     let frame_size = UpdateFrame::SERIALIZED_SIZE;
// //                     remaining_data = &remaining_data[frame_size..];

// //                     // Prepare LED updates
// //                     let mut led_updates: heapless08::Vec<(usize, u8, u8, u8), 216> =
// //                         heapless08::Vec::new();
// //                     for driver_data in &frame.frame {
// //                         if let Some(driver) = DRIVERS
// //                             .iter()
// //                             .find(|d| d.number == driver_data.driver_number as u32)
// //                         {
// //                             led_updates
// //                                 .push((
// //                                     driver_data.led_num as usize,
// //                                     driver.color.0,
// //                                     driver.color.1,
// //                                     driver.color.2,
// //                                 ))
// //                                 .unwrap();
// //                         }
// //                     }

// //                     // Set the LEDs for this frame
// //                     if let Err(err) = hd108.set_leds(&led_updates).await {
// //                         println!("Failed to set LEDs: {:?}", err);
// //                     }

// //                     // Wait for the next frame update
// //                     ticker.next().await;
// //                 }
// //                 Err(_) => {
// //                     println!("Failed to deserialize frame");
// //                     break;
// //                 }
// //             }

// //             // Check if a stop message was received
// //             if receiver.try_receive().is_ok() {
// //                 hd108.set_off().await.unwrap();
// //                 break;
// //             }
// //         }

// //         // Ensure LEDs are turned off at the end
// //         hd108.set_off().await.unwrap();
// //     }
// // }

// fn convert_voltage_to_temperature(pin_mv: u16) -> f32 {
//     const V0C: f32 = 400.0; // Output voltage at 0°C in mV
//     const TC: f32 = 19.5; // Temperature coefficient in mV/°C

//     let voltage = pin_mv as f32; // Convert pin_mv to f32 for calculation
//     let temperature_c = (voltage - V0C) / TC;

//     temperature_c
// }

// #[main]
// async fn main(spawner: Spawner) {
//     println!("Starting program!...");

//     let peripherals = Peripherals::take();
//     let system = SystemControl::new(peripherals.SYSTEM);
//     let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

//     let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
//     esp_hal_embassy::init(&clocks, timg0);

//     let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

//     let analog_pin = io.pins.gpio1;
//     let sclk = io.pins.gpio6;
//     let miso = io.pins.gpio8;
//     let mosi = io.pins.gpio7;
//     let cs = io.pins.gpio9;

//     let mut adc1_config = AdcConfig::new();
//     let adc1_pin =
//         adc1_config.enable_pin_with_cal::<_, AdcCal>(analog_pin, Attenuation::Attenuation11dB);
//     let adc1 = Adc::new(peripherals.ADC1, adc1_config);

//     let dma = Dma::new(peripherals.DMA);

//     let dma_channel = dma.channel0;

//     static TX_DESC: StaticCell<[DmaDescriptor; 8]> = StaticCell::new();
//     let tx_descriptors = TX_DESC.init([DmaDescriptor::EMPTY; 8]);

//     static RX_DESC: StaticCell<[DmaDescriptor; 8]> = StaticCell::new();
//     let rx_descriptors = RX_DESC.init([DmaDescriptor::EMPTY; 8]);

//     let spi = Spi::new(peripherals.SPI2, 20.MHz(), SpiMode::Mode0, &clocks)
//         .with_pins(Some(sclk), Some(mosi), Some(miso), Some(cs))
//         .with_dma(dma_channel.configure_for_async(
//             false,
//             tx_descriptors,
//             rx_descriptors,
//             DmaPriority::Priority0,
//         ));

//     static LED_BUF: StaticCell<[u8; LED_BUFFER_SIZE]> = StaticCell::new();

//     let hd108 = HD108::new(spi, LED_BUF.init([0; LED_BUFFER_SIZE]));

//     // Initialize the button pin as input with interrupt and pull-up resistor
//     let mut button_pin = Input::new(io.pins.gpio10, Pull::Up);

//     // Enable interrupts for the button pin
//     button_pin.listen(Event::FallingEdge);

//     let signal_channel = SIGNAL_CHANNEL.init(Channel::new());

//     // Spawn the button task with ownership of the button pin and the sender
//     spawner
//         .spawn(button_task(button_pin, signal_channel.sender()))
//         .unwrap();

//     // Spawn the led task with the receiver
//     spawner
//         .spawn(led_task2(hd108, signal_channel.receiver()))
//         .unwrap();

//     // Spawn the temperature task
//     spawner.spawn(temperature_task(adc1, adc1_pin)).unwrap();
// }
