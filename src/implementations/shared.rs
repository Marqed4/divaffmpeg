use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use chrono::Utc;

use ratatui_textarea::TextArea;

/// Resolve ffmpeg binary: bundled copy next to the exe wins over PATH,
/// so the app works standalone once shipped with an `ffmpeg` folder alongside it.
/// Resolve ffprobe binary: it ships in the same folder as ffmpeg, so reuse
/// whichever folder `ffmpeg_binary` picked. Without this, a machine that only has
/// the bundled ffmpeg (nothing on PATH) can't probe, and the resize fields never auto-fill.
/// Falls back to plain "ffprobe" on PATH if no bundled copy is found.
pub fn ffprobe_binary() -> PathBuf {
    let ffmpeg = ffmpeg_binary();
    if let Some(dir) = ffmpeg.parent() {
        let probe = dir.join("ffprobe.exe");
        if probe.is_file() {
            return probe;
        }
    }
    PathBuf::from("ffprobe")
}

pub fn ffmpeg_binary() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidates = [
                dir.join("ffmpeg.exe"),
                dir.join("ffmpeg").join("bin").join("ffmpeg.exe"),
                dir.join("ffmpeg").join("ffmpeg.exe"),
            ];
            for candidate in candidates {
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    #[cfg(debug_assertions)]
    {
        let dev_bin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("ffmpeg")
            .join("bin")
            .join("ffmpeg.exe");
        if dev_bin.is_file() {
            return dev_bin;
        }
    }
    PathBuf::from("ffmpeg")
}

//                      <-- GENERIC FIELD / MENU MACHINERY -->
// Every screen (Convert/Compress/Trim/Merge) has its own field enum and its
// own set of options, but they all need the same "focus one of N fields,
// cycle a value, edit a path" bookkeeping. This trait + generic state is
// what keeps that bookkeeping written exactly once.
pub trait FieldSet: Copy + PartialEq + 'static {
    const ALL: &'static [Self];
    fn label(&self) -> &'static str;
}

#[derive(Debug)]
pub struct MenuState<F: FieldSet> {
    focus: usize,
    pub editing: bool,
    _marker: PhantomData<F>,
}

impl<F: FieldSet> MenuState<F> {
    pub fn new() -> Self {
        Self { focus: 0, editing: false, _marker: PhantomData }
    }

    pub fn focus_field(&self) -> F {
        F::ALL[self.focus]
    }

    pub fn next(&mut self) {
        self.focus = (self.focus + 1) % F::ALL.len();
    }

    pub fn previous(&mut self) {
        self.focus = if self.focus == 0 { F::ALL.len() - 1 } else { self.focus - 1 };
    }
}

pub fn cycle<T: PartialEq + Copy>(all: &[T], current: T, forward: bool) -> T {
    let idx = all.iter().position(|v| *v == current).unwrap_or(0);
    let next_idx = if forward {
        (idx + 1) % all.len()
    } else if idx == 0 {
        all.len() - 1
    } else {
        idx - 1
    };
    all[next_idx]
}

//                      <-- FFMPEG JOB (shared by every screen's background process) -->
enum JobMsg {
    Duration(f64),
    Time(f64),
    Done(Result<(), String>),
}

pub struct FfmpegJob {
    rx: Receiver<JobMsg>,
    duration: f64,
    elapsed: f64,
    finished: Option<Result<(), String>>,
}

impl FfmpegJob {
    pub fn poll(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(JobMsg::Duration(d)) => self.duration = d,
                Ok(JobMsg::Time(t)) => self.elapsed = t,
                Ok(JobMsg::Done(r)) => self.finished = Some(r),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    pub fn ratio(&self) -> f64 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else if self.finished.is_some() {
            1.0
        } else {
            0.0
        }
    }

    pub fn status_text(&self, verb: &str) -> String {
        match &self.finished {
            Some(Ok(())) => "✅ We're done here!".to_string(),
            Some(Err(e)) => format!("❌ {e}"),
            None => format!("⏳ {verb}... {:.0}%", self.ratio() * 100.0),
        }
    }

}

/// Reads a child's stderr and feeds each "line" to `on_line`. ffmpeg writes its
/// progress updates (`time=...`) terminated with `\r`, not `\n`, so splitting only
/// on `\n` (what BufRead::lines does) means every progress tick gets buffered up
/// and delivered in one lump right before the next real newline. Splitting on
/// both is what makes the progress bar actually move instead of jumping 0 to 100.
fn for_each_stderr_line(stderr: impl Read, mut on_line: impl FnMut(&str)) {
    let mut reader = std::io::BufReader::new(stderr);
    let mut line = String::new();
    let mut byte = [0u8; 1];

    loop {
        match std::io::Read::read(&mut reader, &mut byte) {
            Ok(0) => break,
            Ok(_) => {
                let c = byte[0] as char;
                if c == '\n' || c == '\r' {
                    if !line.is_empty() {
                        on_line(&line);
                        line.clear();
                    }
                } else {
                    line.push(c);
                }
            },
            Err(_) => break,
        }
    }

    if !line.is_empty() {
        on_line(&line);
    }
}

/// Spawns ffmpeg with the given args in the background, streaming progress back
/// through the returned job and appending a full run log to `log_path`.
pub fn spawn_ffmpeg_job(args: Vec<String>, log_path: &str, job_name: &str) -> FfmpegJob {
    let command_line = format!("{} {}", ffmpeg_binary().display(), args.join(" "));
    let log_path = log_path.to_string();
    let job_name = job_name.to_string();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&log_path) {
            let _ = writeln!(f, "Started ffmpeg {job_name} job at {}: {command_line}", Utc::now().format("%H-%M-%S"));
        }

        let spawned = Command::new(ffmpeg_binary())
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        let mut stderr_lines: Vec<String> = Vec::new();

        let result = match spawned {
            Ok(mut child) => {
                if let Some(stderr) = child.stderr.take() {
                    for_each_stderr_line(stderr, |line| {
                        if let Some(d) = parse_tagged_timestamp(line, "Duration: ") {
                            let _ = tx.send(JobMsg::Duration(d));
                        }
                        if let Some(t) = parse_tagged_timestamp(line, "time=") {
                            let _ = tx.send(JobMsg::Time(t));
                        }
                        stderr_lines.push(line.to_string());
                    });
                }

                match child.wait() {
                    Ok(status) if status.success() => Ok(()),
                    Ok(status) => {
                        let last_error = stderr_lines.iter().rev().find(|l| !l.trim().is_empty());
                        Err(match last_error {
                            Some(l) => format!("ffmpeg exited with {status}: {l}"),
                            None => format!("ffmpeg exited with {status}"),
                        })
                    },
                    Err(e) => Err(e.to_string()),
                }
            },
            Err(e) => Err(format!("couldn't launch ffmpeg: {e}")), // Unavailable/Absent
        };

        if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&log_path) {
            let finished_at = Utc::now().format("%H-%M-%S");
            match &result {
                Ok(()) => { let _ = writeln!(f, "ffmpeg {job_name} job finished OK at {finished_at}."); },
                Err(e) => {
                    let _ = writeln!(f, "ffmpeg {job_name} job FAILED at {finished_at}: {e}");
                    if !stderr_lines.is_empty() {
                        let _ = writeln!(f, "  full stderr:");
                        for line in &stderr_lines {
                            let _ = writeln!(f, "    {line}");
                        }
                    }
                },
            }
        }

        let _ = tx.send(JobMsg::Done(result));
    });

    FfmpegJob { rx, duration: 0.0, elapsed: 0.0, finished: None }
}

pub fn parse_timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 { return None; }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

pub fn parse_tagged_timestamp(line: &str, tag: &str) -> Option<f64> {
    let start = line.find(tag)? + tag.len();
    let rest = &line[start..];
    let end = rest.find(|c: char| c != ':' && c != '.' && !c.is_ascii_digit()).unwrap_or(rest.len());
    parse_timestamp(&rest[..end])
}

/// Formats seconds back into "HH:MM:SS", the same shape `parse_timestamp` reads.
pub fn format_timestamp(secs: f64) -> String {
    let total = secs.max(0.0).round() as u64;
    format!("{:02}:{:02}:{:02}", total / 3600, (total % 3600) / 60, total % 60)
}

/// Blocks briefly to ask ffmpeg for a clip's real length: `-i` alone with no
/// output makes ffmpeg print its "Duration: ..." metadata line and exit
/// immediately, so this is cheap enough to call right before running a job.
pub fn probe_duration(input: &str) -> Option<f64> {
    let output = Command::new(ffmpeg_binary())
        .args(["-i", input])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr.lines().find_map(|line| parse_tagged_timestamp(line, "Duration: "))
}

/// Strips a leading/trailing `"` (or `'`) pair, e.g. from Windows "Copy as path".
/// Without this the quote chars end up as literal bytes in the path arg passed to ffmpeg.
pub fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    let stripped = trimmed
        .strip_prefix('"').and_then(|s| s.strip_suffix('"'))
        .or_else(|| trimmed.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')));
    stripped.unwrap_or(trimmed).to_string()
}

/// Splits "path/to/file.ext" into ("path/to/file", "ext"); no dot means no extension.
fn split_extension(path: &str) -> (&str, Option<&str>) {
    match path.rfind('.') {
        Some(dot) if dot > path.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0) => {
            (&path[..dot], Some(&path[dot + 1..]))
        },
        _ => (path, None),
    }
}

/// Builds a default sibling output path (`name_suffix.ext`) from an input path,
/// used whenever the user leaves the output field blank.
pub fn default_output_path(input: &str, suffix: &str) -> String {
    let (stem, ext) = split_extension(input);
    match ext {
        Some(ext) => format!("{stem}_{suffix}.{ext}"),
        None => format!("{input}_{suffix}"),
    }
}

pub fn new_path_field(placeholder: &'static str) -> TextArea<'static> {
    let mut field = TextArea::default();
    field.set_placeholder_text(placeholder);
    field
}

//                      <-- VALUE TYPES SHARED ACROSS SCREENS -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCodec { H264, H265, Vp9, Gif }

impl ConvertCodec {
    /// Compress never changes container, so only the two re-encode codecs make sense.
    pub const COMPRESS_ALL: [ConvertCodec; 2] = [ConvertCodec::H264, ConvertCodec::H265];

    pub fn label(&self) -> &'static str {
        match self {
            Self::H264 => "🏃 H.264", Self::H265 => "🐌 H.265",
            Self::Vp9 => "🌐 VP9", Self::Gif => "🖼 GIF",
        }
    }

    pub fn ffmpeg_flag(&self) -> &'static str {
        match self {
            Self::H264 => "libx264", Self::H265 => "libx265",
            Self::Vp9 => "libvpx-vp9", Self::Gif => "gif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertFps { Original, Fps120, Fps60, Fps30, Fps24 }

impl ConvertFps {
    pub const ALL: [ConvertFps; 5] = [
        ConvertFps::Original, ConvertFps::Fps120, ConvertFps::Fps60, ConvertFps::Fps30, ConvertFps::Fps24,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::Fps120 => "120fps", Self::Fps60 => "60fps", Self::Fps30 => "30fps",
            Self::Fps24 => "24fps",
        }
    }

    pub fn value(&self) -> Option<u32> {
        match self {
            Self::Original => None, Self::Fps120 => Some(120), Self::Fps60 => Some(60), Self::Fps30 => Some(30), Self::Fps24 => Some(24),
        }
    }
}

/// Extraction pulls stills off a video, so its rates are frame-sampling
/// intervals (one still per N seconds), not encode fps like ConvertFps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionFps { EveryFrame, OnePerSecond, OneEvery2s, OneEvery5s, OneEvery10s }

impl ExtractionFps {
    pub const ALL: [ExtractionFps; 5] = [
        ExtractionFps::EveryFrame, ExtractionFps::OnePerSecond, ExtractionFps::OneEvery2s,
        ExtractionFps::OneEvery5s, ExtractionFps::OneEvery10s,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::EveryFrame => "🔳 every frame", Self::OnePerSecond => "1/sec",
            Self::OneEvery2s => "1/2s", Self::OneEvery5s => "1/5s", Self::OneEvery10s => "1/10s",
        }
    }

    pub fn value(&self) -> Option<f64> {
        match self {
            Self::EveryFrame => None, Self::OnePerSecond => Some(1.0), Self::OneEvery2s => Some(0.5),
            Self::OneEvery5s => Some(0.2), Self::OneEvery10s => Some(0.1),
        }
    }
}
