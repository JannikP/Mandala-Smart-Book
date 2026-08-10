//! Utilities to adapt screen brightness to the ambient brightness measured by the VEML7700.
use brightness::Brightness;
use futures::TryStreamExt;
use iced::Task;

use crate::Message;

pub fn ambient_to_screen_brightness(lux: f32) -> u32 {
    let percentage = (100.0 * lux / 2000.0).round();
    percentage as u32
}

pub fn change_screen_brightness(percentage: u32) -> Task<Message> {
    Task::future(apply_brightness(percentage))
}

async fn apply_brightness(percentage: u32) -> Message {
    brightness::brightness_devices()
        .try_for_each(|mut dev| async move {
            dev.set(percentage).await?;
            Ok(())
        })
        .await
        .expect("Failed to change screen brightness.");
    Message::None
}
