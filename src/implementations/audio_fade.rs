use ratatui_textarea::TextArea;

use super::shared::{
    cycle, default_output_path, new_path_field, probe_duration, strip_quotes, spawn_ffmpeg_job,
    FfmpegJob, FieldSet, MenuState,
};
use super::audio_convert::{force_extension, push_encode_args, AudioBitrate, AudioFormat};

//                      <-- FADE FIELDS (ease the start and end in and out) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFadeField { InputPath, OutputPath, FadeIn, FadeOut, Format, Run }

impl FieldSet for AudioFadeField {
    const ALL: &'static [AudioFadeField] = &[
        AudioFadeField::InputPath, AudioFadeField::OutputPath, AudioFadeField::FadeIn,
        AudioFadeField::FadeOut, AudioFadeField::Format, AudioFadeField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::FadeIn => "🌅 fade in", Self::FadeOut => "🌇 fade out",
            Self::Format => "format", Self::Run => "▶ fade",
        }
    }
}

pub type AudioFadeMenuState = MenuState<AudioFadeField>;

/// Reads a fade length typed as plain seconds ("3", "2.5"). Anything that isn't a
/// non-negative number is treated as no fade at all, so a half-typed field can
/// never turn into a filter argument ffmpeg will choke on.
fn parse_seconds(text: &str) -> f64 {
    text.trim().parse::<f64>().ok().filter(|v| *v >= 0.0).unwrap_or(0.0)
}

//                      <-- FADE SCREEN STATE (ease the start and end in and out) -->
pub struct AudioFadeState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub fade_in: TextArea<'static>,
    pub fade_out: TextArea<'static>,
    pub menu: AudioFadeMenuState,
    pub format: AudioFormat,
    /// The track's real length, learned from `sync_duration`. A fade-out has to
    /// know where the end is before it can start climbing down towards it, so
    /// without this the fade-out field simply can't be honoured.
    duration: Option<f64>,
    job: Option<FfmpegJob>,
}

impl AudioFadeState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/song.mp3"),
            output_file_path: new_path_field("(optional) C:/Users/you/song_faded.mp3"),
            fade_in: new_path_field("3"),
            fade_out: new_path_field("3"),
            menu: AudioFadeMenuState::new(),
            format: AudioFormat::Mp3,
            duration: None,
            job: None,
        }
    }

    /// Probes the input track's length, which is what the fade-out start time is
    /// measured back from. Called once the user finishes typing/confirming the path.
    pub fn sync_duration(&mut self) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        if let Some(duration) = probe_duration(&input) {
            self.duration = Some(duration);
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            AudioFadeField::Format => self.format = cycle(&AudioFormat::ALL, self.format, forward),
            AudioFadeField::InputPath | AudioFadeField::OutputPath
                | AudioFadeField::FadeIn | AudioFadeField::FadeOut | AudioFadeField::Run => {},
        }
    }

    /// Whether a fade-out can actually be applied yet. Shown on the screen so a
    /// user who typed a fade-out before the length was known isn't left wondering
    /// why only the fade-in landed.
    pub fn duration_known(&self) -> bool {
        self.duration.is_some()
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Applies `afade` at one or both ends of the track.
    ///
    /// The fade-in is easy: it starts at zero and lasts however many seconds were
    /// typed. The fade-out is the fiddly one, because `afade=t=out` needs a start
    /// time (`st=`) rather than "however long is left", so the length has to be
    /// known and the start worked back from it. Both fades are clamped so they
    /// can't overrun the track or each other, which would otherwise hand ffmpeg a
    /// negative start time.
    pub fn start_fade(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let fade_in = parse_seconds(&self.fade_in.lines().join(""));
        let fade_out = parse_seconds(&self.fade_out.lines().join(""));

        let mut filters: Vec<String> = Vec::new();
        if fade_in > 0.0 {
            filters.push(format!("afade=t=in:st=0:d={fade_in}"));
        }
        if fade_out > 0.0 {
            if let Some(duration) = self.duration {
                // Keep the two fades from overlapping: the fade-out can only have
                // whatever the fade-in didn't already take.
                let room = (duration - fade_in).max(0.0);
                let length = fade_out.min(room);
                if length > 0.0 {
                    let start = duration - length;
                    filters.push(format!("afade=t=out:st={start}:d={length}"));
                }
            }
        }

        // Nothing to apply means nothing to run: without this an empty -filter:a
        // would be handed to ffmpeg, and the user would get a pointless re-encode
        // of an unchanged file.
        if filters.is_empty() {
            return;
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let mut output = if typed_output.trim().is_empty() {
            default_output_path(&input, "faded")
        } else {
            typed_output
        };
        force_extension(&mut output, self.format);

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-vn".into(),
            "-filter:a".into(), filters.join(","),
        ];
        push_encode_args(&mut args, self.format, AudioBitrate::Auto, None, None);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio fade"));
    }
}
