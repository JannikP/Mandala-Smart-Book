use std::io;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;

use blinksy::{layout2d, ControlBuilder};
use blinksy::driver::ClocklessDriver;
use blinksy::layout::{Layout2d, Shape2d, Vec2};
use blinksy::leds::Ws2812;
use blinksy::patterns::rainbow::{Rainbow, RainbowParams};
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};

layout2d!(
    Layout,
    [Shape2d::Grid {
        start: Vec2::new(-6.0, -1.0),
        horizontal_end: Vec2::new(6.0, -1.0),
        vertical_end: Vec2::new(-6.0, 1.0),
        horizontal_pixel_count: 12,
        vertical_pixel_count: 2,
        serpentine: true,
    }]
);

const TICK: u64 = 20; // [ms]

fn create_spi() -> io::Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(20_000)
         .mode(SpiModeFlags::SPI_MODE_0)
         .build();
    spi.configure(&options)?;
    Ok(spi)
}

fn main() {
    let spi = create_spi().expect("Failed to set up SPI0 for LEDs.");

    let driver = ClocklessDriver::default()
        .with_led::<Ws2812>()
        .with_writer(spi);

    let mut control = ControlBuilder::new_2d()
        .with_layout::<Layout, { Layout::PIXEL_COUNT }>()
        .with_pattern::<Rainbow>(RainbowParams {
            ..Default::default()
        })
        .with_driver(driver)
        .with_frame_buffer_size::<{ Ws2812::frame_buffer_size(Layout::PIXEL_COUNT) }>()
        .build();

    loop {
        control.tick(TICK);
        sleep(Duration::from_millis(TICK));
    }
}
