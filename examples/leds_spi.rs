use std::{thread, time};

use smart_leds::{SmartLedsWrite, RGB8};
use linux_embedded_hal::{SPIError, SpidevBus, spidev::{SpiModeFlags, SpidevOptions}};
use ws2812_spi::hosted::Ws2812;

fn create_spi() -> Result<SpidevBus, SPIError> {
    let mut spi = SpidevBus::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(3_800_000)
         .mode(SpiModeFlags::SPI_MODE_0)
         .build();
    spi.configure(&options)?;
    Ok(spi)
}

fn main() {
    println!("Program start");

    const NUM_LEDS: usize = 24;
    const DELAY: time::Duration = time::Duration::from_millis(1000);

    let spi = create_spi().expect("Failed to open SPI device");
    let mut ws = Ws2812::new(spi);

    let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
    let empty: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    // Blink the LED's in a blue-green-red-white pattern.
    for led in data.iter_mut().step_by(4) {
        led.b = 32;
    }

    if NUM_LEDS > 1 {
        for led in data.iter_mut().skip(1).step_by(4) {
            led.g = 32;
        }
    }

    if NUM_LEDS > 2 {
        for led in data.iter_mut().skip(2).step_by(4) {
            led.r = 32;
        }
    }

    if NUM_LEDS > 3 {
        for led in data.iter_mut().skip(3).step_by(4) {
            led.r = 32;
            led.g = 32;
            led.b = 32;
        }
    }

    loop {
        // On
        println!("LEDS on");
        ws.write(data.iter().cloned()).unwrap();
        thread::sleep(DELAY);

        // Off
        println!("LEDS off");
        ws.write(empty.iter().cloned()).unwrap();
        thread::sleep(DELAY);
    }
}