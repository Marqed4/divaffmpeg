use ratatui_textarea::TextArea;

use super::shared::{
    cycle, new_path_field, strip_quotes, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState,
};
use super::image_convert::{push_keep_quality_args, resolve_output};

//                      <-- IMAGE FILTER FIELDS (one-pick looks) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFilterField { InputPath, OutputPath, Effect, Strength, Run }

impl FieldSet for ImageFilterField {
    const ALL: &'static [ImageFilterField] = &[
        ImageFilterField::InputPath, ImageFilterField::OutputPath,
        ImageFilterField::Effect, ImageFilterField::Strength, ImageFilterField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Effect => "effect", Self::Strength => "strength", Self::Run => "▶ apply",
        }
    }
}

pub type ImageFilterMenuState = MenuState<ImageFilterField>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageEffect { Grayscale, Sepia, Blur, Sharpen, Invert, Vignette }

impl ImageEffect {
    pub const ALL: [ImageEffect; 6] = [
        ImageEffect::Grayscale, ImageEffect::Sepia, ImageEffect::Blur,
        ImageEffect::Sharpen, ImageEffect::Invert, ImageEffect::Vignette,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Grayscale => "⚫ grayscale", Self::Sepia => "🟤 sepia", Self::Blur => "🌫 blur",
            Self::Sharpen => "🔪 sharpen", Self::Invert => "🔁 invert", Self::Vignette => "🎞 vignette",
        }
    }

    /// Invert is a hard on/off swap of every color, so a strength dial has nothing
    /// to scale. Every other effect has a natural "how much" axis.
    pub fn has_strength(&self) -> bool {
        !matches!(self, Self::Invert)
    }

    /// Builds the filter string at the given strength (0-100). Each effect maps the
    /// slider onto its own parameter, always so that 0 is barely-there and 100 is heavy:
    /// grayscale drains saturation, sepia blends from the untouched colors toward the
    /// full sepia matrix, blur widens the gaussian, sharpen raises the unsharp amount,
    /// and vignette narrows the lit circle (a smaller angle darkens more of the frame).
    pub fn filter(&self, strength: u8) -> String {
        let t = strength.min(STRENGTH_MAX) as f64 / STRENGTH_MAX as f64;
        match self {
            Self::Grayscale => format!("hue=s={:.3}", 1.0 - t),
            Self::Sepia => {
                const SEPIA: [f64; 9] = [0.393, 0.769, 0.189, 0.349, 0.686, 0.168, 0.272, 0.534, 0.131];
                const IDENTITY: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
                let m: Vec<f64> = SEPIA.iter().zip(IDENTITY).map(|(s, i)| i + (s - i) * t).collect();
                format!(
                    "colorchannelmixer={:.3}:{:.3}:{:.3}:0:{:.3}:{:.3}:{:.3}:0:{:.3}:{:.3}:{:.3}",
                    m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8],
                )
            },
            Self::Blur => format!("gblur=sigma={:.2}", 0.5 + t * 19.5),
            Self::Sharpen => format!("unsharp=5:5:{:.2}:5:5:0.0", 0.3 + t * 2.7),
            Self::Invert => "negate".to_string(),
            Self::Vignette => format!("vignette={:.4}", std::f64::consts::PI / (2.0 + t * 8.0)),
        }
    }
}

// Strength is a plain 0-100 percentage in steps of five, the same feel as the
// quality slider, so the label can just print it.
pub const STRENGTH_MAX: u8 = 100;
const STRENGTH_STEP: u8 = 5;

pub fn strength_ratio(strength: u8) -> f64 {
    strength.min(STRENGTH_MAX) as f64 / STRENGTH_MAX as f64
}

//                      <-- IMAGE FILTER SCREEN STATE -->
pub struct ImageFilterState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ImageFilterMenuState,
    pub effect: ImageEffect,
    pub strength: u8,
    job: Option<FfmpegJob>,
}

impl ImageFilterState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/photo.png"),
            output_file_path: new_path_field("(optional) C:/Users/you/photo_filtered.png"),
            menu: ImageFilterMenuState::new(),
            effect: ImageEffect::Grayscale,
            strength: 50,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ImageFilterField::Effect => self.effect = cycle(&ImageEffect::ALL, self.effect, forward),
            ImageFilterField::Strength => {
                self.strength = if forward {
                    self.strength.saturating_add(STRENGTH_STEP).min(STRENGTH_MAX)
                } else {
                    self.strength.saturating_sub(STRENGTH_STEP)
                };
            },
            ImageFilterField::InputPath | ImageFilterField::OutputPath | ImageFilterField::Run => {},
        }
    }

    pub fn strength_text(&self) -> String {
        format!("{}%", self.strength)
    }

    /// True while the strength slider points at an effect it can't scale, so the
    /// screen can say "n/a" instead of showing a number that never reaches ffmpeg.
    pub fn strength_ignored(&self) -> bool {
        !self.effect.has_strength()
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    pub fn start_filter(&mut self, log_path: &str) {
        let input = strip_quotes(&self.input_file_path.lines().join(""));
        if input.trim().is_empty() {
            return;
        }

        let output = resolve_output(&input, strip_quotes(&self.output_file_path.lines().join("")), "filtered");

        let mut args: Vec<String> = vec![
            "-y".into(), "-i".into(), input, "-frames:v".into(), "1".into(),
            "-vf".into(), self.effect.filter(self.strength),
        ];
        push_keep_quality_args(&mut args, &output);
        args.push(output);

        self.job = Some(spawn_ffmpeg_job(args, log_path, "image filter"));
    }
}
