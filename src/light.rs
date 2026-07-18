use iced::futures::Stream;
use iced::stream;
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightShow {
    /// All lights off, no show.
    #[default]
    Off,

    /// All LEDs same plain white color.
    PlainWhite,
}

impl LightShow {
    pub fn is_some(self) -> bool {
        self != LightShow::Off
    }

    pub fn steps(self) -> usize {
        1
    }

    pub fn sample(self, step: usize, data: &mut [RGB8; NUM_LEDS]) {
        assert!(step < self.steps());
        match self {
            LightShow::Off => {
                for led in data.iter_mut() {
                    led.r = 0;
                    led.g = 0;
                    led.b = 0;
                }
            }
            LightShow::PlainWhite => {
                for led in data.iter_mut() {
                    led.r = 255;
                    led.g = 255;
                    led.b = 255;
                }
            }
        }
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
