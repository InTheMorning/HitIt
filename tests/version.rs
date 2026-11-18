use std::process::Command;

#[test]
fn musicindex_adr_0083_cli_prints_one_version_line() {
    let data_dir = tempfile::tempdir().expect("create temporary app data");
    let config_dir = tempfile::tempdir().expect("create temporary app config");
    let cache_dir = tempfile::tempdir().expect("create temporary app cache");
    let output = Command::new(env!("CARGO_BIN_EXE_hitit"))
        .arg("--version")
        .env("XDG_DATA_HOME", data_dir.path())
        .env("XDG_CONFIG_HOME", config_dir.path())
        .env("XDG_CACHE_HOME", cache_dir.path())
        .output()
        .expect("run hitit --version");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("version output is UTF-8");
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.ends_with('\n'));

    // R-VERSION-9 item 1: a build in the Git checkout knows its git revision
    // and its build time, so the line holds the three build values.
    let line = stdout.trim_end();
    let values = line
        .strip_prefix("HitIt 0.1.0 (")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unexpected version line: {line}"));
    let (revision, date) = values
        .split_once(", ")
        .unwrap_or_else(|| panic!("no git revision and build date: {line}"));
    let hash = revision.strip_suffix("-dirty").unwrap_or(revision);
    assert!(
        hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit()),
        "{line}"
    );
    let date_shape = date.len() == 10
        && date.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    assert!(date_shape, "{line}");
}
