mod display;

use anyhow::anyhow;
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

#[derive(Debug)]
pub enum Message {
    Tick(chrono::DateTime<chrono::Local>),
    SCD41Measurement(anyhow::Result<scd4x::types::SensorData>),
    VEML7700Measurement(anyhow::Result<f32>),
    PMSA003IMeasurement(anyhow::Result<pmsa003i::Reading>),
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
    scd4x_measurement: anyhow::Result<scd4x::types::SensorData>,
    veml7700_measurement: anyhow::Result<f32>,
    pmsa003i_measurement: anyhow::Result<pmsa003i::Reading>,
    cache: Cache,
    centerpiece: Handle,
    stencil: Handle,
}

impl Clock {
    fn new() -> Self {
        Self {
            now: chrono::offset::Local::now(),
            scd4x_measurement: anyhow::Result::Err(anyhow!("Not measured")),
            veml7700_measurement: anyhow::Result::Err(anyhow!("Not measured")),
            pmsa003i_measurement: anyhow::Result::Err(anyhow!("Not measured")),
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
            let mut center = frame.center();
            center.x = 0.0;
            let radius = center.y;

            draw_centerpiece_photo(frame, bounds, palette, &self.centerpiece, &self.stencil);
            draw_time_and_date(frame, self.now);
            draw_co2(frame, &self.scd4x_measurement);
            draw_temperature(frame, &self.scd4x_measurement);
            draw_humidity(frame, &self.scd4x_measurement);
            draw_air_quality(frame, &self.pmsa003i_measurement);
            draw_weather_forecast(frame);
        });
        vec![clock]
    }
}
