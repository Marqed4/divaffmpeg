use std::process::Command;

use ratatui_textarea::TextArea;

use super::shared::{default_output_path, new_path_field, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState};

//                      <-- RESIZE FIELDS (scale to a new width/height, re-encodes) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeField { InputPath, OutputPath, Width, Height, Lock, Run }

impl FieldSet for ResizeField {
    // Order here is the tab order in the UI.
    const ALL: &'static [ResizeField] = &[
        ResizeField::InputPath, ResizeField::OutputPath, ResizeField::Width,
        ResizeField::Height, ResizeField::Lock, ResizeField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Width => "📏 width", Self::Height => "📏 height",
            Self::Lock => "🔗 ratio", Self::Run => "▶ resize",
        }
    }
}

pub type ResizeMenuState = MenuState<ResizeField>;

//                      <-- RESIZE SCREEN STATE (input, dimensions, export) -->

pub struct ResizeState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub width: TextArea<'static>,
    pub height: TextArea<'static>,
    pub menu: ResizeMenuState,
    /// When true, editing width or height recalculates the other so the source's
    /// aspect ratio is kept.
    lock_aspect: bool,
    /// The input's real (width, height), learned from `sync_dimensions`.
    source_dims: Option<(u32, u32)>,
    /// The running (or finished) ffmpeg job, if any.
    job: Option<FfmpegJob>,
}

/// Reads the first video stream's size with ffprobe (ships with ffmpeg).
/// Uses the bundled ffprobe via `ffprobe_binary` rather than PATH, so it works
/// standalone. Returns None if the file can't be probed (bad path, no video stream).
fn probe_dimensions(path: &str) -> Option<(u32, u32)> {
    let out = Command::new(super::shared::ffprobe_binary())
        .args([
            "-v", "error", "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0", path,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let (w, h) = text.trim().split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

fn parse_dim(field: &TextArea<'static>) -> Option<u32> {
    field.lines().join("").trim_end_matches("px").trim().parse().ok()
}

fn set_text(field: &mut TextArea<'static>, value: String) {
    *field = TextArea::new(vec![value]);
    field.move_cursor(ratatui_textarea::CursorMove::End);
}

/// Nearest even number, minimum 2. Encoders like H.264 reject odd dimensions.
fn round_even(n: f64) -> u32 {
    ((n / 2.0).round() as u32 * 2).max(2)
}

impl ResizeState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/video.mov"),
            output_file_path: new_path_field("(optional) C:/Users/you/output.mov"),
            width: new_path_field("1280px"),
            height: new_path_field("720px"),
            menu: ResizeMenuState::new(),
            lock_aspect: true,
            source_dims: None,
            job: None,
        }
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    /// Advance the running job's state; call this once per UI tick.
    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Text shown in the lock toggle.
    pub fn lock_label(&self) -> &'static str {
        if self.lock_aspect { "locked" } else { "free" }
    }

    /// Flips the lock; turning it on snaps height to match the current width.
    pub fn toggle_lock(&mut self) {
        self.lock_aspect = !self.lock_aspect;
        if self.lock_aspect {
            self.apply_lock(ResizeField::Width);
        }
    }

    /// Probes the input and fills width/height with its real size.
    /// Leaves the fields alone if the file can't be probed.
    pub fn sync_dimensions(&mut self) {
        let input = super::shared::strip_quotes(&self.input_file_path.lines().join(""));
        self.source_dims = probe_dimensions(&input);
        if let Some((w, h)) = self.source_dims {
            set_text(&mut self.width, w.to_string());
            set_text(&mut self.height, h.to_string());
        }
    }

    /// With the lock on, rewrites the other dimension from the one just edited
    /// (pass `Width` or `Height`). Call when editing of that field finishes.
    pub fn apply_lock(&mut self, edited: ResizeField) {
        if !self.lock_aspect {
            return;
        }
        let Some((src_w, src_h)) = self.source_dims else { return };
        match edited {
            ResizeField::Width => {
                let Some(w) = parse_dim(&self.width) else { return };
                let h = round_even(w as f64 * src_h as f64 / src_w as f64);
                set_text(&mut self.height, h.to_string());
            }
            ResizeField::Height => {
                let Some(h) = parse_dim(&self.height) else { return };
                let w = round_even(h as f64 * src_w as f64 / src_h as f64);
                set_text(&mut self.width, w.to_string());
            }
            _ => {}
        }
    }

    /// Scales the input to `width` x `height` with ffmpeg's `scale` filter.
    /// Unlike trim, this re-encodes the video, so it's slower and uses default codecs.
    /// Does nothing if the input, width, or height is empty.
    pub fn start_resize(&mut self, log_path: &str) {
        let input = super::shared::strip_quotes(&self.input_file_path.lines().join(""));
        // Strip a trailing "px" so "1280px" and "1280" both work; ffmpeg wants bare numbers.
        let width = self.width.lines().join("").trim_end_matches("px").trim().to_string();
        let height = self.height.lines().join("").trim_end_matches("px").trim().to_string();
        if input.is_empty() || width.is_empty() || height.is_empty() {
            return;
        }

        // Fall back to "<input name>_resized.<ext>" when no output path is typed.
        let typed_output = super::shared::strip_quotes(&self.output_file_path.lines().join(""));
        let output = if typed_output.is_empty() {
            default_output_path(&input, "resized")
        } else {
            typed_output
        };

        // -y overwrites an existing output; -vf scale=W:H does the actual resize.
        let args: Vec<String> = vec![
            "-y".into(), "-i".into(), input,
            "-vf".into(), format!("scale={width}:{height}"),
            output,
        ];

        self.job = Some(spawn_ffmpeg_job(args, log_path, "resize"));
    }
}