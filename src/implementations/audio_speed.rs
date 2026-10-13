use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};
use super::audio_convert::{force_extension, push_encode_args, AudioBitrate, AudioFormat};

//                      <-- SPEED FIELDS (stretch or squash the timeline) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSpeedField { InputPath, OutputPath, Speed, Format, Bitrate, Run }

impl FieldSet for AudioSpeedField {
    const ALL: &'static [AudioSpeedField] = &[
        AudioSpeedField::InputPath, AudioSpeedField::OutputPath, AudioSpeedField::Speed,
        AudioSpeedField::Format, AudioSpeedField::Bitrate, AudioSpeedField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Speed => "speed", Self::Format => "format",
            Self::Bitrate => "bitrate", Self::Run => "▶ respeed",
        }
    }
}

pub type AudioSpeedMenuState = MenuState<AudioSpeedField>;

// Speed is stored in hundredths rather than as an f64 so the slider can step
// through it exactly: 100 is "untouched", 50 is half speed (twice as long), 400
// is four times as fast. Floating point steps would drift off the round numbers
// the label prints after enough keypresses.
pub const SPEED_MIN: u16 = 25;
pub const SPEED_MAX: u16 = 400;
const SPEED_STEP: u16 = 5;

/// Plain-language read of a speed value, shown next to the slider so the raw
/// multiplier isn't the only thing telling the user what they're picking.
pub fn speed_label(hundredths: u16) -> &'static str {
    match hundredths {
        0..=49 => "🐌 way slower, much longer",
        50..=89 => "🐢 slowed down, stretched out",
        90..=109 => "🎯 roughly untouched",
        110..=159 => "🐇 sped up, shorter",
        _ => "🚀 chipmunk territory",
    }
}

/// How full the slider bar should look. The range is lopsided (25 below 100, 300
/// above it), so this measures position across the whole span instead of pretending
/// 100 sits in the middle.
pub fn speed_ratio(hundredths: u16) -> f64 {
    (hundredths.saturating_sub(SPEED_MIN) as f64) / ((SPEED_MAX - SPEED_MIN) as f64)
}

/// Builds the `atempo` filter string for a speed factor.
///
/// A single `atempo` only accepts a factor between 0.5 and 2.0, so anything
/// outside that gets chained: 4x becomes `atempo=2.0,atempo=2.0`, 0.25x becomes
/// `atempo=0.5,atempo=0.5`. Each stage multiplies into the last, which is why the
/// loop divides the remaining factor down as it goes. Without this, picking 4x
/// hands ffmpeg "Value 4.000000 for parameter 'tempo' out of range" and nothing runs.
///
/// Unlike the naive approach of resampling, `atempo` keeps the pitch where it was,
/// so a slowed track sounds slower rather than deeper.
pub fn atempo_chain(hundredths: u16) -> String {
    let mut remaining = hundredths as f64 / 100.0;
    let mut stages: Vec<String> = Vec::new();

    // Each pass takes the biggest bite a single atempo can handle (2.0 going up,
    // 0.5 going down), then carries whatever is left into the next stage.
    while remaining > 2.0 {
        stages.push("atempo=2.0".to_string());
        remaining /= 2.0;
    }
    while remaining < 0.5 {
        stages.push("atempo=0.5".to_string());
        remaining /= 0.5;
    }
    stages.push(format!("atempo={remaining:.4}"));

    stages.join(",")
}

//                      <-- SPEED SCREEN STATE (stretch or squash the timeline) -->
pub struct AudioSpeedState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: AudioSpeedMenuState,
    pub speed: u16,
    pub format: AudioFormat,
    pub bitrate: AudioBitrate,
    job: Option<FfmpegJob>,
}

impl AudioSpeedState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/song.mp3"),
            output_file_path: new_path_field("(optional) C:/Users/you/song_respeed.mp3"),
            menu: AudioSpeedMenuState::new(),
            speed: 100,
            format: AudioFormat::Mp3,
            bitrate: AudioBitrate::Auto,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            AudioSpeedField::Speed => {
                self.speed = if forward {
                    (self.speed + SPEED_STEP).min(SPEED_MAX)
                } else {
                    self.speed.saturating_sub(SPEED_STEP).max(SPEED_MIN)
                };
            },
            AudioSpeedField::Format => self.format = cycle(&AudioFormat::ALL, self.format, forward),
            AudioSpeedField::Bitrate => self.bitrate = cycle(&AudioBitrate::ALL, self.bitrate, forward),
            AudioSpeedField::InputPath | AudioSpeedField::OutputPath | AudioSpeedField::Run => {},
        }
    }

    /// The multiplier as the slider label prints it, e.g. "1.25x".
    pub fn speed_text(&self) -> String {
        format!("{:.2}x", self.speed as f64 / 100.0)
    }

    pub fn bitrate_ignored(&self) -> bool {
        self.format.is_lossless()
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Changing the tempo means touching every sample, so this always re-encodes,
    /// there is no stream-copy shortcut the way Trim has one.
    pub fn start_respeed(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let mut output = if typed_output.trim().is_empty() {
            default_output_path(&input, "respeed")
        } else {
            typed_output
        };
        force_extension(&mut output, self.format);

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-vn".into(),
            "-filter:a".into(), atempo_chain(self.speed),
        ];
        push_encode_args(&mut args, self.format, self.bitrate, None, None);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio speed"));
    }
}
