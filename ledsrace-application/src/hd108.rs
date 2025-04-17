use embedded_hal_async::spi::SpiBus;

pub const fn required_buffer_size<const N: usize>() -> usize {
    const PREAMBLE_ZERO_BYTES: usize = 16;
    // We need 8 bytes per LED and then 1 bit per LED
    PREAMBLE_ZERO_BYTES + (N * 8) + (N / 8)
}

pub struct HD108<SPI, const N: usize> {
    spi: SPI,
    buf: &'static mut [u8],
}

impl<SPI, const N: usize> HD108<SPI, N>
where
    SPI: SpiBus<u8>,
{
    pub fn new(spi: SPI, buf: &'static mut [u8]) -> Self {
        if buf.len() != required_buffer_size::<N>() {
            panic!("Buffer size is not correct");
        }
        Self { spi, buf }
    }

    // Function to create an LED frame
    fn create_led_frame(red: u16, green: u16, blue: u16) -> [u8; 8] {
        let start_code: u8 = 0b1;
        let red_gain: u8 = 0b00010; // Regulation level 2 - 2.24 mA
        let green_gain: u8 = 0b00010; // Regulation level 2 - 2.24 mA
        let blue_gain: u8 = 0b00010; // Regulation level 2 - 2.24 mA

        // Combine the gain values into a 15-bit number
        let current_gain =
            ((red_gain as u16) << 10) | ((green_gain as u16) << 5) | (blue_gain as u16);

        // The first byte contains the start code and the 7 most significant bits of the current gain
        let first_byte = (start_code << 7) | ((current_gain >> 8) as u8 & 0x7F);

        // The second byte contains the remaining 8 bits of the current gain
        let second_byte = (current_gain & 0xFF) as u8;

        [
            first_byte,           // Start code and part of current gain
            second_byte,          // Remaining current gain bits
            (red >> 8) as u8,     // High byte of red
            (red & 0xFF) as u8,   // Low byte of red
            (green >> 8) as u8,   // High byte of green
            (green & 0xFF) as u8, // Low byte of green
            (blue >> 8) as u8,    // High byte of blue
            (blue & 0xFF) as u8,  // Low byte of blue
        ]
    }

    pub async fn set_off(&mut self) -> Result<(), SPI::Error> {
        // Set all LEDs to off
        self.buf.fill(0);

        // Write the data to the SPI bus
        self.spi.write(self.buf).await?;

        Ok(())
    }

    pub async fn set_leds(&mut self, leds: &[(usize, u8, u8, u8)]) -> Result<(), SPI::Error> {
        self.buf.fill(0);

        // At least 128 bits of zeros for the start frame. This is 16 bytes, so we start at index 16.
        let mut index = 16;

        // Set the specified LEDs to the given colors and all others to off
        for i in 0..N {
            let led_frame = if let Some(&(_led_num, red, green, blue)) =
                leds.iter().find(|&&(led_num, _, _, _)| led_num == i)
            {
                // Convert the 8-bit RGB values to 16-bit values
                let red = ((red as u16) << 8) | (red as u16);
                let green = ((green as u16) << 8) | (green as u16);
                let blue = ((blue as u16) << 8) | (blue as u16);

                Self::create_led_frame(red, green, blue)
            } else {
                // LED off
                Self::create_led_frame(0x0000, 0x0000, 0x0000)
            };
            self.buf[index..index + 8].copy_from_slice(&led_frame);
            index += 8;
        }

        // After the last frame we required additional clock pulses equal to the number of LEDs in the strip.
        // Since we already zeroed the buffer we don't actually have to insert them here. Just supply the full buffer to the SPI bus.

        self.spi.write(self.buf).await?;

        Ok(())
    }
}
