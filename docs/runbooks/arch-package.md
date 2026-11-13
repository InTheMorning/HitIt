# Arch Linux Package

## Purpose

Use this runbook to build and install the `hitit-git` Arch Linux package.
`packaging/arch/PKGBUILD` gets the source from GitHub. The package installs
the `hitit` binary, a desktop file, the app icon, the license and the README.

The package builds the commit at the top of `main` on GitHub. It does not
build local commits that nobody pushed.

## Prerequisites

- An Arch Linux system with `base-devel`.
- `rustup`. The package build installs the toolchain in `rust-toolchain.toml`.
  `rustup` conflicts with the Arch `rust` package.
- Network access to GitHub and crates.io during the build.

## Build And Install

1. Copy the `packaging/arch/PKGBUILD` file to an empty folder.
2. Go to that folder.
3. Run this command:

   ```bash
   makepkg -si
   ```

`makepkg -s` installs the packages in `depends` and `makedepends`. The `check()`
step runs `cargo test`. The tests need no display, no network and no local
data. Add `--nocheck` to skip the tests.

## Optional Packages

`musicindex-live-publisher` is a different package. The Show page needs it on
this machine only when the publisher runs on this machine. Install it to
control event targets and the `musicindex-live-publisher` and
`mixxx-now-playing` user services from the Show page. A publisher on a remote
host needs `openssh` on this machine. The
[broadcast operations](broadcast-operations.md) runbook tells how to operate
the chain.

`flac` or `ffmpeg` converts WAV downloads. `mpv` is the playback driver.
`butt` is the stream encoder.

## Package Options

The PKGBUILD sets `options=('!lto')`. The makepkg default adds `-flto=auto` to
`CFLAGS`. The cc crate compiles the bundled SQLite and aws-lc C sources with
these flags. The result is GCC LTO bitcode, and the Rust linker cannot read it.
Without `!lto`, the link stops with `undefined symbol` errors for SQLite and
aws-lc.

## Package Checks

Run these commands in the folder of the PKGBUILD after a change to the
PKGBUILD:

```bash
namcap PKGBUILD
namcap hitit-git-*.pkg.tar.zst
```
