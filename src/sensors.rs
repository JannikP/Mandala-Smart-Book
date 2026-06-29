use embedded_hal::i2c::I2c;
use futures::channel::mpsc;
use iced::futures::Stream;
use iced::futures::sink::SinkExt;
use iced::stream;
use linux_embedded_hal::{Delay, I2cdev};
use pmsa003i::Pmsa003i;
use scd4x::Scd4x;
use std::{sync::Mutex, time::Duration};
use tokio::time;
use veml7700::Veml7700;

use crate::{Message, Missing};

const ENVIRONMENTAL_SENSOR_INTERVAL: u64 = 5; // seconds
const VEML7700_INTERVAL: u64 = 1000; // milliseconds

pub fn stream_sensors() -> impl Stream<Item = Message> {
    stream::channel(100, async |output| {
        let dev = Mutex::new(I2cdev::new("/dev/i2c-1").expect("Could not open I2C device."));
        tokio::join! {
            stream_pmsa003i(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
            stream_scd41(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
            stream_veml7700(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
        };
    })
}

async fn stream_pmsa003i<I2C, E>(mut output: mpsc::Sender<Message>, dev: I2C)
where
    I2C: I2c<Error = E>,
    E: std::fmt::Debug,
{
    let mut sensor = Pmsa003i::new(dev);

    let mut interval = time::interval(Duration::from_secs(ENVIRONMENTAL_SENSOR_INTERVAL));
    loop {
        let result = sensor
            .read()
            .map_err(|e| Missing::HardwareFault(format!("Failed to read: {:?}", e)));
        output
            .send(Message::PMSA003IMeasurement(result))
            .await
            .expect("Failed to send message.");
        interval.tick().await;
    }
}

async fn stream_scd41<I2C, E>(mut output: mpsc::Sender<Message>, dev: I2C)
where
    I2C: I2c<Error = E>,
    E: std::fmt::Debug,
{
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
    let mut interval = time::interval(Duration::from_secs(ENVIRONMENTAL_SENSOR_INTERVAL));
    loop {
        interval.tick().await;

        let ready = sensor
            .data_ready_status()
            .expect("Could not get measurement ready status.");
        if !ready {
            continue;
        }

        let result = sensor
            .measurement()
            .map_err(|e| Missing::HardwareFault(format!("Failed to read: {:?}", e)));

        output
            .send(Message::SCD41Measurement(result))
            .await
            .expect("Failed to send message.");
    }
}

async fn stream_veml7700<I2C, E>(mut output: mpsc::Sender<Message>, dev: I2C)
where
    I2C: I2c<Error = E>,
    E: std::fmt::Debug,
{
    let mut sensor = Veml7700::new(dev);

    sensor.enable().expect("Failed to enable the VEML7700.");
    time::sleep(Duration::from_millis(4)).await;

    let mut interval = time::interval(Duration::from_millis(VEML7700_INTERVAL));
    loop {
        let result = sensor
            .read_lux()
            .map_err(|e| Missing::HardwareFault(format!("Failed to read: {:?}", e)));
        output
            .send(Message::VEML7700Measurement(result))
            .await
            .expect("Failed to send message.");
        interval.tick().await;
    }
}
