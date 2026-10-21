use ratatui_textarea::TextArea;

use super::shared::{
    cycle, new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState,
};
use super::image_convert::{
    force_image_extension, push_quality_args, resolve_output, step_quality, ImageFormat,
};

//                      <-- IMAGE COMPRESS FIELDS (shrink file size) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCompressField { InputPath, OutputPath, Quality, Format, Run }

impl FieldSet for ImageCompressField {
    const ALL: &'static [ImageCompressField] = &[
        ImageCompressField::InputPath, ImageCompressField::OutputPath,
        ImageCompressField::Quality, ImageCompressField::Format, ImageCompressField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Quality => "quality", Self::Format => "format", Self::Run => "▶ compress",
        }
    }
}

pub type ImageCompressMenuState = MenuState<ImageCompressField>;

//                      <-- IMAGE COMPRESS SCREEN STATE -->
pub struct ImageCompressState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ImageCompressMenuState,
    pub quality: u8,
    pub format: ImageFormat,
    job: Option<FfmpegJob>,
}

impl ImageCompressState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("(optional) C:/Users/you/photo_compressed.jpg"),
            menu: ImageCompressMenuState::new(),
            quality: 60,
            format: ImageFormat::Jpg,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ImageCompressField::Quality => self.quality = step_quality(self.quality, forward),
            ImageCompressField::Format => self.format = cycle(&ImageFormat::COMPRESS_ALL, self.format, forward),
            ImageCompressField::InputPath | ImageCompressField::OutputPath | ImageCompressField::Run => {},
        }
    }

    pub fn quality_text(&self) -> String {
        format!("{}%", self.quality)
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Re-encodes into a lossy format at the chosen quality. Only JPG and WebP are
    /// offered, since those are the formats where "quality" actually trades detail for size.
    pub fn start_compress(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let mut output = resolve_output(&input, strip_quotes(&self.output_file_path.lines().join("")), "compressed");
        force_image_extension(&mut output, self.format);

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input, "-frames:v".into(), "1".into()];
        push_quality_args(&mut args, self.format, self.quality);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image compress"));
    }
}
