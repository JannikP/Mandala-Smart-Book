use anyhow::{Context, Result, anyhow};
use iced::{Color, futures::Stream};
use iced::{color, stream};
use image::{ImageBuffer, Rgb};
use linux_embedded_hal::{
    SPIError, SpidevBus,
    spidev::{SpiModeFlags, SpidevOptions},
};
use smart_leds::{RGB8, SmartLedsWrite};
use std::time::Duration;
use tokio::time;
use ws2812_spi::hosted::Ws2812;

use crate::Message;

const NUM_LEDS: usize = 24;
const ANIMATION_INTERVAL: u64 = 40; // milliseconds = 25 frames per second
const SHOWS: usize = 5;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ShowMaster {
    shows: [LightShow; SHOWS],
    off: LightShow,
    current: usize,
    playing: bool,
}

impl ShowMaster {
    pub fn new() -> Result<Self> {
        Ok(Self {
            shows: [
                LightShow::white(),
                LightShow::rainbow()?,
                LightShow::bits()?,
                LightShow::sparks()?,
                LightShow::fire()?,
            ],
            off: LightShow::off(),
            current: 0,
            playing: false,
        })
    }

    pub fn show<'a>(&'a self) -> &'a LightShow {
        if self.playing {
            &self.shows[self.current]
        } else {
            &self.off
        }
    }

    pub fn on(&mut self) {
        self.playing = true
    }

    pub fn off(&mut self) {
        self.playing = false
    }

    pub fn next(&mut self) {
        if self.playing {
            self.current = (SHOWS - 1).min(self.current + 1)
        }
    }

    pub fn previous(&mut self) {
        if self.playing {
            if self.current > 0 {
                self.current -= 1
            }
        }
    }
}

impl Default for ShowMaster {
    fn default() -> Self {
        ShowMaster::new().expect("Failed to create one or more light shows.")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LightShow {
    data: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl LightShow {
    pub fn off() -> Self {
        Self::constant(color!(0x000000))
    }

    pub fn white() -> Self {
        Self::constant(color!(0xffffff))
    }

    pub fn bits() -> Result<Self> {
        let buffer = include_bytes!("../assets/animations/bits.png");
        Self::from_image_bytes(buffer.as_slice())
    }

    pub fn rainbow() -> Result<Self> {
        let buffer = include_bytes!("../assets/animations/rainbow.png");
        Self::from_image_bytes(buffer.as_slice())
    }

    pub fn sparks() -> Result<Self> {
        let buffer = include_bytes!("../assets/animations/sparks.png");
        Self::from_image_bytes(buffer.as_slice())
    }

    pub fn fire() -> Result<Self> {
        let buffer = include_bytes!("../assets/animations/fire.png");
        Self::from_image_bytes(buffer.as_slice())
    }

    pub fn constant(color: Color) -> Self {
        let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(1, NUM_LEDS as u32);
        for pixel in image.pixels_mut() {
            pixel[0] = (color.r * 255.0) as u8;
            pixel[1] = (color.g * 255.0) as u8;
            pixel[2] = (color.b * 255.0) as u8;
        }
        Self { data: image }
    }

    /// Read a [LightShow] from image data. The image must be exactly [NUM_LEDS] pixel in height and
    /// any number > 0 pixels wide. The animation advances by 1 pixel to the right per [ANIMATION_INTERVAL]
    /// milliseconds (40 ms per frame or 25 frames/pixels per second).
    ///
    /// One column of pixels is mapped to the LEDs directly.
    /// ```text
    ///  0: Top pixel --> Bottom rear LED
    ///   ...
    /// 11: --> Top rear LED
    /// 12: --> Top front LED
    ///   ...
    /// 24: --> Bottom front LED
    /// ```
    pub fn from_image_bytes(buffer: &[u8]) -> Result<Self> {
        let image = image::load_from_memory(buffer)
            .context("Failed to decode image from memory.")?
            .into_rgb8();

        if image.height() as usize != NUM_LEDS {
            return Err(anyhow!(
                "Invalid animation image size. Must have a height of {NUM_LEDS} pixel."
            ));
        }

        Ok(Self { data: image })
    }

    pub fn sample(&self, step: u32, buffer: &mut [RGB8; NUM_LEDS]) {
        for row in 0..NUM_LEDS {
            let pixel = self.data.get_pixel(step, row as u32);
            buffer[row].r = pixel[0];
            buffer[row].g = pixel[1];
            buffer[row].b = pixel[2];
        }
    }

    pub fn steps(&self) -> u32 {
        self.data.width()
    }
}

impl Default for LightShow {
    fn default() -> Self {
        Self::off()
    }
}

pub struct Lights {
    ws: Ws2812<SpidevBus>,
}

impl Lights {
    pub fn new() -> Result<Self, Error> {
        let mut spi = SpidevBus::open("/dev/spidev0.0")?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(3_800_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;

        Ok(Lights {
            ws: Ws2812::new(spi),
        })
    }

    pub fn show(&mut self, data: &[RGB8; NUM_LEDS]) -> Result<(), Error> {
        self.ws.write(data.iter().cloned())?;
        Ok(())
    }
}

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error {
    /// SPI bus error
    SPI(SPIError),

    /// Generic IO error
    IO(std::io::Error),
}

impl From<SPIError> for Error {
    fn from(other: SPIError) -> Self {
        Error::SPI(other)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::IO(other)
    }
}

pub fn stream_light_show(show: LightShow) -> impl Stream<Item = Result<Message, Error>> {
    stream::try_channel(4, async move |_| {
        let mut lights = Lights::new()?;
        let mut data = [RGB8::default(); NUM_LEDS];
        let mut interval = time::interval(Duration::from_millis(ANIMATION_INTERVAL));
        loop {
            for step in 0..show.steps() {
                show.sample(step, &mut data);
                lights.show(&data)?;
                interval.tick().await;
            }
        }
    })
}
