mod display;
mod gr10_30;
mod light;
mod screen;
mod sensors;

use bytes::Bytes;
use iced::theme::Palette;
use iced::time::{self, milliseconds};
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Geometry};
use iced::widget::image::Handle;
use iced::widget::mouse_area;
use iced::window::Settings as WindowSettings;
use iced::{Color, Task, color, mouse};
use iced::{Element, Fill, Rectangle, Renderer, Size, Subscription, Theme};
use std::env;

use crate::display::*;
use crate::gr10_30::Gesture;
use crate::light::{LightShow, stream_light_show};
use crate::screen::{ambient_to_screen_brightness, change_screen_brightness};
use crate::sensors::stream_sensors;

const CENTERPIECE: Bytes = Bytes::from_static(include_bytes!(
    "../assets/images/centerpiece_placeholder.jpg"
));
const STENCIL: Bytes = Bytes::from_static(include_bytes!("../assets/images/stencil.png"));

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Tick(chrono::DateTime<chrono::Local>),
    SCD41Measurement(Result<scd4x::types::SensorData, Missing>),
    VEML7700Measurement(Result<f32, Missing>),
    PMSA003IMeasurement(Result<pmsa003i::Reading, Missing>),
    Gesture(Gesture),
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

    // Required to be launched from SSH and still use the one and only display.
    // If removed iced fails to launch with error message that this env war is not set.
    unsafe {
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
    }

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
    brightness: u32,
    light_show: LightShow,
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
            brightness: 0,
            light_show: LightShow::default(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Tick(now) => {
                if now != self.now {
                    self.now = now;
                    self.cache.clear();
                }
                Task::none()
            }
            Message::SCD41Measurement(measurement) => {
                self.scd4x_measurement = measurement;
                self.cache.clear();
                Task::none()
            }
            Message::VEML7700Measurement(measurement) => {
                self.veml7700_measurement = measurement.clone();
                if let Ok(value) = measurement {
                    let target_brightness = ambient_to_screen_brightness(value);
                    if target_brightness != self.brightness {
                        self.brightness = target_brightness;
                        change_screen_brightness(target_brightness)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::PMSA003IMeasurement(measurement) => {
                self.pmsa003i_measurement = measurement;
                self.cache.clear();
                Task::none()
            }
            Message::Gesture(_gesture) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        mouse_area(canvas(self as &Self).width(Fill).height(Fill))
            .interaction(mouse::Interaction::Hidden)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(milliseconds(500)).map(|_| Message::Tick(chrono::offset::Local::now())),
            Subscription::run(stream_sensors),
            Subscription::run_with(self.light_show.clone(), |d| stream_light_show(d.clone()))
                .map(ignore_error),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            "Black and White",
            Palette {
                background: Color::BLACK,
                text: color!(0xbcbcbc),
                primary: Color::WHITE,
                success: color!(0x23c00e),
                danger: color!(0xc03e0e),
                warning: color!(0xc08d0e),
            },
        )
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
            draw_co2(frame, &self.scd4x_measurement, palette);
            draw_temperature(frame, &self.scd4x_measurement, palette);
            draw_humidity(frame, &self.scd4x_measurement, palette);
            draw_air_quality(frame, &self.pmsa003i_measurement, palette);
            draw_weather_forecast(frame);
        });
        vec![clock]
    }
}

fn ignore_error<E>(item: Result<Message, E>) -> Message {
    match item {
        Ok(message) => message,
        Err(_) => Message::None,
    }
}
