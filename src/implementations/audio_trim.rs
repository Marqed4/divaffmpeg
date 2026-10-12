use ratatui_textarea::TextArea;

use super::shared::{default_output_path, format_timestamp, new_path_field, parse_timestamp,
    probe_duration, spawn_ffmpeg_job, strip_quotes, FfmpegJob, FieldSet, MenuState};

//                      <-- AUDIO TRIM FIELDS (cut a range, no re-encode) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioTrimField { InputPath, OutputPath, Start, End, Run }

impl FieldSet for AudioTrimField {
    const ALL: &'static [AudioTrimField] = &[
        AudioTrimField::InputPath, AudioTrimField::OutputPath,
        AudioTrimField::Start, AudioTrimField::End, AudioTrimField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Start => "⏱ start", Self::End => "⏱ end", Self::Run => "▶ trim",
        }
    }
}

pub type AudioTrimMenuState = MenuState<AudioTrimField>;

//                      <-- AUDIO TRIM SCREEN STATE (cut a range, no re-encode) -->
pub struct AudioTrimState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub start_time: TextArea<'static>,
    pub end_time: TextArea<'static>,
    pub menu: AudioTrimMenuState,
    /// The track's real length, learned from `sync_duration`. `end_time` is
    /// clamped to this so a typed/leftover end can never point past the track.
    duration: Option<f64>,
    job: Option<FfmpegJob>,
}

// Same deal as the video Trim screen: the length is read off the file itself so the
// user never has to go look it up, and `start_trim` clamps to it afterwards. Audio
// keyframes sit far closer together than video ones, so a stream copy here lands much
// nearer the requested timestamps than the same copy would on a video track.
impl AudioTrimState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/song.mp3"),
            output_file_path: new_path_field("(optional) C:/Users/you/song_trimmed.mp3"),
            start_time: new_path_field("00:00:00"),
            end_time: new_path_field("00:00:10"),
            menu: AudioTrimMenuState::new(),
            duration: None,
            job: None,
        }
    }

    /// Probes the input track's real length and sets `end_time` to it, so the
    /// field starts at the furthest end point actually reachable in this file.
    /// Called once the user finishes typing/confirming the input path.
    pub fn sync_duration(&mut self) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        if let Some(duration) = probe_duration(&input) {
            self.duration = Some(duration);
            self.end_time = TextArea::new(vec![format_timestamp(duration)]);
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

    /// Trim only ever cuts a time range: stream-copied, no re-encode, no format change.
    pub fn start_trim(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        let start = self.start_time.lines().join("");
        let mut end = self.end_time.lines().join("");
        if input.trim().is_empty() || start.trim().is_empty() || end.trim().is_empty() {
            return;
        }

        // Never let a typed (or stale) end point past the track's real length.
        if let Some(duration) = self.duration {
            if parse_timestamp(&end).is_none_or(|secs| secs > duration) {
                end = format_timestamp(duration);
            }
        }

        let typed_output = strip_quotes(&self.output_file_path.lines().join(""));
        let output = if typed_output.trim().is_empty() {
            default_output_path(&input, "trimmed")
        } else {
            typed_output
        };

        // -ss/-to before -i: fast seek on the input side, -c copy skips re-encoding
        // entirely, and -vn drops cover art so an .mp3 with embedded artwork doesn't
        // make ffmpeg try (and fail) to copy a one-frame video stream alongside it.
        let args: Vec<String> = vec![
            "-y".into(), "-ss".into(), start, "-to".into(), end,
            "-i".into(), input, "-vn".into(), "-c:a".into(), "copy".into(), output,
        ];

        self.job = Some(spawn_ffmpeg_job(args, log_path, "audio trim"));
    }
}
