# GUI 0.1.3 source and execution contract

This document describes the **published Linux desktop trial** and the Windows
candidate source, not the proposed NeoCruncher engine release. The Rust crate
still declares version `0.1.1`; the Linux `0.1.3` trial repackages the earlier
frontend and CLI binaries with a corrected launcher. The Windows candidate
adds platform-specific source changes and has no qualified binary yet.
Source for the arithmetic program is in the separate
[`Neocruncher-1.0.0-release.zip`](../Neocruncher-1.0.0-release.zip).

## Source map

| File | Responsibility |
| --- | --- |
| [`src/main.rs`](src/main.rs) | egui interface, input validation, process launch, JSONL progress reader, cancellation, and verification display. |
| [`Cargo.toml`](Cargo.toml), [`Cargo.lock`](Cargo.lock) | Rust dependencies and reproducible dependency resolution. |
| [`packaging/AppRun`](packaging/AppRun) | AppImage/direct-archive launcher, Wayland/X11 selection, diagnostics, CPU self-test. |
| [`packaging/cli_entry.py`](packaging/cli_entry.py) | Entry point used to freeze the released 1.0 CLI for the trial package. |
| [`packaging/build-inside-ubuntu24.sh`](packaging/build-inside-ubuntu24.sh), [`packaging/Dockerfile.ubuntu24`](packaging/Dockerfile.ubuntu24) | Ubuntu 24.04 packaging recipe. |
| [`packaging/install-desktop.sh`](packaging/install-desktop.sh) | Per-user application-menu entry for the extracted archive. |
| [`packaging/patch_windows_cli.py`](packaging/patch_windows_cli.py), [`../.github/workflows/windows-gui-013.yml`](../.github/workflows/windows-gui-013.yml) | Hash-checked Windows CPU CLI source adaptation, native build and frozen-CLI smoke test. |

## Runtime boundary

```text
egui UI thread ── validated Job ──> Rust worker ── argv ──> NeoCruncher 1.0 CLI
       ^                            │                       │
       └── WorkerEvent channel <────┴── JSONL progress ────┘
                                      CLI stdout/stderr and result.json
```

`NeoApp::validated_job` checks the digit count, output path, installed bundle
or packaged CLI path, and full-verification limit before starting. It refuses
an existing output or JSON report. `Job::cli_args` maps the selected constant,
digits, mode, and verification setting to individual CLI arguments. Rust
`Command` passes those arguments directly; it does not interpolate a shell
command. In a source build the worker calls the installed
`run_neocruncher.sh`; in the distributed trial it calls the packaged CLI
selected by `NEOCRUNCHER_CLI`.
On Windows the packaged CLI may also be found in the `neocruncher-cli`
directory beside the GUI executable; an installed Unix shell bundle is not
accepted. The Windows build uses the released 1.0 source plus a hash-checked
patch to obtain RAM capacity through `GlobalMemoryStatusEx`, use UTF-8 for
worker pipes, and hide the worker console. The frozen entry point sets UTF-8
for progress output. The GUI and CLI remain separate executables in one ZIP.

The worker polls a temporary JSONL progress file. It processes complete
newline-terminated records and leaves a partial final record for the next
poll. It also streams CLI stdout and stderr to the activity log. These are
separate information channels: a displayed stage percentage is a phase
percentage, not a measure of total completion. On exit the worker reads the
adjacent `.json` report's `independent_verification` field. A requested Full
or Prefix check counts as a successful GUI job only if that field begins with
`PASS:` or `PASS_PREFIX:` respectively and the CLI exits successfully.
With verification set to None, a successful CLI exit completes the job without
claiming an independent check. The GUI does not calculate or certify digits.

On Unix, cancellation sends `SIGINT` to the CLI process and, after five
seconds, kills it if still running. On Windows the GUI terminates the hidden
frozen CLI immediately because Unix signals are unavailable. The installed
Unix shell launcher replaces itself with Python; the packaged routes run
their frozen CLI directly. A cancelled job is not marked
successful. The temporary progress file is removed when its owning worker
returns; result and report files remain under the user-selected path.

## Desktop packaging

`AppRun` chooses native Wayland when the session is Wayland and
`WAYLAND_DISPLAY` is present, even if GNOME also sets `DISPLAY` for XWayland.
Otherwise it chooses X11 when `DISPLAY` exists. `--wayland` and `--x11` force
either path; `--diagnose` prints the selection and missing linked libraries.
The trial defaults to software OpenGL for the small desktop UI. This choice
affects window rendering, not the CLI calculation mode. `--self-test` runs a
packaged CPU π calculation at 1000 digits and checks for an independent Full
`PASS` in the CLI report.

The AppImage can be integrated with AppImageLauncher. The direct archive has
`install-desktop.sh`, which writes a menu entry under the current user's
`~/.local/share/applications`; its path must remain stable. Neither path
installs a system service or edits the arithmetic engine.

## What is verified and what is pending

The published 0.1.3 AppImage passed its extract-and-run CPU self-test in the
build environment. The owner tested the earlier GUI 0.1.2 on native Wayland:
π at 1000 digits, CPU, Full verification reached `Completed` and `PASS`.
Version 0.1.3 changes launcher selection; its AppImageLauncher integration
still needs a desktop test. GPU and hybrid selections are exposed by the GUI
but require a separately supported CLI route and OpenCL setup. This package
is not evidence for a new Vulkan backend, NeoBayes, GPU-only π, or a speedup.
The Windows ZIP remains an **unqualified candidate** until its native build,
frozen CPU/GUI-worker checks and a manual Windows desktop window/run test pass.

See the [GUI README](README.md) for building and the
[Spanish trial guide](../dist/GUI-Prototype-0.1.3-README.md) for installation.
