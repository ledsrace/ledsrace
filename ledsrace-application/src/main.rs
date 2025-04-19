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
use zandvoort::Zandvoort;

// mod driver_info;
mod hd108;
mod zandvoort;

use ledsrace_logic::animation::{
    basic::{SectorFrames, ShowSectors},
    Animations,
};
use ledsrace_logic::{animation::advanced::SunsetGlow, Circuit};
use ledsrace_logic::{
    animation::{basic::StaticColor, AnimationQueue},
    Color,
};
use once_cell::sync::Lazy;
use static_cell::StaticCell;

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
    const OFF: Color = Color(0, 0, 0);
    const PURPLE: Color = Color(101, 10, 50);
    const GREEN: Color = Color(30, 150, 1);
    const YELLOW: Color = Color(160, 106, 2);
    static SUNSET: Animations = Animations::Sunset(SunsetGlow::new());
    static STATIC_COLOR: Animations = Animations::Static(StaticColor::new(Color(255, 0, 0)));

    // Sector animation with purple, green and yellow f1 sector colors
    static SECTOR: Animations = Animations::ShowSectors(ShowSectors::new(PURPLE, GREEN, YELLOW));

    static SECTOR_FRAMES: Lazy<Animations> = Lazy::new(|| {
        let mut frames = SectorFrames::new(Duration::from_millis(1000));
        frames.add_frame([OFF, OFF, OFF]);
        frames.add_frame([PURPLE, OFF, OFF]);
        frames.add_frame([PURPLE, PURPLE, OFF]);
        frames.add_frame([GREEN, PURPLE, OFF]);
        frames.add_frame([GREEN, PURPLE, PURPLE]);
        frames.add_frame([GREEN, GREEN, PURPLE]);
        frames.add_frame([GREEN, GREEN, GREEN]);
        frames.add_frame([YELLOW, GREEN, GREEN]);
        frames.add_frame([YELLOW, YELLOW, GREEN]);
        frames.add_frame([YELLOW, YELLOW, YELLOW]);
        Animations::SectorFrames(frames)
    });

    let mut queue = AnimationQueue::new();

    queue.add_animation(&STATIC_COLOR);
    queue.add_animation(&SUNSET);
    queue.add_animation(&SECTOR);
    queue.add_animation(&*SECTOR_FRAMES);

    receiver.receive().await;

    let frame_duration = Duration::from_millis(20);
    let start = Instant::now();
    let mut ticker = Ticker::every(frame_duration);

    let mut zandvoort: Zandvoort<LED_COUNT> = Zandvoort::new();

    loop {
        // Check for button press
        if let Ok(Message::ButtonPressed) = receiver.try_receive() {
            queue.next_animation();
            let _ = hd108.set_off().await;
        }

        ticker.next().await;
        let _elapsed = start.elapsed();

        // Get current LED states from animation
        queue.render(&mut zandvoort, Instant::now());

        let led_states = zandvoort.led_buffer().get_colors();

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

// #[embassy_executor::task]
// async fn led_task(
//     mut hd108: HD108<impl SpiBus<u8> + 'static, LED_COUNT>,
//     receiver: Receiver<'static, NoopRawMutex, Message, 1>,
// ) {
//     // Define the brightness levels
//     let low_brightness = 10; // Low brightness for background LEDs

//     // Start the train animation immediately
//     let high_brightness = 255;
//     let train_length = 15;
//     let colors = [
//         (high_brightness, 0, 0),
//         (high_brightness, 0, 0),
//         (high_brightness, 0, 0),
//         (high_brightness, 0, 0),
//         (high_brightness, 0, 0),
//         (0, 0, high_brightness),
//         (0, 0, high_brightness),
//         (0, 0, high_brightness),
//         (0, 0, high_brightness),
//         (0, 0, high_brightness),
//         (0, high_brightness, 0),
//         (0, high_brightness, 0),
//         (0, high_brightness, 0),
//         (0, high_brightness, 0),
//         (0, high_brightness, 0),
//     ];

//     let mut iteration_count = 0;

//     while iteration_count < 1 {
//         for i in 1..=LED_COUNT {
//             let mut led_updates: heapless08::Vec<(usize, u8, u8, u8), LED_COUNT> =
//                 heapless08::Vec::new();
//             // Set all LEDs to low brightness
//             for j in 1..=LED_COUNT {
//                 led_updates
//                     .push((j, low_brightness, low_brightness, low_brightness))
//                     .unwrap();
//             }

//             // Update the train LEDs with high brightness colors
//             for j in 0..train_length {
//                 let pos = (i + j) % LED_COUNT;
//                 let color = colors[j];
//                 led_updates[pos] = (pos, color.0, color.1, color.2);
//             }

//             hd108.set_leds(&led_updates).await.unwrap();
//             Timer::after(Duration::from_millis(20)).await;
//         }
//         iteration_count += 1;
//     }

//     println!("Startup animation complete...");

//     // Set all leds off
//     hd108.set_off().await.unwrap();

//     loop {
//         // Wait for the start message
//         receiver.receive().await;

//         println!("Starting race...");
//         let mut ticker = Ticker::every(Duration::from_millis(50));

//         // Start deserialization in chunks
//         let data_bin = include_bytes!("zandvoort_2024_20x20_5hz.bin");
//         let mut remaining_data = &data_bin[..];

//         while !remaining_data.is_empty() {
//             // Attempt to deserialize a single frame from the data using `try_from_bytes`
//             match UpdateFrame::try_from_bytes(remaining_data) {
//                 Ok(frame) => {
//                     // Move the remaining_data pointer forward by the size of the serialized frame
//                     let frame_size = UpdateFrame::SERIALIZED_SIZE;
//                     remaining_data = &remaining_data[frame_size..];

//                     // Prepare LED updates
//                     let mut led_updates: heapless08::Vec<(usize, u8, u8, u8), 216> =
//                         heapless08::Vec::new();
//                     for driver_data in &frame.frame {
//                         if let Some(driver) = DRIVERS
//                             .iter()
//                             .find(|d| d.number == driver_data.driver_number as u32)
//                         {
//                             led_updates
//                                 .push((
//                                     driver_data.led_num as usize,
//                                     driver.color.0,
//                                     driver.color.1,
//                                     driver.color.2,
//                                 ))
//                                 .unwrap();
//                         }
//                     }

//                     // Set the LEDs for this frame
//                     if let Err(err) = hd108.set_leds(&led_updates).await {
//                         println!("Failed to set LEDs: {:?}", err);
//                     }

//                     // Wait for the next frame update
//                     ticker.next().await;
//                 }
//                 Err(_) => {
//                     println!("Failed to deserialize frame");
//                     break;
//                 }
//             }

//             // Check if a stop message was received
//             if receiver.try_receive().is_ok() {
//                 hd108.set_off().await.unwrap();
//                 break;
//             }
//         }

//         // Ensure LEDs are turned off at the end
//         hd108.set_off().await.unwrap();
//     }
// }
