//! Tone writer for WAV files that the app and the debug fixture play.
//!
//! The test sound of ADR 0104 Decision 4 and the debug fixture use this writer.
//! The file is 16-bit mono PCM. Each caller gives its own wave, length and level.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The wave shape of a tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Wave {
    Sine,
    Square,
}

/// The parameters of one tone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToneSpec {
    pub(crate) wave: Wave,
    pub(crate) frequency_hz: u32,
    pub(crate) seconds: u32,
    pub(crate) sample_rate: u32,
    /// The largest sample value. The sample range is `-peak` to `peak`.
    pub(crate) peak: i16,
}

/// The level of the test sound in dBFS (decibels below digital full scale).
const TEST_TONE_LEVEL_DBFS: f64 = -18.0;

/// The file name of the test sound in the cache folder of the app.
pub(crate) const TEST_TONE_FILE_NAME: &str = "test-sound.wav";

/// The test sound: 440 Hz sine, 2 s, -18 dBFS.
pub(crate) fn test_tone() -> ToneSpec {
    ToneSpec {
        wave: Wave::Sine,
        frequency_hz: 440,
        seconds: 2,
        sample_rate: 44_100,
        peak: peak_for_dbfs(TEST_TONE_LEVEL_DBFS),
    }
}

/// Gives the peak sample value for a level in dBFS.
#[expect(
    clippy::cast_possible_truncation,
    reason = "The rounded value is below i16::MAX for a level of 0 dBFS or less."
)]
fn peak_for_dbfs(level: f64) -> i16 {
    (f64::from(i16::MAX) * 10_f64.powf(level / 20.0)).round() as i16
}

/// Builds the bytes of a WAV file for the tone.
pub(crate) fn wav_bytes(spec: &ToneSpec) -> Vec<u8> {
    let sample_count = u64::from(spec.sample_rate) * u64::from(spec.seconds);
    let data_bytes = u32::try_from(sample_count * 2).expect("tone length fits a WAV file");
    let mut wav = Vec::with_capacity(44 + data_bytes as usize);
    wav.extend(b"RIFF");
    wav.extend((36 + data_bytes).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16_u32.to_le_bytes());
    wav.extend(1_u16.to_le_bytes());
    wav.extend(1_u16.to_le_bytes());
    wav.extend(spec.sample_rate.to_le_bytes());
    wav.extend((spec.sample_rate * 2).to_le_bytes());
    wav.extend(2_u16.to_le_bytes());
    wav.extend(16_u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(data_bytes.to_le_bytes());
    for index in 0..sample_count {
        wav.extend(sample(spec, index).to_le_bytes());
    }
    wav
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "The sample is at most `peak` in size, and `peak` is an i16."
)]
fn sample(spec: &ToneSpec, index: u64) -> i16 {
    let peak = f64::from(spec.peak);
    match spec.wave {
        Wave::Sine => {
            // Reduce the phase with integers, so a long tone keeps its accuracy.
            let cycle_position =
                (index * u64::from(spec.frequency_hz)) % u64::from(spec.sample_rate);
            let phase = cycle_position as f64 / f64::from(spec.sample_rate);
            (peak * (std::f64::consts::TAU * phase).sin()).round() as i16
        }
        Wave::Square => {
            let cycle_position =
                (index * u64::from(spec.frequency_hz)) % u64::from(spec.sample_rate);
            if cycle_position < u64::from(spec.sample_rate) / 2 {
                spec.peak
            } else {
                -spec.peak
            }
        }
    }
}

/// Writes the tone to a WAV file at `path`.
///
/// # Errors
///
/// Returns an error when the file cannot be written.
pub(crate) fn write_wav(path: &Path, spec: &ToneSpec) -> Result<()> {
    fs::write(path, wav_bytes(spec)).with_context(|| format!("write tone {}", path.display()))
}

/// Writes the test sound into `directory` and gives the path of the file.
///
/// # Errors
///
/// Returns an error when the folder or the file cannot be written.
pub(crate) fn write_test_tone(directory: &Path) -> Result<PathBuf> {
    fs::create_dir_all(directory)
        .with_context(|| format!("create cache folder {}", directory.display()))?;
    let path = directory.join(TEST_TONE_FILE_NAME);
    write_wav(&path, &test_tone())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples(wav: &[u8]) -> Vec<i16> {
        wav[44..]
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

    #[test]
    fn test_tone_is_two_seconds_of_440_hz_at_minus_18_dbfs() {
        let spec = test_tone();
        let wav = wav_bytes(&spec);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1, "mono");
        let rate = u32::from_le_bytes(wav[24..28].try_into().unwrap());
        assert_eq!(rate, 44_100);
        let data_bytes = u32::from_le_bytes(wav[40..44].try_into().unwrap());
        assert_eq!(data_bytes as usize, wav.len() - 44);
        let samples = samples(&wav);
        assert_eq!(samples.len(), 2 * rate as usize, "2 s of samples");

        let peak = samples.iter().map(|s| i32::from(*s).abs()).max().unwrap();
        let dbfs = 20.0 * (f64::from(peak) / 32767.0).log10();
        assert!((dbfs - -18.0).abs() < 0.05, "peak is {dbfs} dBFS");

        // 440 Hz for 2 s is 880 cycles. Each cycle has two zero crossings.
        let crossings = samples
            .windows(2)
            .filter(|pair| (pair[0] < 0) != (pair[1] < 0))
            .count();
        assert!(
            (crossings as i64 - 1760).abs() <= 2,
            "{crossings} zero crossings"
        );
    }

    #[test]
    fn square_wave_keeps_the_fixture_shape() {
        let spec = ToneSpec {
            wave: Wave::Square,
            frequency_hz: 200,
            seconds: 1,
            sample_rate: 8_000,
            peak: 300,
        };
        let samples = samples(&wav_bytes(&spec));
        for (index, value) in samples.iter().enumerate() {
            assert_eq!(*value, if index % 40 < 20 { 300 } else { -300 });
        }
    }

    #[test]
    fn write_test_tone_creates_the_file_in_the_folder() {
        let temp = tempfile::tempdir().unwrap();
        let path = write_test_tone(&temp.path().join("cache")).unwrap();
        assert_eq!(path.file_name().unwrap(), TEST_TONE_FILE_NAME);
        assert_eq!(std::fs::read(&path).unwrap(), wav_bytes(&test_tone()));
    }
}
