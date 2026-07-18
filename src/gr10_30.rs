//! A platform-agnostic `no_std` Rust driver for the GR10-30 Gesture Sensor from DFRobot,
//! built on [embedded-hal](https://crates.io/crates/embedded-hal) traits.
//! Based on the [Python driver from DFRobot](https://github.com/cdjq/DFRobot_GR10_30/blob/master/python/DFRobot_GR10_30.py).

use bitmask_enum::bitmask;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use std::time::Duration;
use tokio::time;

const DEVICE_ADDRESS: u8 = 0x73;
const DEVICE_VID: u16 = 0x3343;

/// Input Register
#[repr(u8)]
#[allow(dead_code)]
pub enum InputRegister {
    /// Device PID
    PID = 0x00,
    /// VID of the device, fixed to be `0x3343`.
    VID = 0x01,
    /// Device address of the module
    Addr = 0x02,
    /// UART baud rate
    BaudRate = 0x03,
    /// UART check bit and stop bit
    StopBit = 0x04,
    /// Firmware version information
    Version = 0x05,
    /// Data ready register
    DataReady = 0x06,
    /// Gesture interrupt status
    InterruptState = 0x07,
    /// Presence status
    InterruptRegExistsState = 0x08,
}

/// Holding Register
#[repr(u8)]
#[allow(dead_code)]
pub enum HoldingRegister {
    /// The gesture that can trigger an interrupt
    InterruptMode = 0x09,
    /// Detection window
    LeftRightUpWindow = 0x0a,
    /// The distance your hand should move to the left
    LeftRange = 0x0b,
    /// The distance your hand should move to the right
    RightRange = 0x0c,
    /// The distance your hand should move up
    UpRange = 0x0d,
    /// The distance your hand should move down
    DownRange = 0x0e,
    /// The distance your hand should move forward
    ForwardRange = 0x0f,
    /// The distance your hand should move backward
    BackwardRange = 0x10,
    /// The times you need to wave hands
    WaveCount = 0x11,
    /// Hover detection window
    HoverWindow = 0x12,
    /// The duration your hand should hover
    HoverTimer = 0x13,
    /// Clockwise rotation angle, each value equals 22.5°
    CWSAngle = 0x14,
    /// Counterclockwise rotation angle, each value equals 22.5°
    CCWAngle = 0x15,
    /// Continuous clockwise rotation angle, each value equals 22.5°
    CWSAngleCount = 0x16,
    /// Continuous counterclockwise rotation angle, each value equals 22.5°
    CCWAngleCount = 0x17,
    /// Reset sensor
    Reset = 0x18,
}

#[bitmask(u16)]
pub enum Gesture {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backward,
    Clockwise,
    CounterClockwise,
    Wave,
    Hover,
    Unknown,
    ClockwiseContinuous = 0b0100_0000_0000_0000,
    CounterClockwiseContinuous = 0b1000_0000_0000_0000,
}

/// GR10-30 device driver.
#[derive(Debug)]
pub struct GR1030<I2C> {
    /// The concrete I²C device implementation.
    i2c: I2C,
}

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),

    /// Invalid VID.
    InvalidVID,
}

impl<E> From<E> for Error<E> {
    fn from(other: E) -> Self {
        Error::I2C(other)
    }
}

impl<I2C> GR1030<I2C>
where
    I2C: I2c<SevenBitAddress>,
    I2C::Error: Into<Error<I2C::Error>>,
{
    /// Create new instance of the GR10-30 device.
    pub fn new(i2c: I2C) -> Self {
        GR1030 { i2c }
    }

    pub async fn set_up(&mut self, gestures: Gesture) -> Result<(), Error<I2C::Error>> {
        self.reset()?;
        time::sleep(Duration::from_millis(500)).await;
        if !self.validate_vid()? {
            return Err(Error::InvalidVID);
        }
        self.enable_gestures(gestures)?;
        time::sleep(Duration::from_millis(500)).await;
        self.set_detection_window(20, 20)?;
        self.set_left_range(10)?;
        self.set_right_range(10)?;
        Ok(())
    }

    /// Set what gestures the module can recognize to trigger interrupts.
    pub fn enable_gestures(&mut self, gestures: Gesture) -> Result<(), Error<I2C::Error>> {
        let value_1 = ((gestures.bits() >> 8) & 0xC7) as u8;
        let value_2 = (gestures.bits() & 0x00ff) as u8;
        self.write_register(HoldingRegister::InterruptMode as u8, value_1, value_2)
    }

    /// Set the detection window
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`GR1030`) - GR10-30 sensor instance.
    /// - `width` (`u8`) - Distance from left to right with range 1-30.
    /// - `height` (`u8`) - Distance from top to bottom with range 1-30.
    ///
    /// # Returns
    ///
    /// - `Result<(), Error<I2C::Error>>` - Returns Ok(()) if the configuration was written to the sensor.
    ///
    /// # Errors
    ///
    /// - `Error<I2C::Error>` - If the I²C communication failed.
    pub fn set_detection_window(&mut self, width: u8, height: u8) -> Result<(), Error<I2C::Error>> {
        let value_1 = width & 0x1f;
        let value_2 = height & 0x1f;
        self.write_register(HoldingRegister::LeftRightUpWindow as u8, value_1, value_2)
    }

    /// Set how far your hand should move to the left so the sensor can recognize it.
    /// Distance range 5-25, must be less than distance from left to right of the detection window
    pub fn set_left_range(&mut self, range: u8) -> Result<(), Error<I2C::Error>> {
        self.write_register(HoldingRegister::LeftRange as u8, 0, range)
    }

    /// Set how far your hand should move to the right so the sensor can recognize it.
    /// Distance range 5-25, must be less than distance from right to left of the detection window
    pub fn set_right_range(&mut self, range: u8) -> Result<(), Error<I2C::Error>> {
        self.write_register(HoldingRegister::RightRange as u8, 0, range)
    }

    // TODO: Implement set_up_range

    // TODO: Implement set_down_range

    // TODO: Implement set_forward_range

    // TODO: Implement set_backwards_range

    // TODO: Implement set_wave_number

    // TODO: Implement set_hover_window

    // TODO: Implement set_hover_timer

    // TODO: Implement set_cws_angle

    // TODO: Implement set_ccw_angle

    // TODO: Implement set_cws_angle_count

    // TODO: Implement set_ccw_angle_count

    pub fn reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_register(HoldingRegister::Reset as u8, 0x55, 0x00)
    }

    /// Reads the VID.
    pub fn get_vid(&mut self) -> Result<u16, Error<I2C::Error>> {
        self.read_register_u16(InputRegister::VID as u8)
    }

    /// Checks if the VID is 0x3343.
    pub fn validate_vid(&mut self) -> Result<bool, Error<I2C::Error>> {
        self.get_vid().map(|value| value == DEVICE_VID)
    }

    /// Get if a gesture is detected. Returns `true` if a gesture is detected, `false` otherwise.
    pub fn is_data_ready(&mut self) -> Result<bool, Error<I2C::Error>> {
        self.read_register_u16(InputRegister::DataReady as u8)
            .map(|value| value == 1)
    }

    /// Destroy driver instance, return I²C bus instance.
    #[allow(dead_code)]
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Get gesture type
    pub fn gesture(&mut self) -> Result<Gesture, Error<I2C::Error>> {
        self.read_register_u16(InputRegister::InterruptState as u8)
            .map(Gesture::from)
    }

    /// Checks if data is ready and if yes, reads the detected gesture.
    pub fn check_and_get_gesture(&mut self) -> Result<Option<Gesture>, Error<I2C::Error>> {
        if self.is_data_ready()? {
            Ok(Some(self.gesture()?))
        } else {
            Ok(None)
        }
    }

    /// Get whether an object is in the sensor detection range.
    #[allow(dead_code)]
    pub fn object_in_range(&mut self) -> Result<bool, Error<I2C::Error>> {
        self.read_register_u8(InputRegister::InterruptRegExistsState as u8)
            .map(|value| value == 1)
    }

    /// Write data to a register of the GR10-30.
    fn write_register(
        &mut self,
        register: u8,
        value_1: u8,
        value_2: u8,
    ) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(DEVICE_ADDRESS, &[register, value_1, value_2])
            .map_err(Error::I2C)
    }

    /// Read two bytes from a register of the GR10-30 as u8.
    fn read_register_u8(&mut self, register: u8) -> Result<u8, Error<I2C::Error>> {
        let mut data = [0; 1];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }

    /// Read two bytes from a register of the GR10-30 as u16.
    fn read_register_u16(&mut self, register: u8) -> Result<u16, Error<I2C::Error>> {
        let mut data = [0; 2];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(u16::from(data[1]) | u16::from(data[0]) << 8))
    }
}
