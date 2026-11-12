# ADR 0096: The App Name Is HitIt

## Status

Accepted - 2026-11-11. The operator selected the name, the scope and the data move on 2026-11-11.

Amended 2026-11-11: the operator renamed the GitHub repository to `InTheMorning/HitIt` and the local directory to `~/build/HitIt`. The Consequences section records this change.

## Context

The app had the name v4vmm. The operator renamed the app HitIt on 2026-11-11.

The crate, the binary, the CLI help and the log prefixes used the name v4vmm. The HTTP User-Agent, the temporary file names and the user directories also used it. The configuration is in `~/.config/v4vmm/config.toml`. The library database is in `~/.local/share/v4vmm/v4vmm.sqlite`. The configuration file keeps the database path as text.

## Decision

1. The display name of the app is HitIt. Text that a person reads uses HitIt.
2. The machine name is `hitit`. The crate, the binary, the CLI help, the log prefixes, the User-Agent, the thread names and the temporary file prefixes use `hitit`. Environment variables use the prefix `HITIT_`.
3. The project directories come from `ProjectDirs::from("xyz", "HeyCitizen", "hitit")`. The default database file is `hitit.sqlite` in the data directory.
4. The window has the title HitIt and the `app_id` value `hitit`.
5. Before the app or a CLI command reads the configuration, the app moves the v4vmm user data one time:
   1. If there is no `hitit` configuration directory and there is a `v4vmm` configuration directory, the app moves the `v4vmm` directory to the `hitit` path.
   2. If there is no `hitit` data directory and there is a `v4vmm` data directory, the app moves the `v4vmm` directory to the `hitit` path.
   3. In the `hitit` data directory, if there is no `hitit.sqlite` file, the app renames `v4vmm.sqlite` to `hitit.sqlite`. The app also renames its `-wal`, `-shm` and `-journal` files.
   4. If `db_path` in the configuration file gives the v4vmm default database path, the app replaces it with the `hitit` default path. The app changes no other line.
6. The app moves the files. It does not make a second set of the data. If a move or a rename fails, the app shows the error with the two paths and stops. It does not start with an empty library.
7. Archived records, finished tasks, reviews, notes and dated run records keep the name v4vmm, because they are history. In ADRs 0001 to 0095, the name v4vmm means HitIt. In those records, the paths and commands with `v4vmm` mean the `hitit` forms of Decisions 2 and 3.
8. The shared file `src/assets/tokens.json` is a copy of the file in `musicindex-assets` (musicindex ADR 0032 R-DOT-2). This app does not change its copy. The text of the shared file changes through musicindex.

## Consequences

- The code of Decision 5 is necessary only for the move from v4vmm. A subsequent record can remove it after the operator confirms that no install uses the `v4vmm` directories.
- The `mixxx-now-playing` producer of musicindex-live-publisher reads `music_dir` from the HitIt configuration. A publisher ADR gives it the `hitit` path first and the `v4vmm` path second. The code of Decision 5 stays until the publisher no longer needs the `v4vmm` path.
- After a crash, temporary files with a `v4vmm` prefix can stay in the directories. The app does not remove them, because the cleanup code looks only for the `hitit` prefixes. A person can remove them.
- The GitHub repository is `InTheMorning/HitIt` and the local directory is `~/build/HitIt`. In the history records of Decision 7, the path `~/build/v4vmm` means `~/build/HitIt`. The other suite repositories keep the name v4vmm in their text until their owners change it. The orchestrator gets a message about this change.
- A unit test enforces Decision 5 with temporary directories. No test enforces Decisions 1 and 2, because no incident broke them. A person examines the window title and the CLI help when a change touches the name.
