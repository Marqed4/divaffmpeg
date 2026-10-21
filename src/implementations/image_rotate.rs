use ratatui_textarea::TextArea;

use super::shared::{
    cycle, new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState,
};
use super::image_convert::{push_keep_quality_args, resolve_output};

//                      <-- IMAGE ROTATE FIELDS (turn or mirror) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageRotateField { InputPath, OutputPath, Transform, Run }

impl FieldSet for ImageRotateField {
    const ALL: &'static [ImageRotateField] = &[
        ImageRotateField::InputPath, ImageRotateField::OutputPath,
        ImageRotateField::Transform, ImageRotateField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Transform => "turn", Self::Run => "▶ rotate",
        }
    }
}

pub type ImageRotateMenuState = MenuState<ImageRotateField>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageTransform { Clockwise, CounterClockwise, Half, FlipHorizontal, FlipVertical }

impl ImageTransform {
    pub const ALL: [ImageTransform; 5] = [
        ImageTransform::Clockwise, ImageTransform::CounterClockwise, ImageTransform::Half,
        ImageTransform::FlipHorizontal, ImageTransform::FlipVertical,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Clockwise => "↻ 90° right", Self::CounterClockwise => "↺ 90° left",
            Self::Half => "🔄 180°", Self::FlipHorizontal => "↔ mirror",
            Self::FlipVertical => "↕ flip",
        }
    }

    /// Every one of these is a lossless pixel shuffle, so there's no resampling
    /// blur the way an arbitrary-angle `rotate` would add.
    pub fn filter(&self) -> &'static str {
        match self {
            Self::Clockwise => "transpose=1", Self::CounterClockwise => "transpose=2",
            Self::Half => "hflip,vflip", Self::FlipHorizontal => "hflip",
            Self::FlipVertical => "vflip",
        }
    }
}

//                      <-- IMAGE ROTATE SCREEN STATE -->
pub struct ImageRotateState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ImageRotateMenuState,
    pub transform: ImageTransform,
    job: Option<FfmpegJob>,
}

impl ImageRotateState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("(optional) C:/Users/you/photo_rotated.png"),
            menu: ImageRotateMenuState::new(),
            transform: ImageTransform::Clockwise,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ImageRotateField::Transform => self.transform = cycle(&ImageTransform::ALL, self.transform, forward),
            ImageRotateField::InputPath | ImageRotateField::OutputPath | ImageRotateField::Run => {},
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

    pub fn start_rotate(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let output = resolve_output(&input, strip_quotes(&self.output_file_path.lines().join("")), "rotated");

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-frames:v".into(), "1".into(),
            "-vf".into(), self.transform.filter().into(),
        ];
        push_keep_quality_args(&mut args, &output);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image rotate"));
    }
}
