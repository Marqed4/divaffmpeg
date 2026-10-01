use ratatui_textarea::TextArea;

use super::shared::{
    cycle, new_path_field, strip_quotes, spawn_ffmpeg_job,
    ConvertCodec, ConvertFps, FfmpegJob, FieldSet, MenuState,
};

//                      <-- CONVERT-ONLY OPTION TYPES -->
// Convert is the only screen that changes container (format) or resolution;
// Compress/Trim/Merge never touch either of these.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertFormat { Mp4, Mkv, WebM, Mov, Avi, Ts, Mpd, Mxf, Flv, Gp3, Mpeg, Wmv, Gif }

impl ConvertFormat {
    const ALL: [ConvertFormat; 13] = [
        ConvertFormat::Mp4, ConvertFormat::Mkv, ConvertFormat::WebM, ConvertFormat::Mov,
        ConvertFormat::Avi, ConvertFormat::Ts, ConvertFormat::Mpd, ConvertFormat::Mxf,
        ConvertFormat::Flv, ConvertFormat::Gp3, ConvertFormat::Mpeg, ConvertFormat::Wmv,
        ConvertFormat::Gif,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Mp4 => "🎬 .mp4", Self::Mkv => ".mkv", Self::WebM => "🌐 .webm",
            Self::Mov => ".mov", Self::Avi => ".avi", Self::Ts => ".ts",
            Self::Mpd => ".mpd", Self::Mxf => ".mxf", Self::Flv => ".flv",
            Self::Gp3 => "📱 .3gp", Self::Mpeg => ".mpeg", Self::Wmv => ".wmv",
            Self::Gif => "🖼 .gif",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4", Self::Mkv => "mkv", Self::WebM => "webm", Self::Mov => "mov",
            Self::Avi => "avi", Self::Ts => "ts", Self::Mpd => "mpd", Self::Mxf => "mxf",
            Self::Flv => "flv", Self::Gp3 => "3gp", Self::Mpeg => "mpeg", Self::Wmv => "wmv",
            Self::Gif => "gif",
        }
    }
}

impl ConvertCodec {
    /// Codecs a container can actually mux. WebM only takes VP8/VP9/AV1;
    /// the .gif muxer only takes the `gif` codec. h264/h265 fail there with
    /// "Could not write header (incorrect codec parameters ?): Invalid argument".
    fn allowed_for(format: ConvertFormat) -> &'static [ConvertCodec] {
        match format {
            ConvertFormat::WebM => &[ConvertCodec::Vp9],
            ConvertFormat::Gif => &[ConvertCodec::Gif],
            _ => &[ConvertCodec::H264, ConvertCodec::H265],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertResolution { Original, R4K, R1440, R1080, R720, R480, R360 }

impl ConvertResolution {
    const ALL: [ConvertResolution; 7] = [
        ConvertResolution::Original, ConvertResolution::R4K, ConvertResolution::R1440,
        ConvertResolution::R1080, ConvertResolution::R720, ConvertResolution::R480, ConvertResolution::R360,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::R4K => "4k", Self::R1440 => "1440p",
            Self::R1080 => "1080p", Self::R720 => "720p", Self::R480 => "480p", Self::R360 => "360p",
        }
    }

    fn scale(&self) -> Option<(u32, u32)> {
        match self {
            Self::Original  => None,
            Self::R4K       => Some((3840 , 2160)),
            Self::R1440     => Some((2560, 1440)),
            Self::R1080     => Some((1920, 1080)),
            Self::R720      => Some((1280, 720)),
            Self::R480      => Some((854, 480)),
            Self::R360      => Some((640, 360)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCrf { Auto, High, VeryHigh, Medium, Low, Trash, Meme}

impl ConvertCrf {
    const ALL: [ConvertCrf; 7] = [ConvertCrf::Auto, ConvertCrf::High, ConvertCrf::VeryHigh,
    ConvertCrf::Medium, ConvertCrf::Low, ConvertCrf::Trash, ConvertCrf::Meme];

    fn value(&self) -> Option<u8> {
        match self {
            Self::Auto => None, Self::High => Some(18),  Self::VeryHigh => Some(20), Self::Medium => Some(24),
            Self::Low => Some(27), Self::Trash => Some(30), Self::Meme => Some(34)
        }
    }
}

//                      <-- CONVERT FIELDS (format + quality only) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertField { InputPath, OutputPath, Format, Codec, Resolution, Fps, Crf, Run }

impl FieldSet for ConvertField {
    const ALL: &'static [ConvertField] = &[
        ConvertField::InputPath, ConvertField::OutputPath, ConvertField::Format,
        ConvertField::Codec, ConvertField::Resolution, ConvertField::Fps,
        ConvertField::Crf, ConvertField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::Format => "format", Self::Codec => "codec",
            Self::Resolution => "res", Self::Fps => "fps", Self::Crf => "quality",
            Self::OutputPath => "💾 output", Self::Run => "▶ convert",
        }
    }
}

pub type ConvertMenuState = MenuState<ConvertField>;

//                      <-- CONVERT SCREEN STATE (format + quality only) -->
pub struct ConvertState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ConvertMenuState,
    pub format: ConvertFormat,
    pub codec: ConvertCodec,
    pub resolution: ConvertResolution,
    pub fps: ConvertFps,
    pub crf: ConvertCrf,
    job: Option<FfmpegJob>,
}

impl ConvertState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/video.mov"),
            output_file_path: new_path_field("C:/Users/you/output.mp4"),
            menu: ConvertMenuState::new(),
            format: ConvertFormat::Mp4,
            codec: ConvertCodec::H264,
            resolution: ConvertResolution::Original,
            fps: ConvertFps::Original,
            crf: ConvertCrf::Auto,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ConvertField::Format => {
                self.format = cycle(&ConvertFormat::ALL, self.format, forward);
                // Drop to a codec the new container can actually mux (e.g. leaving
                // .webm/.gif) so the picker never lands on a combo ffmpeg will reject.
                let allowed = ConvertCodec::allowed_for(self.format);
                if !allowed.contains(&self.codec) {
                    self.codec = allowed[0];
                }
            },
            ConvertField::Codec => {
                let allowed = ConvertCodec::allowed_for(self.format);
                self.codec = cycle(allowed, self.codec, forward);
            },
            ConvertField::Resolution => self.resolution = cycle(&ConvertResolution::ALL, self.resolution, forward),
            ConvertField::Fps => self.fps = cycle(&ConvertFps::ALL, self.fps, forward),
            ConvertField::Crf => self.crf = cycle(&ConvertCrf::ALL, self.crf, forward),
            ConvertField::InputPath | ConvertField::OutputPath | ConvertField::Run => {},
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

    /// Convert only ever changes container/quality, never trims or joins footage.
    pub fn start_convert(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        let mut output = strip_quotes(&self.output_file_path.lines().join(""));
        if input.trim().is_empty() || output.trim().is_empty() {
            return;
        }

        // Swap in the chosen format's extension, replacing whatever the user typed (if any).
        if let Some(dot) = output.rfind('.') {
            output.truncate(dot);
        }
        output.push('.');
        output.push_str(self.format.extension());

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input];

        if let Some((w, h)) = self.resolution.scale() {
            args.push("-vf".into());
            args.push(format!("scale={w}:{h}"));
        }
        if let Some(fps) = self.fps.value() {
            args.push("-r".into());
            args.push(fps.to_string());
        }
        args.push("-c:v".into());
        args.push(self.codec.ffmpeg_flag().into());
        // The `gif` codec doesn't take -crf; ffmpeg rejects the flag outright.
        if self.codec != ConvertCodec::Gif {
            if let Some(crf) = self.crf.value() {
                args.push("-crf".into());
                args.push(crf.to_string());
            }
        }
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "convert"));
    }
}
