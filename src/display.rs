use chrono::{Datelike, Timelike};
use iced::theme::Palette;
use iced::widget::canvas::{Frame, Text};
use iced::widget::image::Handle;
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
    let weekday = match now.weekday() {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    };
    let day = now.day();
    let month = match now.month() {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };
    frame.fill_text(Text {
        content: format!("{weekday}, {day:02}. {month}"),
        position: Point::new(20.0, 117.0),
        max_width: 1080.0,
        color: palette.text,
        size: 48.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
    frame.fill_text(Text {
        content: format!("{:02}:{:02}", now.hour(), now.minute()),
        position: Point::new(0.0, 136.0),
        max_width: 1080.0,
        color: palette.primary,
        size: 200.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
}

pub fn draw_co2(
    frame: &mut Frame<Renderer>,
    measurement: &Result<scd4x::types::SensorData, Missing>,
    palette: Palette,
) {
    let text = match measurement {
        Ok(data) => {
            format!("{}", data.co2)
        }
        Err(Missing::NotMeasured) => "--".into(),
        Err(_) => "Err".into(),
    };
    frame.fill_text(Text {
        content: text,
        position: Point::new(500.0, 488.0),
        max_width: 400.0,
        color: palette.primary,
        size: 105.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
}

pub fn draw_temperature(
    frame: &mut Frame<Renderer>,
    measurement: &Result<scd4x::types::SensorData, Missing>,
    palette: Palette,
) {
    let text = match measurement {
        Ok(data) => {
            format!("{:.0}", data.temperature)
        }
        Err(Missing::NotMeasured) => "--".into(),
        Err(_) => "Err".into(),
    };
    frame.fill_text(Text {
        content: text,
        position: Point::new(666.0, 892.0),
        max_width: 300.0,
        color: palette.primary,
        size: 120.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
}

pub fn draw_humidity(
    _frame: &mut Frame<Renderer>,
    _measurement: &Result<scd4x::types::SensorData, Missing>,
    _palette: Palette,
) {
    // TODO: Implement
}

pub fn draw_air_quality(
    frame: &mut Frame<Renderer>,
    measurement: &Result<pmsa003i::Reading, Missing>,
    palette: Palette,
) {
    let (aqi_value, aqi_subtext) = match measurement {
        Ok(reading) => match reading.aqi_pm10 {
            Ok(value) => (
                format!("{:}", value.aqi()),
                match value.level() {
                    pmsa003i::AirQualityLevel::Good => "Good",
                    pmsa003i::AirQualityLevel::Moderate => "Moderate",
                    pmsa003i::AirQualityLevel::UnhealthySensitive
                    | pmsa003i::AirQualityLevel::Unhealthy => "Unhealthy",
                    pmsa003i::AirQualityLevel::VeryUnhealthy => "Very Unhealthy",
                    pmsa003i::AirQualityLevel::Hazardous => "Hazardous",
                }
                .to_string(),
            ),
            Err(reason) => (
                "N/A".into(),
                match reason {
                    pmsa003i::AirQualityError::OutOfRange => "Out of Range",
                }
                .into(),
            ),
        },
        Err(Missing::NotMeasured) => ("--".into(), "Pending...".into()),
        Err(_) => ("N/A".into(), "Error".into()),
    };
    frame.fill_text(Text {
        content: aqi_value,
        position: Point::new(565.0, 1275.0),
        max_width: 300.0,
        color: palette.primary,
        size: 120.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
    frame.fill_text(Text {
        content: aqi_subtext,
        position: Point::new(535.0, 1475.0),
        max_width: 300.0,
        color: palette.text,
        size: 35.0.into(),
        font: Font::DEFAULT,
        ..Default::default()
    });
}

pub fn draw_weather_forecast(_frame: &mut Frame<Renderer>) {
    // TODO: Design and implement the weather forecast system.
}
