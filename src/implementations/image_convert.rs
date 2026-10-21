use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};

//                      <-- IMAGE-WIDE OPTION TYPES -->
// Same arrangement as audio_convert.rs: Convert is the screen that uses every
// image option type, so it owns them and the other image screens borrow them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat { Png, Jpg, Webp, Bmp, Tiff, Gif }

impl ImageFormat {
    pub const ALL: [ImageFormat; 6] = [
        ImageFormat::Png, ImageFormat::Jpg, ImageFormat::Webp,
        ImageFormat::Bmp, ImageFormat::Tiff, ImageFormat::Gif,
    ];

    /// Compress never changes how lossless a file is, so only the two formats
    /// with a real quality dial make sense there.
    pub const COMPRESS_ALL: [ImageFormat; 2] = [ImageFormat::Jpg, ImageFormat::Webp];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Png => "💎 .png", Self::Jpg => "📷 .jpg", Self::Webp => "🌐 .webp",
            Self::Bmp => ".bmp", Self::Tiff => ".tiff", Self::Gif => "🖼 .gif",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png", Self::Jpg => "jpg", Self::Webp => "webp",
            Self::Bmp => "bmp", Self::Tiff => "tiff", Self::Gif => "gif",
        }
    }

    /// Only JPG and WebP have a quality dial. PNG, BMP, TIFF and GIF never lose
    /// detail to a quality setting, so every screen checks this before pushing one.
    pub fn has_quality(&self) -> bool {
        matches!(self, Self::Jpg | Self::Webp)
    }
}

// Quality is 1 (smallest, roughest) to 100 (biggest, cleanest) in steps of five,
// the scale people already know from image editors, rather than the encoder's
// own inverted scales: JPG's -q:v runs 2 (best) to 31 (worst), WebP's runs 0-100.
pub const QUALITY_MIN: u8 = 1;
pub const QUALITY_MAX: u8 = 100;
const QUALITY_STEP: u8 = 5;

/// Plain-language read of a quality value, shown next to the slider.
pub fn quality_label(q: u8) -> &'static str {
    match q {
        0..=29 => "🪨 tiny file, visibly rough",
        30..=59 => "📉 small file, some softness",
        60..=84 => "⚖ good balance",
        _ => "✨ near-lossless, big file",
    }
}

pub fn quality_ratio(q: u8) -> f64 {
    (q.saturating_sub(QUALITY_MIN) as f64) / ((QUALITY_MAX - QUALITY_MIN) as f64)
}

/// Steps a quality value one notch, clamped to the slider's range.
pub fn step_quality(q: u8, forward: bool) -> u8 {
    if forward {
        q.saturating_add(QUALITY_STEP).min(QUALITY_MAX)
    } else {
        q.saturating_sub(QUALITY_STEP).max(QUALITY_MIN)
    }
}

/// Pushes the encoder's quality flag for the chosen format, translating the
/// 1-100 slider into whatever scale that encoder expects. Formats without a
/// quality dial get nothing.
pub fn push_quality_args(args: &mut Vec<String>, format: ImageFormat, quality: u8) {
    match format {
        ImageFormat::Jpg => {
            // 100 -> 2 (best), 1 -> 31 (worst).
            let q = quality.clamp(QUALITY_MIN, QUALITY_MAX) as f64;
            let qv = (31.0 - (q - 1.0) * 29.0 / 99.0).round() as u32;
            args.push("-q:v".into());
            args.push(qv.clamp(2, 31).to_string());
        },
        ImageFormat::Webp => {
            args.push("-quality".into());
            args.push(quality.to_string());
        },
        ImageFormat::Png | ImageFormat::Bmp | ImageFormat::Tiff | ImageFormat::Gif => {},
    }
}

/// For screens that keep the input's format (Resize, Crop, Rotate, Filter): without
/// this, ffmpeg re-saves a JPG or WebP at its stock quality, which is noticeably
/// softer than most source files. Picks a high setting by looking at the output's extension.
pub fn push_keep_quality_args(args: &mut Vec<String>, output: &str) {
    let lower = output.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        args.push("-q:v".into());
        args.push("2".into());
    } else if lower.ends_with(".webp") {
        args.push("-quality".into());
        args.push("95".into());
    }
}

/// Swaps whatever extension the user typed for the chosen format's own, so the
/// container and the filename can never disagree about what the file is.
pub fn force_image_extension(output: &mut String, format: ImageFormat) {
    let name_start = output.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    if let Some(dot) = output.rfind('.') {
        if dot > name_start {
            output.truncate(dot);
        }
    }
    output.push('.');
    output.push_str(format.extension());
}

/// The output path the user typed, or a `name_suffix.ext` sibling of the input if
/// they left it blank.
pub fn resolve_output(input: &str, typed: String, suffix: &str) -> String {
    if typed.trim().is_empty() {
        default_output_path(input, suffix)
    } else {
        typed
    }
}

//                      <-- IMAGE CONVERT FIELDS (format + quality only) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageConvertField { InputPath, OutputPath, Format, Quality, Run }

impl FieldSet for ImageConvertField {
    const ALL: &'static [ImageConvertField] = &[
        ImageConvertField::InputPath, ImageConvertField::OutputPath,
        ImageConvertField::Format, ImageConvertField::Quality, ImageConvertField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Format => "format", Self::Quality => "quality",
            Self::Run => "▶ convert",
        }
    }
}

pub type ImageConvertMenuState = MenuState<ImageConvertField>;

//                      <-- IMAGE CONVERT SCREEN STATE -->
pub struct ImageConvertState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ImageConvertMenuState,
    pub format: ImageFormat,
    pub quality: u8,
    job: Option<FfmpegJob>,
}

impl ImageConvertState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("C:/Users/you/photo.jpg"),
            menu: ImageConvertMenuState::new(),
            format: ImageFormat::Jpg,
            quality: 90,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ImageConvertField::Format => self.format = cycle(&ImageFormat::ALL, self.format, forward),
            ImageConvertField::Quality => self.quality = step_quality(self.quality, forward),
            ImageConvertField::InputPath | ImageConvertField::OutputPath | ImageConvertField::Run => {},
        }
    }

    pub fn quality_text(&self) -> String {
        format!("{}%", self.quality)
    }

    /// True while the quality slider is pointed at a format that has no such dial,
    /// so the screen can say "n/a" instead of showing a number that never reaches ffmpeg.
    pub fn quality_ignored(&self) -> bool {
        !self.format.has_quality()
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    pub fn start_convert(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        let mut output = strip_quotes(&self.output_file_path.lines().join(""));
        if input.trim().is_empty() || output.trim().is_empty() {
            return;
        }

        force_image_extension(&mut output, self.format);

        // -frames:v 1 writes exactly one picture, which is also what stops ffmpeg
        // complaining that the output name isn't an image sequence pattern.
        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input, "-frames:v".into(), "1".into()];
        push_quality_args(&mut args, self.format, self.quality);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image convert"));
    }
}
