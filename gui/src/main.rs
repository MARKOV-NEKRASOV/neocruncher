// SPDX-License-Identifier: GPL-3.0-only
//! A separate desktop frontend for the existing NeoCruncher CLI.
use eframe::egui;
use serde_json::Value;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

const CONSTANTS: &[&str] = &[
    "pi",
    "e",
    "sqrt2",
    "phi",
    "ln2",
    "zeta3",
    "catalan",
    "gamma",
    "ln10",
    "zeta5",
    "lemniscate",
    "gamma_quarter",
    "gamma_third",
    "gamma_fifth",
];

/// A packaged CLI is supplied by the Linux AppImage environment or beside a
/// Windows GUI executable. A missing Windows CLI never starts a shell script.
fn packaged_cli_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("NEOCRUNCHER_CLI") {
        return Some(PathBuf::from(path));
    }
    #[cfg(windows)]
    {
        let exe = std::env::current_exe().ok()?;
        let cli = exe.parent()?.join("neocruncher-cli").join("neocruncher-cli.exe");
        return cli.is_file().then_some(cli);
    }
    #[cfg(not(windows))]
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Cpu,
    Gpu,
    Hybrid,
}
impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Gpu => "OpenCL GPU (experimental)",
            Self::Hybrid => "CPU + GPU (experimental)",
        }
    }
    fn cli_value(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
            Self::Hybrid => "both",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Verification {
    Full,
    Prefix,
    None,
}

/// Immutable request sent from the UI thread to one CLI worker.
struct Job {
    bundle: PathBuf,
    packaged_cli: Option<PathBuf>,
    constant: String,
    digits: u64,
    mode: Mode,
    verification: Verification,
    output: PathBuf,
}
impl Job {
    fn cli_args(&self, progress: &Path) -> Vec<OsString> {
        let mut args = vec![
            "--constant".into(),
            self.constant.clone().into(),
            "--digits".into(),
            self.digits.to_string().into(),
            "--compute-mode".into(),
            self.mode.cli_value().into(),
            "--output".into(),
            self.output.as_os_str().to_owned(),
            "--progress-file".into(),
            progress.as_os_str().to_owned(),
        ];
        match self.verification {
            Verification::Full => args.push("--verify".into()),
            Verification::Prefix => {
                args.push("--verify-prefix".into());
                args.push(self.digits.min(100_000).to_string().into());
            }
            Verification::None => {}
        }
        args
    }
}

/// Messages returned to egui; workers never mutate UI state directly.
enum WorkerEvent {
    Log(String),
    Progress {
        stage: String,
        percent: Option<f32>,
    },
    Finished {
        success: bool,
        exit: String,
        verification: String,
    },
}

/// Presentation state and the channels for the current calculation.
struct NeoApp {
    theme_ready: bool,
    bundle: String,
    constant: usize,
    digits: String,
    mode: Mode,
    verification: Verification,
    output: String,
    status: String,
    progress: Option<f32>,
    verification_result: String,
    log: Vec<String>,
    events: Option<Receiver<WorkerEvent>>,
    cancel: Option<Sender<()>>,
}
impl Default for NeoApp {
    fn default() -> Self {
        Self {
            theme_ready: false,
            bundle: std::env::var("NEOCRUNCHER_BUNDLE").unwrap_or_default(),
            constant: 0,
            digits: "1000".into(),
            mode: Mode::Cpu,
            verification: Verification::Full,
            output: std::env::temp_dir()
                .join("neocruncher-pi-1000.txt")
                .display()
                .to_string(),
            status: "Ready".into(),
            progress: None,
            verification_result: "Not run".into(),
            log: Vec::new(),
            events: None,
            cancel: None,
        }
    }
}
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.panel_fill = egui::Color32::from_rgb(246, 248, 252);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(219, 232, 255);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(219, 232, 255);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(239, 245, 255);
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    ctx.set_style(style);
}
impl NeoApp {
    /// Reject inputs that cannot be launched or would overwrite an existing result.
    fn validated_job(&self) -> Result<Job, String> {
        let bundle = PathBuf::from(self.bundle.trim());
        let packaged_cli = packaged_cli_path();
        let digits: u64 = self
            .digits
            .trim()
            .parse()
            .map_err(|_| "Digits must be a positive integer".to_string())?;
        if digits == 0 {
            return Err("Digits must be positive".into());
        }
        if self.verification == Verification::Full && digits > 100_000 {
            return Err("Full independent verification supports at most 100,000 digits; select prefix or none".into());
        }
        let output = PathBuf::from(self.output.trim());
        if output.as_os_str().is_empty() || output.file_name().is_none() {
            return Err("Select a result filename".into());
        }
        if !output.is_absolute() {
            return Err("Result filename must be an absolute path".into());
        }
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        if !parent.is_dir() {
            return Err("Output directory does not exist".into());
        }
        if output.exists() || PathBuf::from(format!("{}.json", output.display())).exists() {
            return Err("Result or JSON report already exists; choose a new filename".into());
        }
        if let Some(cli) = &packaged_cli {
            if !cli.is_absolute() || !cli.is_file() {
                return Err("Packaged NeoCruncher CLI is unavailable".into());
            }
        } else {
            #[cfg(windows)]
            return Err("The Windows GUI requires its bundled neocruncher-cli.exe beside the GUI executable".into());
            #[cfg(unix)]
            {
            if !bundle.is_absolute() {
                return Err("Installed release directory must be an absolute path".into());
            }
            if !bundle.join("run_neocruncher.sh").is_file()
                || !bundle.join(".venv-system/bin/python").is_file()
            {
                return Err("Select an installed release directory containing run_neocruncher.sh and .venv-system".into());
            }
            }
        }
        Ok(Job {
            bundle,
            packaged_cli,
            constant: CONSTANTS[self.constant].into(),
            digits,
            mode: self.mode,
            verification: self.verification,
            output,
        })
    }
    fn start(&mut self) {
        match self.validated_job() {
            Err(message) => self.status = message,
            Ok(job) => {
                let (events_tx, events_rx) = mpsc::channel();
                let (cancel_tx, cancel_rx) = mpsc::channel();
                thread::spawn(move || run_job(job, events_tx, cancel_rx));
                self.events = Some(events_rx);
                self.cancel = Some(cancel_tx);
                self.log.clear();
                self.progress = None;
                self.verification_result = "Pending".into();
                self.status = "Starting calculation".into();
            }
        }
    }
    fn receive_events(&mut self) {
        let events: Vec<_> = self
            .events
            .as_ref()
            .map(|r| r.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                WorkerEvent::Log(line) => {
                    self.log.push(line);
                    if self.log.len() > 200 {
                        self.log.remove(0);
                    }
                }
                WorkerEvent::Progress { stage, percent } => {
                    self.status = stage;
                    self.progress = percent;
                }
                WorkerEvent::Finished {
                    success,
                    exit,
                    verification,
                } => {
                    self.status = if success {
                        "Completed".into()
                    } else {
                        format!("Stopped: {exit}")
                    };
                    self.verification_result = verification;
                    self.progress = if success { Some(1.0) } else { None };
                    self.events = None;
                    self.cancel = None;
                }
            }
        }
    }
}
impl eframe::App for NeoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_ready {
            apply_theme(ctx);
            self.theme_ready = true;
        }
        self.receive_events();
        let running = self.events.is_some();
        let border = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(225, 230, 238));
        let muted = egui::Color32::from_rgb(100, 113, 132);
        let ink = egui::Color32::from_rgb(31, 42, 60);
        let accent = egui::Color32::from_rgb(44, 103, 212);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(246, 248, 252)).inner_margin(20))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(egui::RichText::new("NEOCRUNCHER  /  WORKSPACE").size(11.0).strong().color(accent));
                    ui.label(egui::RichText::new("New calculation").size(27.0).strong().color(ink));
                    ui.label(egui::RichText::new("Configure a computation, review its progress, and inspect verification.").color(muted));
                    ui.add_space(18.0);

                    ui.columns(2, |columns| {
                        egui::Frame::new().fill(egui::Color32::WHITE).stroke(border).corner_radius(12)
                            .inner_margin(18).show(&mut columns[0], |ui| {
                                ui.label(egui::RichText::new("1  Calculation").size(17.0).strong().color(ink));
                                ui.label(egui::RichText::new("Choose the mathematical task and check scope.").color(muted));
                                ui.add_space(12.0);
                                ui.add_enabled_ui(!running, |ui| {
                                    ui.label("Constant");
                                    egui::ComboBox::from_id_salt("constant")
                                        .selected_text(CONSTANTS[self.constant])
                                        .width(ui.available_width())
                                        .show_ui(ui, |ui| {
                                            for (index, name) in CONSTANTS.iter().enumerate() {
                                                ui.selectable_value(&mut self.constant, index, *name);
                                            }
                                        });
                                    ui.add_space(7.0);
                                    ui.label("Decimal digits");
                                    ui.add(egui::TextEdit::singleline(&mut self.digits).desired_width(ui.available_width()));
                                    ui.add_space(7.0);
                                    ui.label("Execution mode");
                                    egui::ComboBox::from_id_salt("mode")
                                        .selected_text(self.mode.label())
                                        .width(ui.available_width())
                                        .show_ui(ui, |ui| {
                                            for mode in [Mode::Cpu, Mode::Gpu, Mode::Hybrid] {
                                                ui.selectable_value(&mut self.mode, mode, mode.label());
                                            }
                                        });
                                    ui.add_space(7.0);
                                    ui.label("Independent check");
                                    egui::ComboBox::from_id_salt("verification")
                                        .selected_text(match self.verification {
                                            Verification::Full => "Full (up to 100,000 digits)",
                                            Verification::Prefix => "Prefix (up to 100,000 digits)",
                                            Verification::None => "None",
                                        })
                                        .width(ui.available_width())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.verification, Verification::Full, "Full (up to 100,000 digits)");
                                            ui.selectable_value(&mut self.verification, Verification::Prefix, "Prefix (up to 100,000 digits)");
                                            ui.selectable_value(&mut self.verification, Verification::None, "None");
                                        });
                                });
                            });
                        egui::Frame::new().fill(egui::Color32::WHITE).stroke(border).corner_radius(12)
                            .inner_margin(18).show(&mut columns[1], |ui| {
                                ui.label(egui::RichText::new("2  Files and runtime").size(17.0).strong().color(ink));
                                ui.label(egui::RichText::new("Results use new files; existing reports are protected.").color(muted));
                                ui.add_space(12.0);
                                ui.add_enabled_ui(!running, |ui| {
                                    ui.label("Output file (.txt)");
                                    ui.add(egui::TextEdit::singleline(&mut self.output).desired_width(ui.available_width()));
                                    ui.label(egui::RichText::new("Enter an absolute path in an existing directory.").small().color(muted));
                                    ui.add_space(14.0);
                                    if packaged_cli_path().is_some() {
                                        ui.label("CLI runtime");
                                        ui.label(egui::RichText::new("NeoCruncher CLI included with this package.").color(muted));
                                    } else {
                                        ui.label("Installed release directory");
                                        ui.add(egui::TextEdit::singleline(&mut self.bundle).desired_width(ui.available_width()));
                                        ui.label(egui::RichText::new("Select the extracted 1.0 bundle with its Python environment.").small().color(muted));
                                    }
                                });
                                ui.add_space(14.0);
                                ui.label(egui::RichText::new("Compatibility").strong().color(ink));
                                ui.label(egui::RichText::new("GPU and hybrid paths require a working OpenCL installation. The CLI checks the chosen route before computing.").color(muted));
                            });
                    });
                    ui.add_space(14.0);

                    egui::Frame::new().fill(egui::Color32::WHITE).stroke(border).corner_radius(12)
                        .inner_margin(18).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Run and verification").size(17.0).strong().color(ink));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if running {
                                        if ui.button("Cancel calculation").clicked() {
                                            if let Some(cancel) = &self.cancel { let _ = cancel.send(()); }
                                            self.status = "Cancellation requested".into();
                                        }
                                    } else if ui.add(egui::Button::new(egui::RichText::new("Start calculation").color(egui::Color32::WHITE)).fill(accent)).clicked() {
                                        self.start();
                                    }
                                });
                            });
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Status: {}", self.status)).strong().color(ink));
                            if let Some(value) = self.progress {
                                ui.add(egui::ProgressBar::new(value).show_percentage().text(if running { "Current stage" } else { "Complete" }));
                            } else if running {
                                ui.horizontal(|ui| { ui.spinner(); ui.label("Current stage has no percentage estimate"); });
                            }
                            let verify_color = if self.verification_result.starts_with("PASS") {
                                egui::Color32::from_rgb(22, 124, 83)
                            } else if self.verification_result.starts_with("FAIL") {
                                egui::Color32::from_rgb(184, 61, 61)
                            } else { muted };
                            ui.label(egui::RichText::new(format!("Independent verification: {}", self.verification_result)).color(verify_color));
                            ui.label(egui::RichText::new("A prefix check covers only its stated prefix.").small().color(muted));
                        });
                    ui.add_space(14.0);

                    egui::Frame::new().fill(egui::Color32::WHITE).stroke(border).corner_radius(12)
                        .inner_margin(18).show(ui, |ui| {
                            ui.label(egui::RichText::new("CLI activity").size(17.0).strong().color(ink));
                            if self.log.is_empty() {
                                ui.label(egui::RichText::new("Engine output will appear here after a run starts.").color(muted));
                            } else {
                                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                                    for line in &self.log {
                                        ui.label(egui::RichText::new(line).monospace().size(12.0).color(ink));
                                    }
                                });
                            }
                        });
                });
            });
        if running {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn stream_lines<R: Read + Send + 'static>(
    input: R,
    sender: Sender<WorkerEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(input).lines() {
            match line {
                Ok(text) => {
                    let _ = sender.send(WorkerEvent::Log(text));
                }
                Err(_) => break,
            }
        }
    })
}

/// Consume only complete JSONL records, retaining an incomplete trailing record.
fn read_progress(file: &mut File, position: &mut u64, sender: &Sender<WorkerEvent>) {
    if file.seek(SeekFrom::Start(*position)).is_err() {
        return;
    }
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return;
    }
    // The producer writes one JSON object and newline per event. Keep partial last writes.
    let complete = bytes
        .iter()
        .rposition(|b| *b == b'\n')
        .map(|n| n + 1)
        .unwrap_or(0);
    for line in bytes[..complete]
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
    {
        if let Ok(event) = serde_json::from_slice::<Value>(line) {
            let stage = event
                .get("stage")
                .and_then(Value::as_str)
                .unwrap_or("Computing")
                .to_owned();
            let percent = event
                .get("percent")
                .or_else(|| event.get("phase_percent"))
                .and_then(Value::as_f64)
                .map(|value| (value as f32 / 100.0).clamp(0.0, 1.0));
            let _ = sender.send(WorkerEvent::Progress { stage, percent });
        }
    }
    *position += complete as u64;
}

fn report_verification(output: &Path) -> String {
    let report = PathBuf::from(format!("{}.json", output.display()));
    fs::read_to_string(report)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| {
            value
                .get("independent_verification")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Not reported by CLI".into())
}

/// Own the CLI process, progress polling, cancellation, and final check result.
fn run_job(job: Job, sender: Sender<WorkerEvent>, cancel: Receiver<()>) {
    let temp = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(error) => {
            let _ = sender.send(WorkerEvent::Finished {
                success: false,
                exit: error.to_string(),
                verification: "Not run".into(),
            });
            return;
        }
    };
    let progress_path = temp.path().join("progress.jsonl");
    let mut progress_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&progress_path)
    {
        Ok(file) => file,
        Err(error) => {
            let _ = sender.send(WorkerEvent::Finished {
                success: false,
                exit: error.to_string(),
                verification: "Not run".into(),
            });
            return;
        }
    };
    let mut command = if let Some(cli) = &job.packaged_cli {
        Command::new(cli)
    } else {
        let mut command = Command::new("sh");
        command.arg(job.bundle.join("run_neocruncher.sh"));
        command
    };
    command
        .args(job.cli_args(&progress_path))
        .current_dir(if job.packaged_cli.is_some() {
            job.output.parent().unwrap_or(Path::new("."))
        } else {
            &job.bundle
        })
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // The GUI collects both streams; keep the packaged CLI console hidden.
        command.creation_flags(0x0800_0000);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = sender.send(WorkerEvent::Finished {
                success: false,
                exit: error.to_string(),
                verification: "Not run".into(),
            });
            return;
        }
    };
    let readers: Vec<_> = [
        child
            .stdout
            .take()
            .map(|pipe| stream_lines(pipe, sender.clone())),
        child
            .stderr
            .take()
            .map(|pipe| stream_lines(pipe, sender.clone())),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut position = 0;
    let mut cancellation_started = None;
    loop {
        read_progress(&mut progress_file, &mut position, &sender);
        if cancel.try_recv().is_ok() && cancellation_started.is_none() {
            #[cfg(unix)]
            unsafe {
                // The Unix launcher execs Python, so this PID receives SIGINT.
                libc::kill(child.id() as libc::pid_t, libc::SIGINT);
            }
            #[cfg(windows)]
            {
                // Windows has no Unix SIGINT for a hidden child process.
                // Terminate the frozen CLI immediately; result files may be partial.
                let _ = child.kill();
            }
            cancellation_started = Some(Instant::now());
            let _ = sender.send(WorkerEvent::Log("Cancellation signal sent".into()));
        }
        if cancellation_started.is_some_and(|at: Instant| at.elapsed() > Duration::from_secs(5)) {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(exit)) => {
                read_progress(&mut progress_file, &mut position, &sender);
                for reader in readers {
                    let _ = reader.join();
                }
                let verification = report_verification(&job.output);
                let check_passed = match job.verification {
                    Verification::None => true,
                    Verification::Full => verification.starts_with("PASS:"),
                    Verification::Prefix => verification.starts_with("PASS_PREFIX:"),
                };
                let success = exit.success() && cancellation_started.is_none() && check_passed;
                let exit_text = if exit.success() && !check_passed {
                    "independent verification did not pass".into()
                } else {
                    exit.to_string()
                };
                let _ = sender.send(WorkerEvent::Finished {
                    success,
                    exit: exit_text,
                    verification,
                });
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                let _ = sender.send(WorkerEvent::Finished {
                    success: false,
                    exit: error.to_string(),
                    verification: report_verification(&job.output),
                });
                break;
            }
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([940.0, 720.0])
            .with_min_inner_size([760.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "NeoCruncher",
        options,
        Box::new(|_| Ok(Box::new(NeoApp::default()))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builds_cli_arguments_without_shell_interpolation() {
        let job = Job {
            bundle: PathBuf::from("/tmp/bundle"),
            packaged_cli: None,
            constant: "pi".into(),
            digits: 1000,
            mode: Mode::Cpu,
            verification: Verification::Full,
            output: PathBuf::from("/tmp/result with spaces.txt"),
        };
        let args = job.cli_args(Path::new("/tmp/events.jsonl"));
        assert_eq!(args[7], OsString::from("/tmp/result with spaces.txt"));
        assert_eq!(args.last(), Some(&OsString::from("--verify")));
    }
    #[test]
    fn validates_full_verification_limit() {
        let mut app = NeoApp::default();
        app.digits = "100001".into();
        assert!(matches!(app.validated_job(), Err(message) if message.contains("100,000")));
    }
    #[test]
    fn rejects_existing_or_relative_output() {
        let mut app = NeoApp::default();
        app.output = "result.txt".into();
        assert!(matches!(app.validated_job(), Err(message) if message.contains("absolute")));
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("result.txt");
        File::create(&output).unwrap();
        app.output = output.display().to_string();
        assert!(matches!(app.validated_job(), Err(message) if message.contains("already exists")));
    }
    #[test]
    fn updates_visible_job_state_from_worker_events() {
        let mut app = NeoApp::default();
        let (tx, rx) = mpsc::channel();
        let (cancel_tx, _cancel_rx) = mpsc::channel();
        app.events = Some(rx);
        app.cancel = Some(cancel_tx);
        tx.send(WorkerEvent::Progress {
            stage: "Calculating".into(),
            percent: Some(0.25),
        })
        .unwrap();
        app.receive_events();
        assert_eq!(app.status, "Calculating");
        assert_eq!(app.progress, Some(0.25));
        tx.send(WorkerEvent::Finished {
            success: true,
            exit: "exit status: 0".into(),
            verification: "PASS: checked".into(),
        })
        .unwrap();
        app.receive_events();
        assert_eq!(app.status, "Completed");
        assert_eq!(app.verification_result, "PASS: checked");
        assert!(app.events.is_none());
    }
    #[test]
    fn runs_installed_cli_when_requested() {
        let Ok(bundle) = std::env::var("NEOCRUNCHER_TEST_BUNDLE") else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("pi.txt");
        let job = Job {
            bundle: PathBuf::from(bundle),
            packaged_cli: None,
            constant: "pi".into(),
            digits: 1000,
            mode: Mode::Cpu,
            verification: Verification::Full,
            output: output.clone(),
        };
        let (tx, rx) = mpsc::channel();
        let (_cancel_tx, cancel_rx) = mpsc::channel();
        run_job(job, tx, cancel_rx);
        let events: Vec<_> = rx.try_iter().collect();
        assert!(events
            .iter()
            .any(|event| matches!(event, WorkerEvent::Progress { .. })));
        assert!(events.iter().any(|event| matches!(event,
            WorkerEvent::Finished { success: true, verification, .. } if verification.starts_with("PASS:"))));
        assert!(output.exists());
    }
    #[test]
    fn accepts_cli_prefix_verification_when_requested() {
        let Ok(bundle) = std::env::var("NEOCRUNCHER_TEST_BUNDLE") else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let job = Job {
            bundle: PathBuf::from(bundle),
            packaged_cli: None,
            constant: "pi".into(),
            digits: 1000,
            mode: Mode::Cpu,
            verification: Verification::Prefix,
            output: dir.path().join("pi-prefix.txt"),
        };
        let (tx, rx) = mpsc::channel();
        let (_cancel_tx, cancel_rx) = mpsc::channel();
        run_job(job, tx, cancel_rx);
        assert!(rx.try_iter().any(|event| matches!(event,
            WorkerEvent::Finished { success: true, verification, .. } if verification.starts_with("PASS_PREFIX:"))));
    }
    #[test]
    fn runs_frozen_cli_with_full_verification_when_requested() {
        let Ok(cli) = std::env::var("NEOCRUNCHER_TEST_CLI") else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("pi-packaged.txt");
        let job = Job {
            bundle: dir.path().to_path_buf(),
            packaged_cli: Some(PathBuf::from(cli)),
            constant: "pi".into(),
            digits: 1000,
            mode: Mode::Cpu,
            verification: Verification::Full,
            output: output.clone(),
        };
        let (tx, rx) = mpsc::channel();
        let (_cancel_tx, cancel_rx) = mpsc::channel();
        run_job(job, tx, cancel_rx);
        let events: Vec<_> = rx.try_iter().collect();
        assert!(events.iter().any(|event| matches!(event,
            WorkerEvent::Finished { success: true, verification, .. } if verification.starts_with("PASS:"))));
        assert!(output.exists());
        assert!(PathBuf::from(format!("{}.json", output.display())).exists());
    }
}
