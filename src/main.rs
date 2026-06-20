mod display;

use bytes::Bytes;
use iced::mouse;
use iced::time::{self, milliseconds};
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Geometry};
use iced::widget::image::Handle;
use iced::widget::mouse_area;
use iced::window::Settings as WindowSettings;
use iced::{Element, Fill, Rectangle, Renderer, Size, Subscription, Theme};

use crate::display::*;

const CENTERPIECE: Bytes = Bytes::from_static(include_bytes!(
    "../assets/images/centerpiece_placeholder.jpg"
));
const STENCIL: Bytes = Bytes::from_static(include_bytes!("../assets/images/stencil.png"));

#[derive(Debug, Clone)]
pub enum Message {
    Tick(chrono::DateTime<chrono::Local>),
    SCD41Measurement(Result<scd4x::types::SensorData, Missing>),
    VEML7700Measurement(Result<f32, Missing>),
    PMSA003IMeasurement(Result<pmsa003i::Reading, Missing>),
}

#[derive(Debug, Clone)]
pub enum Missing {
    NotMeasured,
    HardwareFault(String),
    Timeout,
    Other(String),
}

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Clock::new, Clock::update, Clock::view)
        .window(WindowSettings {
            fullscreen: true,
            size: Size::new(1080.0, 1920.0),
            ..Default::default()
        })
        .subscription(Clock::subscription)
        .theme(Clock::theme)
        .run()
}

#[derive(Debug)]
struct Clock {
    now: chrono::DateTime<chrono::Local>,
    scd4x_measurement: Result<scd4x::types::SensorData, Missing>,
    veml7700_measurement: Result<f32, Missing>,
    pmsa003i_measurement: Result<pmsa003i::Reading, Missing>,
    cache: Cache,
    centerpiece: Handle,
    stencil: Handle,
}

impl Clock {
    fn new() -> Self {
        Self {
            now: chrono::offset::Local::now(),
            scd4x_measurement: Result::Err(Missing::NotMeasured),
            veml7700_measurement: Result::Err(Missing::NotMeasured),
            pmsa003i_measurement: Result::Err(Missing::NotMeasured),
            cache: Cache::default(),
            centerpiece: Handle::from_bytes(CENTERPIECE),
            stencil: Handle::from_bytes(STENCIL),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if now != self.now {
                    self.now = now;
                    self.cache.clear();
                }
            }
            Message::SCD41Measurement(measurement) => {
                self.scd4x_measurement = measurement;
                self.cache.clear();
            }
            Message::VEML7700Measurement(measurement) => {
                self.veml7700_measurement = measurement;
                self.cache.clear();
            }
            Message::PMSA003IMeasurement(measurement) => {
                self.pmsa003i_measurement = measurement;
                self.cache.clear();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        mouse_area(canvas(self as &Self).width(Fill).height(Fill))
            .interaction(mouse::Interaction::Hidden)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(500)).map(|_| Message::Tick(chrono::offset::Local::now()))
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl<Message> canvas::Program<Message> for Clock {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let clock = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.palette();
            draw_centerpiece_photo(frame, bounds, palette, &self.centerpiece, &self.stencil);
            draw_time_and_date(frame, self.now, palette);
            draw_co2(frame, &self.scd4x_measurement);
            draw_temperature(frame, &self.scd4x_measurement);
            draw_humidity(frame, &self.scd4x_measurement);
            draw_air_quality(frame, &self.pmsa003i_measurement);
            draw_weather_forecast(frame);
        });
        vec![clock]
    }
}
