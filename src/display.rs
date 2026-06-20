use chrono::Timelike;
use iced::alignment::Vertical;
use iced::theme::Palette;
use iced::widget::canvas::{Frame, Text};
use iced::widget::image::Handle;
use iced::widget::text::{Alignment, Shaping};
use iced::{Font, Point, Rectangle, Renderer};

use crate::Missing;

pub fn draw_centerpiece_photo(
    frame: &mut Frame<Renderer>,
    bounds: Rectangle,
    palette: Palette,
    centerpiece: &Handle,
    stencil: &Handle,
) {
    frame.fill_rectangle(bounds.position(), bounds.size(), palette.background);
    frame.draw_image(
        Rectangle {
            x: 0.0,
            y: 367.0,
            width: 667.0,
            height: 1186.0,
        },
        centerpiece,
    );
    frame.draw_image(
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1080.0,
            height: 1920.0,
        },
        stencil,
    );
}

pub fn draw_time_and_date(
    frame: &mut Frame<Renderer>,
    now: chrono::DateTime<chrono::Local>,
    palette: Palette,
) {
    frame.fill_text(Text {
        content: format!("{}:{}", now.hour(), now.minute()),
        position: Point::new(0.0, 136.0),
        max_width: 1080.0,
        color: palette.primary,
        size: 200.0.into(),
        line_height: 220.0.into(),
        font: Font::DEFAULT,
        align_x: Alignment::Left,
        align_y: Vertical::Top,
        shaping: Shaping::Basic,
    });
}

pub fn draw_co2(
    _frame: &mut Frame<Renderer>,
    _measurement: &Result<scd4x::types::SensorData, Missing>,
) {
    // TODO: Implement
}

pub fn draw_temperature(
    _frame: &mut Frame<Renderer>,
    _measurement: &Result<scd4x::types::SensorData, Missing>,
) {
    // TODO: Implement
}

pub fn draw_humidity(
    _frame: &mut Frame<Renderer>,
    _measurement: &Result<scd4x::types::SensorData, Missing>,
) {
    // TODO: Implement
}

pub fn draw_air_quality(
    _frame: &mut Frame<Renderer>,
    _measurement: &Result<pmsa003i::Reading, Missing>,
) {
    // TODO: Implement
}

pub fn draw_weather_forecast(_frame: &mut Frame<Renderer>) {
    // TODO: Design and implement the weather forecast system.
}
