//! Shared build values for musicindex ADR 0083 R-VERSION-8.

#[must_use]
pub(crate) fn info() -> musicindex_build_info::BuildInfo {
    musicindex_build_info::build_info!("HitIt")
}

#[cfg(test)]
mod tests {
    use musicindex_build_info::BuildInfo;

    #[test]
    fn version_line_formats_known_build_values() {
        let info = BuildInfo::new(
            "HitIt",
            "0.1.0",
            Some("a1b2c3d"),
            Some("2026-11-18T12:00:00Z"),
        );

        assert_eq!(info.version_line(), "HitIt 0.1.0 (a1b2c3d, 2026-11-18)");
    }
}
