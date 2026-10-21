use ratatui_textarea::TextArea;

use super::shared::{new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState};
use super::image_convert::{push_keep_quality_args, resolve_output};

//                      <-- IMAGE RESIZE FIELDS (width x height, keep format) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageResizeField { InputPath, OutputPath, Width, Height, Run }

impl FieldSet for ImageResizeField {
    const ALL: &'static [ImageResizeField] = &[
        ImageResizeField::InputPath, ImageResizeField::OutputPath,
        ImageResizeField::Width, ImageResizeField::Height, ImageResizeField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Width => "↔ width", Self::Height => "↕ height", Self::Run => "▶ resize",
        }
    }
}

pub type ImageResizeMenuState = MenuState<ImageResizeField>;

/// Reads a dimension box. Blank means "work it out from the other side", which
/// ffmpeg's scale filter spells `-1`; anything that isn't a positive whole number
/// is rejected outright rather than guessed at.
fn parse_dimension(text: &str) -> Option<i64> {
    let text = text.trim();
    if text.is_empty() {
        return Some(-1);
    }
    text.parse::<i64>().ok().filter(|n| *n > 0)
}

//                      <-- IMAGE RESIZE SCREEN STATE -->
pub struct ImageResizeState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub width: TextArea<'static>,
    pub height: TextArea<'static>,
    pub menu: ImageResizeMenuState,
    job: Option<FfmpegJob>,
}

impl ImageResizeState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("(optional) C:/Users/you/photo_resized.png"),
            width: new_path_field("1280"),
            height: new_path_field("(blank = keep ratio)"),
            menu: ImageResizeMenuState::new(),
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

    /// Leaving one side blank lets ffmpeg derive it from the picture's own aspect
    /// ratio; filling both stretches to exactly that size. Both blank is a no-op.
    pub fn start_resize(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }
        let (Some(w), Some(h)) = (
            parse_dimension(&self.width.lines().join("")),
            parse_dimension(&self.height.lines().join("")),
        ) else {
            return;
        };
        if w == -1 && h == -1 {
            return;
        }

        let output = resolve_output(&input, strip_quotes(&self.output_file_path.lines().join("")), "resized");

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-frames:v".into(), "1".into(),
            "-vf".into(), format!("scale={w}:{h}"),
        ];
        push_keep_quality_args(&mut args, &output);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image resize"));
    }
}
