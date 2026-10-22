use ratatui_textarea::TextArea;
use chrono::Utc;

use super::shared::{new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState};

//                      <-- MERGE FIELDS (stitch two clips together) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoMergeField { InputA, InputB, OutputPath, Run }

impl FieldSet for VideoMergeField {
    const ALL: &'static [VideoMergeField] = &[
        VideoMergeField::InputA, VideoMergeField::InputB, VideoMergeField::OutputPath, VideoMergeField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputA => "📂 clip A", Self::InputB => "📂 clip B",
            Self::OutputPath => "💾 output", Self::Run => "▶ merge",
        }
    }
}

pub type MergeMenuState = MenuState<VideoMergeField>;

//                      <-- MERGE SCREEN STATE (stitch two clips together) -->
pub struct VideoMergeState {
    pub input_a_path: TextArea<'static>,
    pub input_b_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: MergeMenuState,
    job: Option<FfmpegJob>,
}

impl VideoMergeState {
    pub fn new() -> Self {
        Self {
            input_a_path: new_path_field("C:/Users/you/clip_a.mov"),
            input_b_path: new_path_field("C:/Users/you/clip_b.mov"),
            output_file_path: new_path_field("C:/Users/you/merged.mp4"),
            menu: MergeMenuState::new(),
            job: None,
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

    /// Merge only ever concatenates two clips back-to-back via ffmpeg's concat demuxer;
    /// it doesn't re-encode, trim, or change format/quality.
    pub fn start_merge(&mut self, log_path: &str) {
        let input_a = strip_quotes(&self.input_a_path.lines().join(""));
        let input_b = strip_quotes(&self.input_b_path.lines().join(""));
        let output = strip_quotes(&self.output_file_path.lines().join(""));
        if input_a.trim().is_empty() || input_b.trim().is_empty() || output.trim().is_empty() {
            return;
        }

        // The concat demuxer wants a list file with `file '<path>'` lines; single quotes
        // inside a path have to be escaped as '\'' or ffmpeg mis-parses the line.
        let escape = |p: &str| p.replace('\'', "'\\''");
        let list_contents = format!("file '{}'\nfile '{}'\n", escape(&input_a), escape(&input_b));
        let list_path = std::env::temp_dir().join(format!("diva_ffmpeg_merge_{}.txt", Utc::now().timestamp_millis()));
        if std::fs::write(&list_path, list_contents).is_err() {
            return;
        }

        let args: Vec<String> = vec![
            "-y".into(), "-f".into(), "concat".into(), "-safe".into(), "0".into(),
            "-i".into(), list_path.to_string_lossy().into_owned(),
            "-c".into(), "copy".into(), output,
        ];

        self.job = Some(spawn_ffmpeg_job(args, log_path, "merge"));
    }
}
