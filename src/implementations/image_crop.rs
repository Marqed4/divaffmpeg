use ratatui_textarea::TextArea;

use super::shared::{new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState};
use super::image_convert::{push_keep_quality_args, resolve_output};

//                      <-- IMAGE CROP FIELDS (cut out a rectangle) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropField { InputPath, OutputPath, X, Y, Width, Height, Run }

impl FieldSet for ImageCropField {
    const ALL: &'static [ImageCropField] = &[
        ImageCropField::InputPath, ImageCropField::OutputPath, ImageCropField::X,
        ImageCropField::Y, ImageCropField::Width, ImageCropField::Height, ImageCropField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::X => "x", Self::Y => "y", Self::Width => "width", Self::Height => "height",
            Self::Run => "▶ crop",
        }
    }
}

pub type ImageCropMenuState = MenuState<ImageCropField>;

fn parse_whole(text: &str) -> Option<u32> {
    text.trim().parse::<u32>().ok()
}

//                      <-- IMAGE CROP SCREEN STATE -->
pub struct ImageCropState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub x: TextArea<'static>,
    pub y: TextArea<'static>,
    pub width: TextArea<'static>,
    pub height: TextArea<'static>,
    pub menu: ImageCropMenuState,
    job: Option<FfmpegJob>,
}

impl ImageCropState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("(optional) C:/Users/you/photo_cropped.png"),
            x: new_path_field("0"),
            y: new_path_field("0"),
            width: new_path_field("640"),
            height: new_path_field("480"),
            menu: ImageCropMenuState::new(),
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

    /// Cuts a `width` x `height` rectangle whose top-left corner sits `x`,`y` pixels
    /// in from the picture's top-left. x/y may be blank (meaning 0) but the size
    /// may not, and a rectangle reaching past the edge is left for ffmpeg to report.
    pub fn start_crop(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let offset = |field: &TextArea<'static>| -> Option<u32> {
            let text = field.lines().join("");
            if text.trim().is_empty() { Some(0) } else { parse_whole(&text) }
        };
        let (Some(x), Some(y), Some(w), Some(h)) = (
            offset(&self.x),
            offset(&self.y),
            parse_whole(&self.width.lines().join("")).filter(|n| *n > 0),
            parse_whole(&self.height.lines().join("")).filter(|n| *n > 0),
        ) else {
            return;
        };

        let output = resolve_output(&input, strip_quotes(&self.output_file_path.lines().join("")), "cropped");

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-frames:v".into(), "1".into(),
            "-vf".into(), format!("crop={w}:{h}:{x}:{y}"),
        ];
        push_keep_quality_args(&mut args, &output);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image crop"));
    }
}
