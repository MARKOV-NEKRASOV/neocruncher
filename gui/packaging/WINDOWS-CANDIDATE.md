# NeoCruncher GUI 0.1.3 — Windows x86_64 candidate

This package contains a Rust GUI executable and a separately frozen
NeoCruncher 1.0 CPU CLI. It is an **unqualified candidate** until a person
opens it on a Windows 10/11 x86_64 desktop and completes the tests below.
The existing Linux AppImage is a separate build. GPU and hybrid OpenCL routes
have not been tested or qualified for this Windows package.

Keep `NeoCruncher-GUI.exe` beside the `neocruncher-cli` directory after
extracting the ZIP. Double-click `NeoCruncher-GUI.exe`; no Python or terminal
installation should be required. In the GUI choose π, 1000 decimal places,
CPU, Full independent verification, and a **new absolute** `.txt` path in an
existing writable folder. Click **Start calculation** and check that the UI
reaches `Completed` with `PASS`, the `.txt` and `.txt.json` files exist, and
the report identifies CPU execution. Then try an existing output path, which
must be rejected without replacing the file, and a longer calculation followed
by **Cancel calculation**. Cancellation terminates the frozen CLI immediately
on Windows; a partial output may remain and should be labeled failed.

The Windows candidate build uses the public 1.0 source archive plus the
reviewable `CLI-Windows-source-patch.py`. That patch adds Windows physical/free
RAM discovery and prevents the CLI worker from opening a console window. The
source and build workflow are under `gui/` and `.github/workflows/` in the
repository. The Rust GUI discovers the CLI in the adjacent directory or from
`NEOCRUNCHER_CLI` if explicitly set. The build CI tests the frozen CLI and
GUI-to-CLI worker path; CI does not establish that the graphical window works
on the owner's desktop.
