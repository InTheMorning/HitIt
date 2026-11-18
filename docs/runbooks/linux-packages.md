# Linux Packages

## Purpose

Use this runbook to make the `.deb` and the AppImage. ADR 0103 defines the package contents and the glibc floor.

## Prerequisites

- Linux with the libraries for the app listed in `.github/workflows/ci.yml`.
- `dpkg-deb`, `dpkg-shlibdeps`, `objdump`, `cargo-deb` 3.8.0 and `appimagetool` 1.9.1.
- The AppImage type2 runtime release `20251108`.
- A release binary in `target/release/hitit`.

On Arch Linux, `dpkg-shlibdeps` or `dpkg-deb` might not be installed. The `$auto` dependency field needs `dpkg-shlibdeps`. Use Ubuntu 22.04 to make the `.deb` if the local system cannot resolve `$auto`. Keep `$auto` in `Cargo.toml`.

## Make The Packages

1. Make the release binary:

   ```bash
   cargo build --release --bin hitit
   ```

2. Examine the glibc floor:

   ```bash
   packaging/check-glibc-floor.sh target/release/hitit 2.35
   ```

3. Make the `.deb`:

   ```bash
   cargo deb --no-build
   ```

   The package is in `target/debian/`.

4. Put `appimagetool` and the type2 runtime in `target/`. Then make the AppImage:

   ```bash
   packaging/appimage/build-appimage.sh target/release/hitit 0.1.0 target/appimagetool.AppImage target/runtime-x86_64
   ```

   The AppImage is in `target/`. On a system without FUSE, set `APPIMAGE_EXTRACT_AND_RUN=1` when you start `appimagetool`.

5. Examine package contents and checksums:

   ```bash
   dpkg-deb --field target/debian/hitit_0.1.0-1_amd64.deb
   dpkg-deb --contents target/debian/hitit_0.1.0-1_amd64.deb
   target/HitIt-0.1.0-x86_64.AppImage --appimage-extract
   ```

The AppDir includes no shared libraries. The host provides its graphics libraries and drivers.

## Release Workflow

The workflow runs on `v*` tags and on manual dispatch. It runs on Ubuntu 22.04, checks the tool hashes and glibc floor, then makes the two packages. A tag run uploads a draft Release. The operator publishes it.
