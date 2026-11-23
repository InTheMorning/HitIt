//! The HitIt side of the `musicindex-tags` crate (musicindex ADR 0093
//! R-TAG-1).
//!
//! The crate owns the frame code: the read, the write, the frame names of
//! R-TAG-2 and the removal of the old `MusicIndex ...` frames. HitIt owns the
//! network, so this module gives the crate the source of the pictures and the
//! transcripts that an edit names (ADR 0056). [`write_id3v24_edits`] writes
//! with that source.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

pub use musicindex_tags::frames;
pub use musicindex_tags::{
    id3v24_edit_label_is_writable, involved_people_value, read_audio_tags, read_podcast_tags,
    sanitize_title_text, split_license_list, vorbis_storage_key, AudioTags, BadInvolvedPerson,
    BadTag, Id3Field, Id3v24Edit, ReferenceSource, TagWriteResult, LICENSE_LIST_FRAME,
    LICENSE_NAME_FRAME, LICENSE_URL_FRAME,
};

use crate::media::image_type;
use crate::remote_media;

/// Each old frame label of musicindex ADR 0093 R-TAG-2 and its new label.
pub(crate) const RENAMED_FRAMES: [(&str, &str); 10] = [
    ("TXXX:MusicIndex Feed Guid", frames::FEED_GUID),
    ("TXXX:MusicIndex Track Guid", frames::ITEM_GUID),
    ("TXXX:MusicIndex Value Routes", frames::VALUE_ROUTES),
    ("TXXX:MusicIndex Image", frames::IMAGE),
    ("TXXX:MusicIndex Contributors", frames::CONTRIBUTORS),
    ("TXXX:MusicIndex License", frames::LICENSE),
    ("COMM:MusicIndex Description", frames::DESCRIPTION),
    (
        "COMM:MusicIndex Album Description",
        frames::FEED_DESCRIPTION,
    ),
    ("USLT:MusicIndex Transcript", frames::TRANSCRIPT_TEXT),
    ("SYLT:MusicIndex Transcript", frames::TRANSCRIPT_TIMED),
];

/// The reference source of the app: an `http` or `https` reference goes
/// through `remote_media`, and each other reference is a local path.
#[derive(Clone, Copy, Debug, Default)]
pub struct AppReferences;

impl ReferenceSource for AppReferences {
    fn text(&self, reference: &str) -> Result<String> {
        read_text_reference(reference)
    }

    fn picture(&self, reference: &str) -> Result<(String, Vec<u8>)> {
        read_picture_reference(reference)
    }
}

/// Writes `edits` into `path` with the reference source of the app.
///
/// The result names each edit that the crate did not write, in `skipped`.
/// The caller shows that list to the operator.
///
/// # Errors
///
/// Returns the error of [`musicindex_tags::write_id3v24_edits`].
pub fn write_id3v24_edits(path: &Path, edits: &[Id3v24Edit]) -> Result<TagWriteResult> {
    musicindex_tags::write_id3v24_edits(path, edits, &AppReferences)
}

fn read_text_reference(reference: &str) -> Result<String> {
    if reference.starts_with("http://") || reference.starts_with("https://") {
        let response = remote_media::fetch(reference, "transcript")?;
        // A redirect landing page answers with 200 and markup. Without this the
        // page body is embedded into the file's tags as transcript text.
        if let Some(content_type) = remote_media::declared_content_type(&response) {
            if content_type.starts_with("text/html") || content_type.contains("xhtml") {
                return Err(anyhow!(
                    "transcript {reference} returned markup ({content_type}), not transcript text"
                ));
            }
        }
        return response
            .text()
            .with_context(|| format!("read transcript {reference}"));
    }
    fs::read_to_string(reference).with_context(|| format!("read transcript {reference}"))
}

fn read_picture_reference(reference: &str) -> Result<(String, Vec<u8>)> {
    if reference.starts_with("http://") || reference.starts_with("https://") {
        let response = remote_media::fetch(reference, "APIC image")?;
        let declared_mime_type = remote_media::declared_content_type(&response);
        let data = response
            .bytes()
            .with_context(|| format!("read APIC image {reference}"))?
            .to_vec();
        if data.is_empty() {
            return Err(anyhow!("APIC image is empty"));
        }
        // Bytes only, no declared-type fallback. APIC writes an artifact, so a
        // 200 response carrying markup under `Content-Type: image/jpeg` must not
        // become the picture frame. Display paths may fall back to the declared
        // type because their worst case is a broken thumbnail (ADR 0056).
        let mime_type = image_type::from_bytes(&data).ok_or_else(|| {
            anyhow!(
                "APIC image response missing image type (declared {})",
                declared_mime_type.as_deref().unwrap_or("nothing")
            )
        })?;
        return Ok((mime_type, data));
    }

    let path = Path::new(reference);
    let data = fs::read(path).with_context(|| format!("read APIC image {}", path.display()))?;
    if data.is_empty() {
        return Err(anyhow!("APIC image is empty"));
    }
    let mime_type = image_type::from_path(path)
        .or_else(|| image_type::from_bytes(&data))
        .ok_or_else(|| anyhow!("unsupported APIC image type for {}", path.display()))?;
    Ok((mime_type, data))
}

#[cfg(test)]
mod tests {
    use super::{read_picture_reference, read_text_reference};

    /// A redirect landing page answers with 200 and markup. Without the
    /// content-type rule it is embedded into the file's tags as transcript
    /// text (ADR 0056).
    #[test]
    fn transcript_download_rejects_markup_body() {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buf = [0_u8; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::io::Write::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 18\r\nConnection: close\r\n\r\n<html>moved</html>",
            )
            .expect("write response");
        });

        let error = read_text_reference(&format!("http://{addr}/transcript.txt"))
            .expect_err("markup must not become transcript text");

        assert!(
            error.to_string().contains("returned markup"),
            "error should explain the markup rejection: {error}"
        );
    }

    #[test]
    fn apic_image_download_follows_redirect_with_spaces() {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut redirect_stream, _) = listener.accept().expect("accept redirect request");
            let mut buf = [0_u8; 1024];
            let _ = std::io::Read::read(&mut redirect_stream, &mut buf);
            let location = format!("http://{addr}/Assets/front cover.jpg");
            let response = format!(
                "HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Type: text/html\r\nContent-Length: 8\r\nConnection: close\r\n\r\nredirect"
            );
            std::io::Write::write_all(&mut redirect_stream, response.as_bytes())
                .expect("write redirect response");

            let (mut image_stream, _) = listener.accept().expect("accept image request");
            let mut buf = [0_u8; 1024];
            let _ = std::io::Read::read(&mut image_stream, &mut buf);
            std::io::Write::write_all(
                &mut image_stream,
                b"HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 13\r\nConnection: close\r\n\r\n\xff\xd8\xffjpeg bytes",
            )
            .expect("write image response");
        });

        let (mime_type, data) =
            read_picture_reference(&format!("http://{addr}/cover.jpg")).expect("read APIC image");

        assert_eq!(mime_type, "image/jpeg");
        assert_eq!(data, b"\xff\xd8\xffjpeg bytes");
    }

    /// APIC is stricter than the display paths: a declared image type on a 200
    /// response is not enough to embed artwork. Before this rule a lying server
    /// could put markup into the picture frame (ADR 0056).
    #[test]
    fn apic_image_download_rejects_markup_declared_as_an_image() {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buf = [0_u8; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::io::Write::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 18\r\nConnection: close\r\n\r\n<html>moved</html>",
            )
            .expect("write response");
        });

        let error = read_picture_reference(&format!("http://{addr}/cover.jpg"))
            .expect_err("a declared image type must not be enough to embed artwork");

        assert!(
            error.to_string().contains("missing image type"),
            "error should explain the byte-recognition failure: {error}"
        );
    }

    #[test]
    fn apic_image_download_rejects_non_image_redirect_body() {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buf = [0_u8; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::io::Write::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 17\r\nConnection: close\r\n\r\n<html>moved</html>",
            )
            .expect("write response");
        });

        let error = read_picture_reference(&format!("http://{addr}/cover.jpg"))
            .expect_err("non-image response should not become APIC artwork");

        assert!(
            error.to_string().contains("missing image type"),
            "error should explain image type validation: {error}"
        );
    }
}
