//! Audio detection (ADR 0004) and converter verification/fallback (ADR 0066).

pub(crate) mod probe;

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// The audio format type comes from the shared crate (musicindex ADR 0093
/// R-TAG-1). This module keeps the conversion and the converter probe.
pub use musicindex_tags::format::AudioFormat;

/// Resolve the `flac` binary. `override_path` wins when provided; otherwise we
/// fall back to `flac` on `$PATH`.
fn flac_binary(override_path: Option<&Path>) -> std::ffi::OsString {
    override_path
        .map(|p| p.as_os_str().to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("flac"))
}

/// Freshly checks the selected FLAC executable with bounded process execution.
pub fn flac_cli_available(override_path: Option<&Path>) -> bool {
    probe::ConverterProbe::flac(override_path).available()
}

/// Re-encode a WAV file to FLAC in place. Tries the `flac` CLI first (reference
/// encoder, preserves bit depth when compatible). Falls back to `ffmpeg` on
/// `$PATH` when `flac` rejects the input — notably 32-bit float WAV, which
/// the `flac` CLI does not accept but `ffmpeg` can downmix to s16. Removes the
/// original WAV on success. Returns the new path on success.
pub fn transcode_wav_to_flac(
    wav_path: &Path,
    binary_override: Option<&Path>,
) -> Result<std::path::PathBuf> {
    transcode_observed(
        wav_path,
        &probe::ConverterObservation::refresh(binary_override),
    )
}

/// Actual encoder outcome, separate from version-check availability (ADR 0066).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConversionOutcome {
    #[default]
    NotRequired,
    Flac,
    FfmpegFallback,
    WavRetained,
}

/// Converts an owned staging input without deleting it (ADR 0066).
pub(crate) fn convert_retaining_input(
    wav_path: &Path,
    binary_override: Option<&Path>,
) -> Result<(std::path::PathBuf, ConversionOutcome)> {
    encode_observed(
        wav_path,
        &probe::ConverterObservation::refresh(binary_override),
    )
}

fn transcode_observed(
    wav_path: &Path,
    observation: &probe::ConverterObservation,
) -> Result<std::path::PathBuf> {
    let (path, _) = encode_observed(wav_path, observation)?;
    std::fs::remove_file(wav_path)
        .with_context(|| format!("remove converted WAV input {}", wav_path.display()))?;
    Ok(path)
}

fn encode_observed(
    wav_path: &Path,
    observation: &probe::ConverterObservation,
) -> Result<(std::path::PathBuf, ConversionOutcome)> {
    let flac_path = wav_path.with_extension("flac");
    anyhow::ensure!(
        !flac_path.exists(),
        "conversion output already exists: {}",
        flac_path.display()
    );

    let flac_err = if observation.flac.available() {
        match run_flac_encode(wav_path, &flac_path, Some(&observation.flac.executable)) {
            Ok(()) => {
                return Ok((flac_path, ConversionOutcome::Flac));
            }
            Err(err) => {
                remove_failed_output(&flac_path)?;
                Some(err)
            }
        }
    } else {
        None
    };

    if observation.ffmpeg.available() {
        match run_ffmpeg_encode(wav_path, &flac_path, &observation.ffmpeg.executable) {
            Ok(()) => {
                return Ok((flac_path, ConversionOutcome::FfmpegFallback));
            }
            Err(ffmpeg_err) => {
                remove_failed_output(&flac_path)?;
                match flac_err {
                Some(flac_err) => anyhow::bail!(
                    "flac encode failed ({flac_err:#}); ffmpeg fallback also failed ({ffmpeg_err:#})"
                ),
                None => return Err(ffmpeg_err),
                }
            }
        }
    }

    match flac_err {
        Some(err) => Err(err),
        None => anyhow::bail!(
            "No converter passed its version check: FLAC {} ({:?}, {:?}); ffmpeg {} ({:?}). WAV input retained.",
            observation.flac.executable.display(), observation.flac.source, observation.flac.outcome,
            observation.ffmpeg.executable.display(), observation.ffmpeg.outcome
        ),
    }
}

fn remove_failed_output(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("remove failed conversion output {}", path.display())),
    }
}

fn run_flac_encode(
    wav_path: &Path,
    flac_path: &Path,
    binary_override: Option<&Path>,
) -> Result<()> {
    let binary = flac_binary(binary_override);
    let output = Command::new(&binary)
        .arg("--best")
        .arg("--silent")
        .arg("--totally-silent")
        .arg("-f")
        .arg("-o")
        .arg(flac_path)
        .arg(wav_path)
        .output()
        .with_context(|| format!("invoke flac for {}", wav_path.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "FLAC encoding failed (exit {:?}) for {}",
            output.status.code(),
            wav_path.display()
        );
    }
    anyhow::ensure!(
        AudioFormat::detect_from_file(flac_path)? == AudioFormat::Flac,
        "FLAC encoder did not produce a FLAC file: {}",
        flac_path.display()
    );
    Ok(())
}

fn run_ffmpeg_encode(wav_path: &Path, flac_path: &Path, binary: &Path) -> Result<()> {
    let output = Command::new(binary)
        .arg("-y")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(wav_path)
        .arg("-sample_fmt")
        .arg("s16")
        .arg("-compression_level")
        .arg("12")
        .arg(flac_path)
        .output()
        .with_context(|| format!("invoke ffmpeg for {}", wav_path.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg encoding failed (exit {:?}) for {}",
            output.status.code(),
            wav_path.display()
        );
    }
    anyhow::ensure!(
        AudioFormat::detect_from_file(flac_path)? == AudioFormat::Flac,
        "ffmpeg did not produce a FLAC file: {}",
        flac_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_0066_conversion_actual_fallback_cleans_partial_output_and_keeps_input() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let wav = temp.path().join("input.wav");
        let output = wav.with_extension("flac");
        fs::write(&wav, b"RIFFfixtureWAVE").unwrap();
        let flac = temp.path().join("flac");
        let ffmpeg = temp.path().join("ffmpeg");
        fs::write(&flac, "#!/bin/sh\n[ \"$1\" = --version ] && exit 0\nwhile [ \"$1\" != -o ]; do shift; done\nshift\nprintf partial > \"$1\"\nexit 7\n").unwrap();
        fs::write(&ffmpeg, "#!/bin/sh\n[ \"$1\" = --version ] && exit 0\nfor last do :; done\n[ -e \"$last\" ] && exit 99\nprintf 'fLaC-fallback' > \"$last\"\n").unwrap();
        for binary in [&flac, &ffmpeg] {
            fs::set_permissions(binary, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let observation = probe::ConverterObservation {
            flac: probe::ConverterProbe::flac(Some(&flac)),
            ffmpeg: probe::ConverterProbe::flac(Some(&ffmpeg)),
        };
        let (path, outcome) = encode_observed(&wav, &observation).unwrap();
        assert_eq!(path, output);
        assert_eq!(outcome, ConversionOutcome::FfmpegFallback);
        assert!(wav.exists());
        fs::remove_file(&output).unwrap();
        fs::write(&ffmpeg, "#!/bin/sh\nfor last do :; done\nprintf partial > \"$last\"\nprintf 'fixture secret' >&2\nexit 8\n").unwrap();
        let error = encode_observed(&wav, &observation).unwrap_err().to_string();
        assert!(!error.contains("fixture secret"));
        assert!(!output.exists());
        assert!(wav.exists());
    }
}
