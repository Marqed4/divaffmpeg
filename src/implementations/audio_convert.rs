use ratatui_textarea::TextArea;

use super::shared::{
    cycle, new_path_field, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};

//                      <-- AUDIO-WIDE OPTION TYPES -->
// These live here (not in `shared.rs`) because nothing on the video side can use
// them: every audio screen that needs a container, a bitrate, a sample rate, or a
// channel layout pulls them back out of this file. Convert is simply the screen
// that uses all four, so it owns them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat { Mp3, M4a, Opus, Ogg, Flac, Wav }

impl AudioFormat {
    pub const ALL: [AudioFormat; 6] = [
        AudioFormat::Mp3, AudioFormat::M4a, AudioFormat::Opus,
        AudioFormat::Ogg, AudioFormat::Flac, AudioFormat::Wav,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Mp3 => "🎧 .mp3", Self::M4a => "🍎 .m4a", Self::Opus => "🌐 .opus",
            Self::Ogg => ".ogg", Self::Flac => "💎 .flac", Self::Wav => "🗜 .wav",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3", Self::M4a => "m4a", Self::Opus => "opus",
            Self::Ogg => "ogg", Self::Flac => "flac", Self::Wav => "wav",
        }
    }

    /// Each audio container here only has one encoder worth reaching for, so the
    /// codec picker that Convert-the-video-screen needs has no twin over here:
    /// picking the format picks the codec.
    pub fn ffmpeg_codec(&self) -> &'static str {
        match self {
            Self::Mp3 => "libmp3lame", Self::M4a => "aac", Self::Opus => "libopus",
            Self::Ogg => "libvorbis", Self::Flac => "flac", Self::Wav => "pcm_s16le",
        }
    }

    /// FLAC and WAV store every sample as-is, so there is no bitrate to aim at.
    /// Passing `-b:a` to them is at best ignored and at worst an error, which is
    /// why every screen checks this before pushing the flag.
    pub fn is_lossless(&self) -> bool {
        matches!(self, Self::Flac | Self::Wav)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBitrate { Auto, K320, K256, K192, K128, K96, K64 }

impl AudioBitrate {
    pub const ALL: [AudioBitrate; 7] = [
        AudioBitrate::Auto, AudioBitrate::K320, AudioBitrate::K256, AudioBitrate::K192,
        AudioBitrate::K128, AudioBitrate::K96, AudioBitrate::K64,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "🔳 auto", Self::K320 => "320k", Self::K256 => "256k",
            Self::K192 => "192k", Self::K128 => "128k", Self::K96 => "96k", Self::K64 => "64k",
        }
    }

    /// `None` means "don't pass -b:a at all", which leaves the encoder on its own
    /// default instead of pinning it to a number the user never actually picked.
    pub fn value(&self) -> Option<&'static str> {
        match self {
            Self::Auto => None, Self::K320 => Some("320k"), Self::K256 => Some("256k"),
            Self::K192 => Some("192k"), Self::K128 => Some("128k"),
            Self::K96 => Some("96k"), Self::K64 => Some("64k"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSampleRate { Original, Hz48000, Hz44100, Hz32000, Hz22050 }

impl AudioSampleRate {
    pub const ALL: [AudioSampleRate; 5] = [
        AudioSampleRate::Original, AudioSampleRate::Hz48000, AudioSampleRate::Hz44100,
        AudioSampleRate::Hz32000, AudioSampleRate::Hz22050,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::Hz48000 => "48 kHz", Self::Hz44100 => "44.1 kHz",
            Self::Hz32000 => "32 kHz", Self::Hz22050 => "22 kHz",
        }
    }

    pub fn value(&self) -> Option<u32> {
        match self {
            Self::Original => None, Self::Hz48000 => Some(48000), Self::Hz44100 => Some(44100),
            Self::Hz32000 => Some(32000), Self::Hz22050 => Some(22050),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannels { Original, Stereo, Mono }

impl AudioChannels {
    pub const ALL: [AudioChannels; 3] = [
        AudioChannels::Original, AudioChannels::Stereo, AudioChannels::Mono,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::Stereo => "🎧 stereo", Self::Mono => "🔈 mono",
        }
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Original => None, Self::Stereo => Some(2), Self::Mono => Some(1),
        }
    }
}

/// Pushes the encode flags every audio screen ends up needing in the same order,
/// so a screen that only cares about one of them (Speed, Fade, Volume) still lands
/// on exactly the same arg shape Convert produces. Bitrate is skipped on lossless
/// containers, see `AudioFormat::is_lossless`.
pub fn push_encode_args(
    args: &mut Vec<String>,
    format: AudioFormat,
    bitrate: AudioBitrate,
    sample_rate: Option<AudioSampleRate>,
    channels: Option<AudioChannels>,
) {
    args.push("-c:a".into());
    args.push(format.ffmpeg_codec().into());

    if !format.is_lossless() {
        if let Some(rate) = bitrate.value() {
            args.push("-b:a".into());
            args.push(rate.into());
        }
    }
    if let Some(hz) = sample_rate.and_then(|s| s.value()) {
        args.push("-ar".into());
        args.push(hz.to_string());
    }
    if let Some(count) = channels.and_then(|c| c.value()) {
        args.push("-ac".into());
        args.push(count.to_string());
    }
}

/// Swaps whatever extension the user typed for the chosen format's own, the same
/// way the video Convert screen does, so the container and the filename can never
/// disagree about what the file actually is.
pub fn force_extension(output: &mut String, format: AudioFormat) {
    if let Some(dot) = output.rfind('.') {
        output.truncate(dot);
    }
    output.push('.');
    output.push_str(format.extension());
}

//                      <-- AUDIO CONVERT FIELDS (container + quality only) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioConvertField { InputPath, OutputPath, Format, Bitrate, SampleRate, Channels, Run }

impl FieldSet for AudioConvertField {
    const ALL: &'static [AudioConvertField] = &[
        AudioConvertField::InputPath, AudioConvertField::OutputPath, AudioConvertField::Format,
        AudioConvertField::Bitrate, AudioConvertField::SampleRate, AudioConvertField::Channels,
        AudioConvertField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Format => "format", Self::Bitrate => "bitrate",
            Self::SampleRate => "rate", Self::Channels => "channels",
            Self::Run => "▶ convert",
        }
    }
}

pub type AudioConvertMenuState = MenuState<AudioConvertField>;

//                      <-- AUDIO CONVERT SCREEN STATE (container + quality only) -->
pub struct AudioConvertState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: AudioConvertMenuState,
    pub format: AudioFormat,
    pub bitrate: AudioBitrate,
    pub sample_rate: AudioSampleRate,
    pub channels: AudioChannels,
    job: Option<FfmpegJob>,
}

impl AudioConvertState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/song.wav"),
            output_file_path: new_path_field("C:/Users/you/song.mp3"),
            menu: AudioConvertMenuState::new(),
            format: AudioFormat::Mp3,
            bitrate: AudioBitrate::Auto,
            sample_rate: AudioSampleRate::Original,
            channels: AudioChannels::Original,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            AudioConvertField::Format => self.format = cycle(&AudioFormat::ALL, self.format, forward),
            AudioConvertField::Bitrate => self.bitrate = cycle(&AudioBitrate::ALL, self.bitrate, forward),
            AudioConvertField::SampleRate => self.sample_rate = cycle(&AudioSampleRate::ALL, self.sample_rate, forward),
            AudioConvertField::Channels => self.channels = cycle(&AudioChannels::ALL, self.channels, forward),
            AudioConvertField::InputPath | AudioConvertField::OutputPath | AudioConvertField::Run => {},
        }
    }

    /// True while the bitrate picker is pointed at a container that ignores it, so
    /// the screen can grey the field out instead of letting the user cycle a number
    /// that will never reach ffmpeg.
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

    /// Audio convert only ever re-encodes the sound: `-vn` drops any video stream
    /// the input happened to carry, so feeding it an .mp4 yields a clean audio file
    /// rather than a container with a silent, orphaned video track still inside.
    pub fn start_convert(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        let mut output = strip_quotes(&self.output_file_path.lines().join(""));
        if input.trim().is_empty() || output.trim().is_empty() {
            return;
        }

        force_extension(&mut output, self.format);

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input, "-vn".into()];
        push_encode_args(&mut args, self.format, self.bitrate, Some(self.sample_rate), Some(self.channels));
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio convert"));
    }
}
