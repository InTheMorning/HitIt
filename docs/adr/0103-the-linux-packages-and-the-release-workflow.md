# ADR 0103: The Linux Packages And The Release Workflow

## Status

Accepted - 2026-11-18. The orchestrator session relayed the operator request of 2026-11-18. musicindex ADR 0084 R-PACKAGE-5 gives the selection of the build tools to this repository.

- Bar: crosses repositories. musicindex ADR 0084 binds HitIt, Studio and the three services. It is also difficult to change later, because a user who installs a package depends on its name, its folders and its dependencies.

Uses: musicindex ADR 0084 R-PACKAGE-1, R-PACKAGE-4 and R-PACKAGE-5 (musicindex commit `2518515`). This session did not read a musicindex file. The orchestrator session sent the text of ADR 0084 and of the Studio workflow `desktop-release.yml` (musicindex-studio commit `22971ee`).

Terms:

- A **`.deb`** is a Debian package file. This is an established term.
- An **AppImage** is one Linux file that contains an app. This is an established term.
- An **AppDir** is the folder from which `appimagetool` makes an AppImage. This is an established AppImage term.
- The **glibc floor** is the earliest GNU C library version that can run the binary. This is a term of this record.

## Context

- `packaging/arch/PKGBUILD` builds the package `hitit-git` from the top of `main` on GitHub. The `check()` step runs the tests.
- `.github/workflows/ci.yml` runs the checks on `ubuntu-24.04`. HitIt has no release workflow, no `.deb` and no AppImage.
- HitIt is a GPUI app. The Tauri bundler of Studio does not apply.
- GPUI loads `libvulkan`, `libEGL`, `libwayland-client` and `libwayland-egl` at run time. The graphics driver must come from the host.
- A build on Arch Linux on 2026-11-18 needs `GLIBC_2.44`. That binary does not start on Ubuntu 22.04 or Debian 12.
- The app starts these external programs: `mpv`, `flac`, `ffmpeg`, `butt` and `ssh`. It controls `musicindex-live-publisher` through `systemctl --user`.
- The app tells the user about some missing programs and not about others. See Decision 6.

## Decision

### 1. HitIt ships three Linux packages

This applies musicindex ADR 0084 R-PACKAGE-1.

1. The Arch package `hitit-git` from `packaging/arch/PKGBUILD`.
2. The Debian package `hitit_VERSION-1_amd64.deb`.
3. The AppImage `HitIt-VERSION-x86_64.AppImage`.

The `.deb` and the AppImage are for `x86_64` only. The PKGBUILD has `arch=('x86_64')`. An `aarch64` package needs a new decision.

### 2. cargo-deb makes the `.deb`

1. `cargo-deb` 3.8.0 makes the `.deb` from `[package.metadata.deb]` in `Cargo.toml`. The workflow installs this version with `--locked`.
2. `Depends` is `$auto`, plus each library that GPUI loads at run time: `libvulkan1`, `libegl1`, `libwayland-client0`, `libwayland-egl1`, `fontconfig` and `ca-certificates`.
3. The `.deb` installs the same files as the PKGBUILD: `/usr/bin/hitit`, `packaging/arch/hitit.desktop` as `/usr/share/applications/hitit.desktop`, the icon as `/usr/share/icons/hicolor/512x512/apps/hitit.png`, and the README.

Reason: `cargo-deb` reads the crate data that the build already uses, and `$auto` finds the libraries that the binary links. A script that calls `dpkg-deb` directly must write the control file and the folder tree again.

### 3. appimagetool makes the AppImage from a small AppDir

1. A script in `packaging/appimage/` makes an AppDir with `AppRun`, `hitit.desktop`, `hitit.png` and `usr/bin/hitit`. The AppDir contains no shared library.
2. `appimagetool` 1.9.1 makes the AppImage with the type2 runtime release `20251108`. The workflow downloads both files and examines their SHA-256 values before it uses them.
3. linuxdeploy is not used.

Reason: linuxdeploy copies the shared libraries of the binary into the AppDir. The graphics libraries must come from the host, and the other libraries are on each desktop system that can run GPUI. A copy adds size and can load a library that does not agree with the host driver.

### 4. The build runs on the earliest supported Ubuntu

1. The release workflow builds on `ubuntu-22.04`. The glibc floor of the `.deb` and of the AppImage is 2.35.
2. A script in the workflow reads the `GLIBC_` symbol versions of the binary. It stops the workflow when a version is later than 2.35.
3. When GitHub removes `ubuntu-22.04`, a new decision gives the next floor.

### 5. A release workflow makes the packages, and the operator releases them

1. `.github/workflows/release.yml` runs on a tag `v*` and on `workflow_dispatch`.
2. It runs the checks of `ci.yml`, builds the release binary, makes the `.deb` and the AppImage, and writes `SHA256SUMS`.
3. On a tag, it makes or refreshes a draft GitHub Release with the three files. It never publishes the Release.
4. The workflow stops when the tag version and the `Cargo.toml` version are different.
5. The operator makes each tag, each push and each publication.
6. The release workflow does not build the PKGBUILD. `hitit-git` builds from the source on the host of the user. A build of it on the runner needs an Arch Linux container and proves only that the source builds, which `ci.yml` already proves.

This decides musicindex ADR 0084 Open items 1 and 2 for HitIt.

### 6. External programs and missing-program notices

This applies musicindex ADR 0084 R-PACKAGE-4.

1. The AppImage and the `.deb` contain no external program. The app uses the program on the host.
2. The PKGBUILD lists each external program in `optdepends`. The `.deb` lists `mpv`, `flac`, `ffmpeg`, `openssh-client` and `butt` in `Recommends`, and `musicindex-live-publisher` in `Suggests`.
3. The app tells the user when a program is missing:

| Program | Notice today | Evidence |
| --- | --- | --- |
| `flac`, `ffmpeg` | Yes. The Audio converter report says "executable was not found" for each. | `src/view_models/startup/converter.rs:37` |
| `butt` | Yes. The Show page shows "Not installed". | `src/view_models/show.rs:494` |
| `musicindex-live-publisher` | Yes. The Show page shows "Unit file not found." and "Relay sender target commands are not installed." | `src/view_models/show.rs:506`, `src/view_models/show.rs:2446` |
| `mpv` | Yes, since task 002. The Background tools report says "mpv is not installed" and gives the path that the app tried. | `src/view_models/startup/capabilities.rs`, `src/playback_driver/mpv.rs` (`MpvNotInstalled`) |
| `ssh` | Yes, since task 002. The Show page shows "ssh not installed" for a remote service. | `src/broadcast/control.rs` (`ServiceState::ProgramNotInstalled`), `src/view_models/show.rs` |

[Task 002](../tasks/adr-0103-task-002-missing-program-notices.md) added the notices for `mpv` and `ssh`.

`musicindex-live-publisher` is in `Suggests` because it has no Debian package on 2026-11-18. musicindex ADR 0084 gives it one later. Then it moves to `Recommends`.

### Situational Rules

Each rule of this record is situational. Decisions 2, 3 and 4 hold while the tools and the runner of 2026-11-18 exist. A change of a tool, of the runner or of the set of packages supersedes this record.

## Checks

Mechanical:

- The release workflow makes the `.deb`, the AppImage and `SHA256SUMS` (Decision 5). [Task 001](../tasks/adr-0103-task-001-deb-appimage-and-release-workflow.md) proves it with a local build and the first workflow run.
- The glibc script stops the workflow above 2.35 (Decision 4).
- The PKGBUILD `optdepends` and the `.deb` `Recommends` name each program of Decision 6 item 2. Task 001 examines `dpkg-deb --field` on the local `.deb`.
- The view model names `mpv` when the player cannot start, and the remote host state names `ssh` when the program is missing (Decision 6). Task 002 adds the tests.

Visual, for a person:

- On a host without `mpv`, the AppImage starts and the playback notice names `mpv`. On a host without `flac` and `ffmpeg`, Settings, Library, Audio converter shows that each is missing (musicindex ADR 0084 Checks).
- The AppImage and the `.deb` start on Ubuntu 22.04 and on the newest Ubuntu, with the window, the icon and the fonts.

## Consequences

- `Cargo.toml` gets `[package.metadata.deb]`.
- `packaging/appimage/` holds the AppDir script and `AppRun`.
- A new runbook tells how to make the packages on a local system.
- `AGENTS.md` names the three packages and the release workflow.

## Plan

[The Linux packages plan](../plans/adr-0103-linux-packages-plan.md).
