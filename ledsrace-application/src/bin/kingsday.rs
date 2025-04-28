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

use ledsrace_logic::{animation::*, Circuit, Color};
use once_cell::sync::Lazy;
use static_cell::StaticCell;

use ledsrace::hd108::HD108;
use ledsrace::zandvoort::Zandvoort;

#[cfg(feature = "board20x20")]
const LED_COUNT: usize = 216;

const LED_BUFFER_SIZE: usize = ledsrace::hd108::required_buffer_size::<LED_COUNT>();

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
    const ORANGE: Color = Color(255, 20, 0);
    const RED: Color = Color(255, 0, 0);
    const WHITE: Color = Color(255, 255, 255);
    const BRIGHT_YELLOW: Color = Color(255, 105, 0);

    // static OVERDUEL: StaticCell<OvertakeDuel> = StaticCell::new();
    // static OVERDUEL_ANIM: StaticCell<Animations> = StaticCell::new();
    static OVERDUEL_ANIM: Lazy<Animations> =
        Lazy::new(|| Animations::OvertakeDuel(OvertakeDuel::new(LED_COUNT)));

    static LIGHTNING_SPRINT: Lazy<Animations> = Lazy::new(|| {
        let anim = LightningSprint::new(ORANGE); // Dutch orange for Kingsday
        Animations::LightningSprint(anim)
    });

    static CIRCUIT_PULSE: Lazy<Animations> = Lazy::new(|| {
        Animations::CircuitPulse(CircuitPulse::new(
            30.0, // Speed: 30 units per second
            10.0, // Pulse width: 10 units
            [ORANGE, BRIGHT_YELLOW, RED],
            Some(Duration::from_millis(5000)), // New pulse every 2 seconds
            true,                              // Randomize position
        ))
    });

    static DUTCH_FLAG: Lazy<Animations> = Lazy::new(|| Animations::DutchFlag(DutchFlag::new()));

    static STATIC_COLOR: Lazy<Animations> =
        Lazy::new(|| Animations::Static(StaticColor::new(ORANGE)));

    static GROWING_TRAIL: Lazy<Animations> = Lazy::new(|| {
        let anim = GrowingTrail::new(ORANGE, 60.0);
        Animations::GrowingTrail(anim)
    });

    let mut queue = AnimationQueue::new(Duration::from_secs(5));

    // queue.add_animation(&*LIGHTNING_SPRINT);
    queue.add_animation(&*GROWING_TRAIL);
    queue.add_animation(&*CIRCUIT_PULSE);
    queue.add_animation(&*STATIC_COLOR);
    queue.add_animation(&*OVERDUEL_ANIM);
    queue.add_animation(&*DUTCH_FLAG);

    // receiver.receive().await;

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
