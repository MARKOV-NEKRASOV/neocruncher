# NeoCruncher GUI prototype (Linux)

This is a separate Rust desktop frontend for the existing NeoCruncher 1.0 CLI. It does not contain a new computation engine. It launches the release's `run_neocruncher.sh` with explicit process arguments, reads JSONL progress events, and displays CLI output and the independent verification result from the JSON report. No command is assembled through a shell string.

The [Ubuntu 24.04 x86_64 GUI trial](../dist/GUI-Prototype-0.1.3-README.md)
bundles the released 1.0 CPU CLI. Version 0.1.3 keeps the 0.1.1 Rust GUI
binary and selects native Wayland automatically on a GNOME Wayland session.
AppImageLauncher integration still needs a desktop test. The launcher,
per-user menu installer, and container build recipe are in [`packaging/`](packaging/).

## Build and run

Install a current Rust toolchain and the system libraries required by eframe's Linux X11/OpenGL backend. From this directory:

```sh
cargo test --locked
env -u DISPLAY cargo run --locked  # on a Wayland desktop
```

To include the test that actually runs the installed CLI and checks its JSONL
progress and independent verification, set the bundle path before testing:

```sh
NEOCRUNCHER_TEST_BUNDLE='/path/to/Neocruncher 1.0 release' cargo test --locked
```

That integration test is skipped when `NEOCRUNCHER_TEST_BUNDLE` is unset.

The GUI needs a graphical Linux session (`WAYLAND_DISPLAY` for Wayland or `DISPLAY` for X11). Set **Installed release** to the extracted bundle directory after running its `sh install.sh`, or set `NEOCRUNCHER_BUNDLE` before starting the GUI to prefill that field. The CLI's Python environment and its optional GPU driver remain separate prerequisites.

Enter a built-in constant, decimal digit count, execution mode, verification scope, and a new absolute output filename. The output directory must exist; the CLI does not overwrite a result or its `.json` report. **Start calculation** launches a job. **Cancel calculation** sends SIGINT to the launcher, which replaces itself with Python; a job that does not stop within five seconds is terminated. The GUI shows progress stages even when the engine cannot estimate a percentage. A displayed percentage belongs to the current stage, not necessarily to the complete job. For a run over 100,000 digits, select a prefix check or no independent check: the CLI cannot independently compare the entire output at that size. A prefix check validates only the stated prefix.

GPU and hybrid choices are available in the interface, but the CLI decides whether the selected constant and host OpenCL setup support them. A desktop test of the 0.1.2 GUI on native Wayland completed π at 1,000 digits on CPU with Full independent verification and `PASS`. Version 0.1.3 keeps those binaries and changes launcher selection; the 0.1.3 AppImageLauncher path still needs a desktop test. Separate production CLI and GUI distributions remain planned.

## UI review on a Linux desktop

The prototype uses a light workspace with two configuration panels, a run and verification panel, and a separate activity log. Review it at the default 940 × 720 window size and at its 760 × 620 minimum. Capture the initial, running, and completed states with the desktop screenshot tool so spacing, clipping, contrast, and keyboard focus can be compared between revisions.

For a functional visual pass, run 1,000 digits of π on CPU with **Full** independent verification and a fresh output path. Confirm the status reaches **Completed**, the verification line reports `PASS`, and the output and JSON report exist. Then inspect a prefix run, an existing-output rejection, and cancellation of a larger job. A displayed percentage describes the current CLI stage, not the whole calculation. GPU and hybrid controls need a separately qualified OpenCL host before visual checks can establish their behavior.
