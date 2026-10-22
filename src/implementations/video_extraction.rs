use ratatui_textarea::TextArea;

use super::shared::{cycle, format_timestamp, new_path_field, default_output_path, parse_timestamp,
    probe_duration, strip_quotes, spawn_ffmpeg_job, ExtractionFps, FfmpegJob, FieldSet, MenuState
};

//                      <-- EXTRACTION-ONLY OPTION TYPES -->
// Extraction only ever pulls still frames out of a video, so the format list
// is image formats ffmpeg can mux frames into, not video containers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionFormat { Jpg, Png, Bmp, Tiff, Webp, Gif }

impl ExtractionFormat {
    const ALL: [ExtractionFormat; 6] = [
        ExtractionFormat::Jpg, ExtractionFormat::Png, ExtractionFormat::Bmp,
        ExtractionFormat::Tiff, ExtractionFormat::Webp, ExtractionFormat::Gif,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Jpg => "🎬 .jpg", Self::Png => "🌐 .png", Self::Bmp => ".bmp",
            Self::Tiff => ".tiff", Self::Webp => "📱 .webp", Self::Gif => "🖼 .gif",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Jpg => "jpg", Self::Png => "png", Self::Bmp => "bmp",
            Self::Tiff => "tiff", Self::Webp => "webp", Self::Gif => "gif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoExtractionField { Input, Start, End, Fps, Format, OutputPath, Run }

impl FieldSet for VideoExtractionField {
    const ALL: &'static [VideoExtractionField] = &[
        VideoExtractionField::Input, VideoExtractionField::OutputPath, VideoExtractionField::Start,
        VideoExtractionField::End, VideoExtractionField::Format, VideoExtractionField::Fps, VideoExtractionField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Input => "📂 clip", Self::OutputPath => "💾 output",
            Self::Start => "⏱ start", Self::End => "⏱ end",
            Self::Fps => "fps", Self::Format => "format", Self::Run => "▶ extract",
        }
    }
}

pub type ExtractionMenuState = MenuState<VideoExtractionField>;

//                      <-- EXTRACT SCREEN STATE () -->

pub struct VideoExtractionState {
    pub input_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub start_time: TextArea<'static>,
    pub end_time: TextArea<'static>,
    pub menu: ExtractionMenuState,
    pub fps: ExtractionFps,
    pub format: ExtractionFormat,
    /// The clip's real length, learned from `sync_duration`. `end_time` is
    /// clamped to this so a typed/leftover end can never point past the clip.
    duration: Option<f64>,
    job: Option<FfmpegJob>,
}

// Find video length from 'input_path' so that the user doesn't have to manually recognize the length
// of the video they provided. Implemented as `sync_duration` below: probes the clip with
// ffmpeg, sets `end_time` to that length, and `start_extraction` clamps to it so `end_time`
// can never point past the clip even if the user later types something longer.
impl VideoExtractionState {
    pub fn new() -> Self {
        Self {
            input_path: new_path_field("C:/Users/you/video.mov"),
            output_file_path: new_path_field("C:/Users/you/frame_0001.jpg"),
            start_time: new_path_field("00:00:00"),
            end_time: new_path_field("00:00:10"),
            menu: ExtractionMenuState::new(),
            fps: ExtractionFps::EveryFrame,
            format: ExtractionFormat::Jpg,
            duration: None,
            job: None,
        }
    }

    /// Probes the input clip's real length and sets `end_time` to it, so the
    /// field starts at the furthest end point actually reachable in this clip.
    /// Called once the user finishes typing/confirming the input path.
    pub fn sync_duration(&mut self) {
        let input = strip_quotes(&self.input_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        if let Some(duration) = probe_duration(&input) {
            self.duration = Some(duration);
            self.end_time = TextArea::new(vec![format_timestamp(duration)]);
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            VideoExtractionField::Fps => self.fps = cycle(&ExtractionFps::ALL, self.fps, forward),
            VideoExtractionField::Format => self.format = cycle(&ExtractionFormat::ALL, self.format, forward),
            VideoExtractionField::Input | VideoExtractionField::Start | VideoExtractionField::End
                | VideoExtractionField::OutputPath | VideoExtractionField::Run => {},
        }
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    pub fn start_extraction(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        let start = self.start_time.lines().join("");
        let mut end = self.end_time.lines().join("");

        // Never let a typed (or stale) end point past the clip's real length.
        if let Some(duration) = self.duration {
            if !end.trim().is_empty() && parse_timestamp(&end).is_none_or(|secs| secs > duration) {
                end = format_timestamp(duration);
            }
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let mut output = if typed_output.trim().is_empty() {
            default_output_path(&input, "frame")
        } else {
            typed_output
        };

        // Swap in the chosen format's extension, replacing whatever the user typed (if any).
        if let Some(dot) = output.rfind('.') {
            output.truncate(dot);
        }
        output.push('.');
        output.push_str(self.format.extension());

        let mut args: Vec<String> = vec!["-y".into()];
        if !start.trim().is_empty() {
            args.push("-ss".into());
            args.push(start);
        }
        args.push("-i".into());
        args.push(input);
        if !end.trim().is_empty() {
            args.push("-to".into());
            args.push(end);
        }
        if let Some(fps) = self.fps.value() {
            args.push("-r".into());
            args.push(fps.to_string());
        }
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "extraction"));
    }
}
