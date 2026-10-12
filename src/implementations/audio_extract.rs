use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};
use super::audio_convert::{force_extension, push_encode_args, AudioBitrate, AudioFormat};

//                      <-- EXTRACT-ONLY OPTION TYPES -->
// Extraction is the one audio screen that can get away without encoding at all:
// if the video's existing audio track is already in a shape the chosen container
// accepts, lifting it out byte for byte is both instant and lossless.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractMode { Copy, Encode }

impl ExtractMode {
    pub const ALL: [ExtractMode; 2] = [ExtractMode::Copy, ExtractMode::Encode];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Copy => "⚡ copy", Self::Encode => "♻ re-encode",
        }
    }
}

//                      <-- EXTRACT FIELDS (lift the audio track off a video) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioExtractField { InputPath, OutputPath, Mode, Format, Bitrate, Run }

impl FieldSet for AudioExtractField {
    const ALL: &'static [AudioExtractField] = &[
        AudioExtractField::InputPath, AudioExtractField::OutputPath, AudioExtractField::Mode,
        AudioExtractField::Format, AudioExtractField::Bitrate, AudioExtractField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 video", Self::OutputPath => "💾 output",
            Self::Mode => "mode", Self::Format => "format",
            Self::Bitrate => "bitrate", Self::Run => "▶ extract",
        }
    }
}

pub type AudioExtractMenuState = MenuState<AudioExtractField>;

//                      <-- EXTRACT SCREEN STATE (lift the audio track off a video) -->
pub struct AudioExtractState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: AudioExtractMenuState,
    pub mode: ExtractMode,
    pub format: AudioFormat,
    pub bitrate: AudioBitrate,
    job: Option<FfmpegJob>,
}

impl AudioExtractState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/video.mp4"),
            output_file_path: new_path_field("(optional) C:/Users/you/audio.m4a"),
            menu: AudioExtractMenuState::new(),
            mode: ExtractMode::Copy,
            format: AudioFormat::M4a,
            bitrate: AudioBitrate::Auto,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            AudioExtractField::Mode => self.mode = cycle(&ExtractMode::ALL, self.mode, forward),
            AudioExtractField::Format => self.format = cycle(&AudioFormat::ALL, self.format, forward),
            AudioExtractField::Bitrate => self.bitrate = cycle(&AudioBitrate::ALL, self.bitrate, forward),
            AudioExtractField::InputPath | AudioExtractField::OutputPath | AudioExtractField::Run => {},
        }
    }

    /// In copy mode nothing is being encoded, so neither the bitrate nor the
    /// format's codec has any say in the result. The screen uses this to dim both
    /// fields rather than pretend they still matter.
    pub fn encode_options_ignored(&self) -> bool {
        self.mode == ExtractMode::Copy
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Pulls the audio stream out of a video. `-vn` drops the picture either way,
    /// then copy mode adds `-c:a copy` (instant, lossless, but only works when the
    /// container accepts the track's existing codec) while re-encode mode runs it
    /// through the picked format instead. A copy into the wrong container fails
    /// loudly in the log rather than silently producing a file nothing will play.
    pub fn start_extraction(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let blank_output = typed_output.trim().is_empty();
        let mut output = if blank_output {
            default_output_path(&input, "audio")
        } else {
            typed_output
        };

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input, "-vn".into()];

        match self.mode {
            ExtractMode::Copy => {
                // A typed extension is left alone, since copy never looks at the
                // format picker and renaming the file after it would be a lie. A
                // blank one, though, inherits the video's extension from
                // `default_output_path` (".mp4"), so the picker's extension is
                // stamped on instead to keep an audio-only file from posing as video.
                if blank_output {
                    force_extension(&mut output, self.format);
                }
                args.push("-c:a".into());
                args.push("copy".into());
            },
            ExtractMode::Encode => {
                force_extension(&mut output, self.format);
                push_encode_args(&mut args, self.format, self.bitrate, None, None);
            },
        }

        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio extract"));
    }
}
