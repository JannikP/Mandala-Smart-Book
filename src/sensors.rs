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

use crate::gr10_30::{GR1030, Gesture};
use crate::{Message, Missing};

const ENVIRONMENTAL_SENSOR_INTERVAL: u64 = 5; // seconds
const VEML7700_INTERVAL: u64 = 1000; // milliseconds
const GR1030_INTERVAL: u64 = 500; // milliseconds

pub fn stream_sensors() -> impl Stream<Item = Message> {
    stream::channel(100, async |output| {
        let dev = Mutex::new(I2cdev::new("/dev/i2c-1").expect("Could not open I2C device."));
        tokio::join! {
            stream_gr10_30(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
            stream_pmsa003i(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
            stream_scd41(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
            stream_veml7700(output.clone(), embedded_hal_bus::i2c::MutexDevice::new(&dev)),
        };
    })
}

async fn stream_gr10_30<I2C, E>(mut output: mpsc::Sender<Message>, dev: I2C)
where
    I2C: I2c<Error = E>,
    E: std::fmt::Debug,
{
    let mut sensor = GR1030::new(dev);
    loop {
        let outcome = sensor
            .set_up(Gesture::Left | Gesture::Right | Gesture::Up | Gesture::Down)
            .await
            .err();
        if let Some(error) = outcome {
            println!("Failed to initialize GR10-30, because of {error:?}.");
            time::sleep(Duration::from_secs(2)).await;
            continue;
        }

        let mut interval = time::interval(Duration::from_millis(GR1030_INTERVAL));
        loop {
            match sensor.check_and_get_gesture() {
                Ok(Some(gestures)) => {
                    output
                        .send(Message::Gesture(gestures))
                        .await
                        .expect("Failed to send message.");
                }
                Ok(None) => {
                    // println!("No data ready.");
                }
                Err(error) => {
                    println!("Failed to read gesture from GR10-30: {:?}", error);
                    // TODO: report error properly
                    break;
                }
            }

            interval.tick().await;
        }
    }
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

    let _serial = sensor
        .serial_number()
        .expect("Could not get serial number.");

    sensor
        .start_periodic_measurement()
        .expect("Could not start periodic measurements.");
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
