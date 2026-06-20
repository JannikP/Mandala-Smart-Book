use iced::theme::Palette;
use iced::widget::canvas::Frame;
use iced::widget::image::Handle;
use iced::{Rectangle, Renderer};

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

pub fn draw_time_and_date(_frame: &mut Frame<Renderer>, _now: chrono::DateTime<chrono::Local>) {
    // TODO: Implement
}

pub fn draw_co2(
    _frame: &mut Frame<Renderer>,
    _measurement: &anyhow::Result<scd4x::types::SensorData>,
) {
    // TODO: Implement
}

pub fn draw_temperature(
    _frame: &mut Frame<Renderer>,
    _measurement: &anyhow::Result<scd4x::types::SensorData>,
) {
    // TODO: Implement
}

pub fn draw_humidity(
    _frame: &mut Frame<Renderer>,
    _measurement: &anyhow::Result<scd4x::types::SensorData>,
) {
    // TODO: Implement
}

pub fn draw_air_quality(
    _frame: &mut Frame<Renderer>,
    _measurement: &anyhow::Result<pmsa003i::Reading>,
) {
    // TODO: Implement
}

pub fn draw_weather_forecast(_frame: &mut Frame<Renderer>) {
    // TODO: Design and implement the weather forecast system.
}
