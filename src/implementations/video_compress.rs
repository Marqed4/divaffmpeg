use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, strip_quotes, spawn_ffmpeg_job,
    ConvertCodec, ConvertFps, FfmpegJob, FieldSet, MenuState,
};

//                      <-- COMPRESS FIELDS (shrink file size, same container) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCompressField { InputPath, OutputPath, Codec, Fps, Crf, Run }

impl FieldSet for VideoCompressField {
    const ALL: &'static [VideoCompressField] = &[
        VideoCompressField::InputPath, VideoCompressField::OutputPath, VideoCompressField::Codec,
        VideoCompressField::Fps, VideoCompressField::Crf, VideoCompressField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Codec => "codec", Self::Fps => "fps", Self::Crf => "compression",
            Self::Run => "▶ compress",
        }
    }
}

pub type CompressMenuState = MenuState<VideoCompressField>;

// CRF (Constant Rate Factor) is libx264/libx265's actual quality dial: 0 is
// near-lossless, 51 is the worst ffmpeg allows. Compress exposes the real
// range as a slider instead of a handful of preset names, so the user gets
// real control over the size/quality tradeoff instead of picking a label.
pub const CRF_MIN: u8 = 0;
pub const CRF_MAX: u8 = 51;
const CRF_STEP: u8 = 1;

/// Plain-language read of a CRF value, shown next to the slider so a number
/// alone isn't the only thing telling the user what they're picking.
pub fn crf_quality_label(value: u8) -> &'static str {
    match value {
        0..=17 => "✨ near-lossless, big files",
        18..=22 => "✨ excellent quality",
        23..=27 => "💫 great balance",
        28..=32 => "📦 good, smaller files",
        33..=39 => "💩 visible quality loss",
        _ => "🤡 heavy quality loss, tiny files",
    }
}

/// How full the slider bar should look: low CRF (best quality) fills it, high
/// CRF (most compression) empties it.
pub fn crf_ratio(value: u8) -> f64 {
    1.0 - (value.saturating_sub(CRF_MIN) as f64 / (CRF_MAX - CRF_MIN) as f64)
}

//                      <-- COMPRESS SCREEN STATE (shrink size, same container) -->
pub struct VideoCompressState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: CompressMenuState,
    pub codec: ConvertCodec,
    pub fps: ConvertFps,
    pub crf: u8,
    job: Option<FfmpegJob>,
}

impl VideoCompressState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/video.mov"),
            output_file_path: new_path_field("(optional) C:/Users/you/output.mov"),
            menu: CompressMenuState::new(),
            codec: ConvertCodec::H264,
            fps: ConvertFps::Original,
            crf: 27,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            VideoCompressField::Codec => self.codec = cycle(&ConvertCodec::COMPRESS_ALL, self.codec, forward),
            VideoCompressField::Fps => self.fps = cycle(&ConvertFps::ALL, self.fps, forward),
            VideoCompressField::Crf => {
                self.crf = if forward {
                    (self.crf + CRF_STEP).min(CRF_MAX)
                } else {
                    self.crf.saturating_sub(CRF_STEP).max(CRF_MIN)
                };
            },
            VideoCompressField::InputPath | VideoCompressField::OutputPath | VideoCompressField::Run => {},
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

    /// Compress never touches the container or resolution: it only re-encodes at a
    /// lower quality/bitrate, keeping the input's extension unless the user overrides it.
    pub fn start_compress(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let output = if typed_output.trim().is_empty() {
            default_output_path(&input, "compressed")
        } else {
            typed_output
        };

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input];

        if let Some(fps) = self.fps.value() {
            args.push("-r".into());
            args.push(fps.to_string());
        }
        args.push("-c:v".into());
        args.push(self.codec.ffmpeg_flag().into());
        args.push("-crf".into());
        args.push(self.crf.to_string());
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "compress"));
    }
}
