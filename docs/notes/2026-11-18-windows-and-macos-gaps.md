# Windows And macOS Gaps - 2026-11-18

## Purpose

This note lists what HitIt needs before it can run on Windows and on macOS. The orchestrator requested it on
2026-11-18 for the operator. It is not a port plan and not a decision. It records the gaps only.

This note is advisory. No decision record owns it.

## Method

- The code is at git `82acd38`.
- A ChatGPT run (`gpt-6.1-sol`, effort medium) searched the code, `Cargo.toml`, `Cargo.lock` and the crate sources.
- The main session examined each gap in the code before it put the gap in this note.
- Nobody built HitIt on Windows or on macOS. A gap marked **inference** comes from the code and the known behavior of
  the platform. A build or a run on that platform must confirm it.

Size estimate:

- **small**: a local change in one or two files.
- **medium**: a change in several files, or a new method for one feature.
- **large**: a new subsystem, a design decision, or work outside the code (signing, installers).

## Corrections To The First List

| Item | Result | Evidence |
| --- | --- | --- |
| 1. `gpui_platform` has only `x11` and `wayland` | Not a gap. The crate selects `gpui_macos` and `gpui_windows` by target. The `x11` and `wayland` features apply only to the Linux backend. A build on each platform must confirm this. | `Cargo.toml:43`, `gpui-pre-platform-0.3.1/Cargo.toml:65-88` |
| 2. mpv uses a Unix socket | Confirmed for Windows only. macOS has Unix sockets, and the mpv driver compiles there. | G-04 |
| 3. Broadcast control uses `systemctl` | Confirmed for both. Also `journalctl`, `cat` and `/proc`. | G-07, G-08, G-09 |
| 4. `ashpd` is Linux and FreeBSD only | The dependency is already gated, so it does not stop the build. The result is that the accessibility settings are not read. | `Cargo.toml:65-68`, G-20 |
| 5. About 20 files use Unix code | 21 files. Most uses have a `cfg(unix)` gate or are in tests. One production file stops the Windows build. | G-01, G-03 |
| 6. No method to find mpv, flac, ffmpeg and butt | Partly confirmed. On Windows, `Command::new` finds `flac.exe` through `PATH`. The `butt` check does not. On macOS, an app started from the Finder does not get the shell `PATH`. | G-05, G-06 |
| 7. CI on Linux only, signing | Confirmed. | G-22, G-23, G-24 |

## Gaps

### Build

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-01 | Windows | medium | The converter probe uses `UnixStream::pair` and `OwnedFd` with no platform gate. The production build stops. The probe needs a different bounded read of the converter output. | `src/audio_format/probe.rs:6-7`, `src/audio_format/probe.rs:112-118`, `src/audio_format.rs:3` |
| G-02 | Both | small | **Inference.** Two closure parameters are used only in a Linux block. On other systems they are unused, and `cargo clippy -- -D warnings` stops. | `src/ui/primitives/primary_selection.rs:113-122` |
| G-03 | Windows | medium | The tests do not compile and do not run. Seven test modules import `std::os::unix` with no gate. Test fixtures use `#!/bin/sh`, `/bin/cp` and `/bin/sleep`. A path test expects `/music` to be absolute. A move test writes a Windows path into TOML with no escape. | `src/audio_format/probe.rs:171-174`, `src/audio_format.rs:446`, `src/audio_tags.rs:2489`, `src/subscribe_service/materialization.rs:460`, `src/subscribe_service/materialization.rs:620`, `src/application/conversion_recovery.rs:543`, `src/application/conversion_recovery.rs:594`, `src/track_compare/retained.rs:204`, `src/view_models/startup/correction.rs:513`, `src/library_path.rs:111`, `src/config/user_data_move.rs:195` |

### Playback And Conversion

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-04 | Windows | medium | The mpv driver compiles only on Unix. On Windows, `driver = "mpv"` gives the error "mpv playback driver requires Unix-domain sockets". mpv on Windows uses a named pipe for its JSON IPC. | `src/playback_driver/mod.rs:13-17`, `src/playback_driver/mod.rs:64-75`, `src/playback_driver/mpv.rs:5-6`, `src/playback_driver/mpv.rs:79`, `src/playback_driver/mpv.rs:383` |
| G-05 | macOS | small | **Inference.** The app finds `flac`, `ffmpeg` and `mpv` through `PATH`. An app that the Finder or the Dock starts does not get the shell `PATH`, so Homebrew programs in `/opt/homebrew/bin` are not found. | `src/audio_format/probe.rs:44`, `src/audio_format/probe.rs:57`, `src/playback_driver/mod.rs:67` |
| G-06 | Windows | small | The `butt` check looks for the exact name with no `.exe` and no `PATHEXT`. It finds a path only when the path contains `/`. The app reports an installed `butt.exe` as absent. | `src/broadcast/encoder.rs:377-382` |
| G-07 | Both | small | The encoder check reads `/proc/*/comm`. Without `/proc`, the app runs the status command of the encoder in its place. The protection against a second start of `butt` is lost. | `src/broadcast/encoder.rs:385-394`, `src/broadcast/encoder.rs:357` |

### Broadcast

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-08 | Both | large | Local service control uses `systemctl --user` and `journalctl --user`, and the units `musicindex-live-publisher@INSTANCE.service` and `mixxx-now-playing.service`. macOS uses launchd. Windows uses its service manager. This needs a design decision: a service manager for each platform, or remote control only. | `src/broadcast/control.rs:17-18`, `src/broadcast/control.rs:52`, `src/app/show.rs:55` |
| G-09 | Windows | small | The app reads the drop file with `cat`, also for a local file. Windows has no `cat`. | `src/broadcast/control.rs:19`, `src/broadcast/control.rs:325` |
| G-10 | Both | small | The local publisher configuration path is `~/.config/musicindex-live-publisher/INSTANCE/config.toml`. A publisher that uses the platform folders of macOS or Windows has a different path. The publisher repository owns that path. | `src/broadcast/publisher_targets.rs:356-366` |
| G-11 | Windows | small | **Inference.** A drop-file target name can contain `:`, `?`, `*`, `"`, `<`, `>` or `\|`, or be a device name. The app uses the name in a file name, and the write fails. | `src/broadcast/producer.rs:287-300`, `src/broadcast/producer.rs:124` |
| G-12 | Both | small | Remote control runs the `ssh` program. Windows includes OpenSSH only as an optional feature. The app does not tell the user when `ssh` is missing. | `src/broadcast/transport.rs:79` |

### Files And Paths

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-13 | Windows | medium | **Inference.** The app opens a folder with `File::open` and calls `sync_all`. On Windows this open fails without a special flag. The Settings save, the database backup, the preservation, the restore and the upgrade repair stop before they replace a file. | `src/config/correction.rs:297-299`, `src/db/maintenance.rs:310`, `src/db/maintenance/preservation.rs:333`, `src/db/maintenance/upgrade.rs:326`, `src/db/maintenance/restore.rs:318`, `src/db/maintenance/restore.rs:448` |
| G-14 | Windows | medium | The owner-only permissions `0600` and `0700` apply only on Unix. On Windows the broadcaster token file gets the permissions of its folder. | `src/broadcast/tokens.rs:150-170`, `src/config/correction.rs:451`, `src/db/maintenance/preservation.rs:242`, `src/db/maintenance/preservation.rs:384` |
| G-15 | Windows | medium | The file identity checks (device and inode, hard links) apply only on Unix. On Windows, the staging check and the hard-link check do not occur, and a creation time replaces the inode check. | `src/track_compare/retained.rs:58`, `src/track_compare/retained.rs:124`, `src/db/maintenance/restore.rs:602`, `src/db/maintenance.rs:555` |
| G-16 | Windows | small | **Inference.** The name check rejects `CON` and `NUL`, but not `CON.txt`. Windows also reserves a device name with an extension. | `src/track_compare.rs:583-600`, `src/track_compare.rs:244` |
| G-17 | Windows | small | **Inference.** Each name part has a limit of 120 characters, but the full path has no limit. The app manifest of GPUI does not set `longPathAware`. A long path can fail on a system with the old limit of 260 characters. | `src/track_compare.rs:568`, `src/track_compare.rs:248` |
| G-18 | Both | small | The thumbnail cache is in the config folder, not in the cache folder. On Windows the config folder is in `AppData\Roaming`, so the cache moves with a roaming profile. On macOS it shares `Application Support` with the database. | `src/app/bootstrap.rs:216-219`, `directories-6.0.0/src/win.rs:73` |

### User Interface

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-19 | macOS | small | Select All in a log text accepts Ctrl+A and rejects Cmd+A. | `src/ui/composites/selectable_text.rs:168-173` |
| G-20 | Both | medium | The app reads Reduce Motion, Increase Contrast and Reduce Transparency only through the Linux portal. On other systems each setting stays off. | `src/app/bootstrap.rs:320-351`, `src/ui/system_settings.rs:67-70` |
| G-21 | Windows | medium | `src/main.rs` has no `windows_subsystem` attribute, so a console window opens with the app. The CLI needs a console, so one binary must select the mode. | `src/main.rs:1-15` |

### Verification And Distribution

| ID | Platform | Size | Gap | Evidence |
| --- | --- | --- | --- | --- |
| G-22 | Both | medium | CI runs only on `ubuntu-24.04` with Linux libraries. No job builds or tests on Windows or macOS. | `.github/workflows/ci.yml:27`, `.github/workflows/ci.yml:34-44` |
| G-23 | macOS | large | No `.app` bundle, `Info.plist`, `.icns` icon or `.dmg`. Without signing and notarization, Gatekeeper stops the app at the first start. | `packaging/arch/PKGBUILD:91-99` (Linux only) |
| G-24 | Windows | large | No installer, `.ico` icon or version resource. Without a signature, SmartScreen warns the user, and Smart App Control can block the app. | `packaging/arch/PKGBUILD:91-99` (Linux only) |
| G-25 | Both | small | The user documents give only Linux paths and Linux install steps. | `README.md:79-90`, `docs/architecture/app-overview.md:97-101` |

## What Is Not A Gap

The main session examined these points and found no gap. A build on each platform must confirm them.

- The fonts: the app embeds four Figtree weights (`src/ui/fonts.rs:33`, `src/ui/fonts.rs:55`).
- The keyboard: Cmd on macOS and Ctrl on other systems (`src/app/keyboard.rs:58`, `src/app/keyboard.rs:285`). The macOS
  menu exists (`src/app/menu.rs:125`).
- The dark and light theme: GPUI gives the system appearance (`src/ui/theme_bridge.rs:62`).
- The config, data and audio cache folders: `ProjectDirs` gives the platform folders (`src/config.rs:762`,
  `src/config.rs:769`, `src/config.rs:1141`).
- The move from the previous name `v4vmm`: it uses `ProjectDirs` and handles the shared macOS folder
  (`src/config/user_data_move.rs:35`, `src/config/user_data_move.rs:73`).
- `/run/user/1000` is only a comment example in the generated config (`src/config.rs:1175-1178`).
- The clipboard and the URL open use GPUI and the `open` crate. The PRIMARY selection code has a Linux gate
  (`src/ui/composites/selectable_text.rs:152`).
- The mpv runtime folder falls back to the temporary folder when `XDG_RUNTIME_DIR` is not set
  (`src/playback_driver/mpv.rs:415`).

## Counts

| Size | Count |
| --- | --- |
| small | 13 |
| medium | 9 |
| large | 3 |
| Total | 25 |

By platform: Windows only 13, macOS only 3, both 9.

Only G-01 stops the Windows production build. G-02 stops the zero-warning check on both systems if the inference is
correct. No gap stops the macOS production build.
