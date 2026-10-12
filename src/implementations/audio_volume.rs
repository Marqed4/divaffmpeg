use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};
use super::audio_convert::{force_extension, push_encode_args, AudioBitrate, AudioFormat};

//                      <-- VOLUME FIELDS (push the whole track up or down) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioVolumeField { InputPath, OutputPath, Gain, Format, Bitrate, Run }

impl FieldSet for AudioVolumeField {
    const ALL: &'static [AudioVolumeField] = &[
        AudioVolumeField::InputPath, AudioVolumeField::OutputPath, AudioVolumeField::Gain,
        AudioVolumeField::Format, AudioVolumeField::Bitrate, AudioVolumeField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Gain => "gain", Self::Format => "format",
            Self::Bitrate => "bitrate", Self::Run => "▶ apply",
        }
    }
}

pub type AudioVolumeMenuState = MenuState<AudioVolumeField>;

// Gain is in whole decibels and signed, so it needs an i16 rather than the u8 the
// CRF slider gets away with. 0 is untouched, +6 is roughly twice as loud, -6 is
// roughly half. Past about +12 most already-mastered tracks start clipping, which
// is why the label starts warning well before the top of the range.
pub const GAIN_MIN: i16 = -30;
pub const GAIN_MAX: i16 = 30;
const GAIN_STEP: i16 = 1;

/// Plain-language read of a gain value, shown next to the slider so the decibel
/// number isn't the only thing telling the user what they're picking.
pub fn gain_label(db: i16) -> &'static str {
    match db {
        i16::MIN..=-21 => "🔇 nearly silent",
        -20..=-7 => "🔈 noticeably quieter",
        -6..=-1 => "🔉 gently pulled down",
        0 => "🎯 untouched",
        1..=6 => "🔊 gently pushed up",
        7..=12 => "📣 loud, watch for clipping",
        _ => "💥 very likely to clip",
    }
}

/// How full the slider bar should look: the bottom of the range empties it, the
/// top fills it, and 0 dB lands halfway across.
pub fn gain_ratio(db: i16) -> f64 {
    ((db - GAIN_MIN) as f64) / ((GAIN_MAX - GAIN_MIN) as f64)
}

//                      <-- VOLUME SCREEN STATE (push the whole track up or down) -->
pub struct AudioVolumeState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: AudioVolumeMenuState,
    pub gain: i16,
    pub format: AudioFormat,
    pub bitrate: AudioBitrate,
    job: Option<FfmpegJob>,
}

impl AudioVolumeState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/song.mp3"),
            output_file_path: new_path_field("(optional) C:/Users/you/song_louder.mp3"),
            menu: AudioVolumeMenuState::new(),
            gain: 0,
            format: AudioFormat::Mp3,
            bitrate: AudioBitrate::Auto,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            AudioVolumeField::Gain => {
                self.gain = if forward {
                    (self.gain + GAIN_STEP).min(GAIN_MAX)
                } else {
                    (self.gain - GAIN_STEP).max(GAIN_MIN)
                };
            },
            AudioVolumeField::Format => self.format = cycle(&AudioFormat::ALL, self.format, forward),
            AudioVolumeField::Bitrate => self.bitrate = cycle(&AudioBitrate::ALL, self.bitrate, forward),
            AudioVolumeField::InputPath | AudioVolumeField::OutputPath | AudioVolumeField::Run => {},
        }
    }

    /// The gain as the slider label prints it, signed so "+3 dB" reads differently
    /// from "-3 dB" at a glance.
    pub fn gain_text(&self) -> String {
        format!("{:+} dB", self.gain)
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

    /// Scales every sample by a fixed number of decibels. This is a flat offset,
    /// not loudness matching: a track that already swings between whispers and
    /// shouting still swings just as far afterwards, only higher or lower overall.
    pub fn start_volume(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let mut output = if typed_output.trim().is_empty() {
            default_output_path(&input, "volume")
        } else {
            typed_output
        };
        force_extension(&mut output, self.format);

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-vn".into(),
            "-filter:a".into(), format!("volume={}dB", self.gain),
        ];
        push_encode_args(&mut args, self.format, self.bitrate, None, None);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio volume"));
    }
}
