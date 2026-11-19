# ADR 0103 Task 001: The `.deb`, The AppImage And The Release Workflow

## Goal

Make the `.deb` and the AppImage of ADR 0103, and the release workflow that makes them on a tag.

## Authority

- [ADR 0103](../adr/0103-the-linux-packages-and-the-release-workflow.md) Decisions 1 to 6.
- musicindex ADR 0084 R-PACKAGE-1, R-PACKAGE-4 and R-PACKAGE-5, through ADR 0103.

## Recorded Facts

- `packaging/arch/PKGBUILD` installs the binary, `packaging/arch/hitit.desktop`, `src/assets/music_network_logo.png` as the 512 x 512 icon, `LICENSE` and `README.md`.
- `.github/workflows/ci.yml` installs the Linux build packages and runs `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.
- `appimagetool` 1.9.1: `https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage`, SHA-256 `ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0`.
- type2 runtime `20251108`: `https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64`, SHA-256 `2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d`.
- The Studio workflow `desktop-release.yml` (musicindex-studio commit `cf4fc1e`) is the model for the tag check, the artifacts, the checksums and the draft Release.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Desktop file | reuse | `packaging/arch/hitit.desktop` | The `.deb` and the AppImage use the same file. Do not write a second desktop file. |
| App icon | reuse | `src/assets/music_network_logo.png` | The PKGBUILD uses this 512 x 512 icon. |
| CI checks | reuse | `.github/workflows/ci.yml` | The release workflow runs the same package installs and checks. |
| Package runbook | extend | `docs/runbooks/arch-package.md` | Searched HitIt `docs/runbooks/`: only the Arch runbook exists. Add the `.deb` and the AppImage to a new section, or rename it. The main session selects. |
| AppDir script | new | `packaging/appimage/build-appimage.sh` | Searched HitIt `packaging/`, `scripts/` and `.github/`: no AppImage code exists. musicindex-lib was not read (operator rule of 2026-11-10). |
| glibc script | new | `packaging/check-glibc-floor.sh` | Searched HitIt `packaging/`, `scripts/` and `.github/`: no symbol version check exists. |

## Required Changes

1. `Cargo.toml`: add `[package.metadata.deb]` as ADR 0103 Decision 2 and Decision 6 item 2 give.
2. `packaging/appimage/AppRun` and `packaging/appimage/build-appimage.sh`. The script takes the binary path, the version and the tool paths. It makes the AppDir of ADR 0103 Decision 3 and runs `appimagetool --runtime-file`.
3. `packaging/check-glibc-floor.sh`: read the `GLIBC_` versions with `objdump -T`. When a version is later than the floor argument, stop. The message names ADR 0103 Decision 4.
4. `.github/workflows/release.yml` as ADR 0103 Decision 5 gives. Use `ubuntu-22.04`. Examine the SHA-256 of each downloaded tool.
5. The runbook: how to make the two packages on a local system.
6. `AGENTS.md`: name the three packages and the release workflow, and cite ADR 0103.

## Do Not Change

- `packaging/arch/PKGBUILD`, except a comment that names ADR 0103.
- `.github/workflows/ci.yml`.
- Any file in `src/`.

## Acceptance Criteria

Mechanical:

1. `cargo deb --no-build` on a release binary makes `target/debian/hitit_0.1.0-1_amd64.deb`. `dpkg-deb --field` shows `Recommends: mpv, flac, ffmpeg, openssh-client, butt` and `Suggests: musicindex-live-publisher`.
2. `dpkg-deb --contents` lists `/usr/bin/hitit`, the desktop file, the icon and the README.
3. The script makes `HitIt-0.1.0-x86_64.AppImage`. `--appimage-extract` shows no `.so` file.
4. The glibc script exits 0 for a floor of 2.44 and exits 1 for a floor of 2.35 on the Arch build.
5. `actionlint` or `python3 -c 'import yaml'` parses `release.yml` with no error.
6. `cargo test --test architecture_tests` passes, then `cargo build --bin hitit`.

Visual (open until a person or the agent walk does it):

1. The AppImage starts the window on Xvfb with the icon and the fonts.
2. The `.deb` installs and starts on Ubuntu 22.04.

## Report

Give each command and its result. Give the size of the `.deb` and the AppImage. Say which criterion is open.

## Stop And Ask

- A tool version above does not work. Do not select a different version.
- A change in `src/` seems necessary.

The delegate does not commit. Write prose in STE (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).
