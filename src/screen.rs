//! Utilities to adapt screen brightness to the ambient brightness measured by the VEML7700.
pub fn ambient_to_screen_brightness(lux: f32) -> f32 {
    (100.0 * lux / 2000.0).round()
}
