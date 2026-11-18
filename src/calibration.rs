//! Host-side calibration for the 112 x 112 Metallica sensor.
//!
//! The averaging and correction algorithm follows python-validity's sensor.py
//! (https://github.com/uunicorn/python-validity), Copyright (c) 2020 uunicorn.
//! See LICENSE.python-validity. This module never writes sensor flash; in
//! particular, it does not replace the separate on-chip clean-slate image.

use crate::{CaptureMode, DriverError, Result, SensorSession, SynapticsUsbTransport};

const ROW_BYTES: usize = 120; // 8 header bytes followed by 112 pixels
const ROWS: usize = 112;
pub const CALIBRATION_BYTES: usize = ROW_BYTES * ROWS;
const FRAME_BYTES: usize = CALIBRATION_BYTES * 2; // two interleaved lines per row

/// Discard the first of three frames, then average the remaining four samples
/// for each byte (two frames, each with two interleaved lines).
fn average_frames(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() < 3 * FRAME_BYTES {
        return Err(DriverError::ShortResponse {
            context: "calibration frames on EP 0x82",
            expected: 3 * FRAME_BYTES,
            actual: raw.len(),
        });
    }
    let mut averaged = vec![0; CALIBRATION_BYTES];
    for row in 0..ROWS {
        for col in 0..ROW_BYTES {
            let mut total = 0u16;
            for frame in 1..3 {
                for line in 0..2 {
                    total += raw[frame * FRAME_BYTES + (2 * row + line) * ROW_BYTES + col] as u16;
                }
            }
            averaged[row * ROW_BYTES + col] = (total / 4) as u8;
        }
    }
    Ok(averaged)
}

/// Scale pixel deviations from 128 into signed corrections, combining later
/// iterations with the previous correction using saturating signed addition.
fn update_correction(raw: &[u8], previous: &[u8]) -> Result<Vec<u8>> {
    if !previous.is_empty() && previous.len() != CALIBRATION_BYTES {
        return Err(DriverError::Protocol(
            "Invalid previous host calibration length".into(),
        ));
    }
    let mut next = average_frames(raw)?;
    for row in 0..ROWS {
        let start = row * ROW_BYTES;
        if !previous.is_empty() {
            next[start..start + 8].copy_from_slice(&previous[start..start + 8]);
        }
        for col in 8..ROW_BYTES {
            let i = start + col;
            let correction = ((next[i] as i32 - 128) * 10 / 34).clamp(-128, 127);
            let old = if previous.is_empty() {
                0
            } else {
                previous[i] as i8 as i32
            };
            next[i] = (old + correction).clamp(-128, 127) as i8 as u8;
        }
    }
    Ok(next)
}

impl SensorSession<SynapticsUsbTransport> {
    /// Capture three calibration iterations with the sensor clean and uncovered.
    /// Returns a host correction table; does not write flash or biometric records.
    pub fn calibrate_host(&mut self) -> Result<Vec<u8>> {
        let factory = self.get_factory_calibration_values()?;
        if factory.len() <= 0x38 {
            return Err(DriverError::Protocol(
                "Factory calibration is missing its reference line".into(),
            ));
        }
        let mut correction = Vec::new();
        for iteration in 1..=3 {
            crate::drv_info!("Host calibration iteration {iteration}/3: keep sensor uncovered");
            let command = crate::build_cmd_02_0199(CaptureMode::Calibrate, &factory, &correction);
            let capture = (|| {
                self.execute_cmd(&command, 2)?;
                self.transport.read_calibration_frames()
            })();
            // Always stop acquisition, including when a bulk read fails.
            let _ = self.execute_raw(&[0x04], 2);
            correction = update_correction(&capture?, &correction)?;
        }
        Ok(correction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_interleaved_lines_and_discards_first_frame() {
        let mut raw = vec![255; 3 * FRAME_BYTES + ROW_BYTES];
        for (frame, values) in [(1, [10, 20]), (2, [30, 40])] {
            for row in 0..ROWS {
                for (line, value) in values.into_iter().enumerate() {
                    let start = frame * FRAME_BYTES + (2 * row + line) * ROW_BYTES;
                    raw[start..start + ROW_BYTES].fill(value);
                }
            }
        }
        assert_eq!(average_frames(&raw).unwrap(), vec![25; CALIBRATION_BYTES]);
        assert!(average_frames(&raw[..3 * FRAME_BYTES - 1]).is_err());
    }

    #[test]
    fn correction_preserves_headers_and_saturates_signed_pixels() {
        let mut raw = vec![128; 3 * FRAME_BYTES];
        for frame in 1..3 {
            for line in 0..2 {
                let start = frame * FRAME_BYTES + line * ROW_BYTES;
                raw[start..start + 8].fill(42);
                raw[start + 8] = 255;
                raw[start + 9] = 0;
            }
        }
        let first = update_correction(&raw, &[]).unwrap();
        assert_eq!(&first[..8], &[42; 8]);
        assert_eq!(first[8] as i8, 37);
        assert_eq!(first[9] as i8, -37);
        let mut previous = vec![0; CALIBRATION_BYTES];
        previous[..8].fill(12);
        previous[8] = 120;
        previous[9] = (-120i8) as u8;
        let next = update_correction(&raw, &previous).unwrap();
        assert_eq!(&next[..8], &[12; 8]);
        assert_eq!(next[8] as i8, 127);
        assert_eq!(next[9] as i8, -128);
        assert!(update_correction(&raw, &[0; 10]).is_err());
    }
}
