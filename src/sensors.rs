use iced::futures::Stream;
use iced::futures::sink::SinkExt;
use iced::stream;
use linux_embedded_hal::{Delay, I2cdev};
use std::time::Duration;
use tokio::time;

use scd4x::Scd4x;

use crate::Message;

pub fn stream_sensors() -> impl Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let dev = I2cdev::new("/dev/i2c-1").expect("Could not open I2C device.");
        let mut sensor = Scd4x::new(dev, Delay);

        sensor.wake_up();
        sensor
            .stop_periodic_measurement()
            .expect("Could not stop periodic measurements.");
        sensor.reinit().expect("Failed to reinitialize sensor.");

        let serial = sensor
            .serial_number()
            .expect("Could not get serial number.");
        println!("serial: {serial:#04x}");

        sensor
            .start_periodic_measurement()
            .expect("Could not start periodic measurements.");
        println!("Waiting for first measurement... (5 sec)");
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let ready = sensor
                .data_ready_status()
                .expect("Could not get measurement ready status.");
            if !ready {
                continue;
            }

            let data = sensor.measurement().expect("Could not get sensor reading.");

            println!(
                "CO2: {0}, Temperature: {1:#.2} \u{00b0}C, Humidity: {2:#.2} RH",
                data.co2, data.temperature, data.humidity
            );

            output
                .send(Message::SCD41Measurement(Ok(data)))
                .await
                .expect("Failed to send message.");
        }
    })
}
