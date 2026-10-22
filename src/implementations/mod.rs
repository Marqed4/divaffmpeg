pub mod shared;
pub mod video_convert;
pub mod video_compress;
pub mod video_trim;
pub mod video_merge;
pub mod video_extraction;
pub mod video_resize;

//                      <-- AUDIO SCREENS -->
// Same one-file-per-operation shape as the video screens above. They're prefixed
// `audio_` rather than tucked in a submodule so a `Convert`/`Trim` on the audio
// side can never be mistaken for the video one at the import site.
pub mod audio_convert;
pub mod audio_extract;
pub mod audio_trim;
pub mod audio_speed;
pub mod audio_fade;
pub mod audio_volume;

//                      <-- IMAGE SCREENS -->
// Same one-file-per-operation shape again, prefixed `image_` for the same reason.
pub mod image_convert;
pub mod image_resize;
pub mod image_compress;
pub mod image_crop;
pub mod image_rotate;
pub mod image_filter;

pub use shared::{FfmpegJob, FieldSet};

pub use video_convert::{VideoConvertField, VideoConvertState};
pub use video_compress::{VideoCompressField, VideoCompressState, crf_quality_label, crf_ratio};
pub use video_trim::{VideoTrimField, VideoTrimState};
pub use video_merge::{VideoMergeField, VideoMergeState};
pub use video_extraction::{VideoExtractionField, VideoExtractionState};
pub use video_resize::{VideoResizeField, VideoResizeState};

pub use audio_convert::{AudioConvertField, AudioConvertState};
pub use audio_extract::{AudioExtractField, AudioExtractState};
pub use audio_trim::{AudioTrimField, AudioTrimState};
pub use audio_speed::{AudioSpeedField, AudioSpeedState, speed_label, speed_ratio};
pub use audio_fade::{AudioFadeField, AudioFadeState};
pub use audio_volume::{AudioVolumeField, AudioVolumeState, gain_label, gain_ratio};

pub use image_convert::{ImageConvertField, ImageConvertState, quality_label, quality_ratio};
pub use image_resize::{ImageResizeField, ImageResizeState};
pub use image_compress::{ImageCompressField, ImageCompressState};
pub use image_crop::{ImageCropField, ImageCropState};
pub use image_rotate::{ImageRotateField, ImageRotateState};
pub use image_filter::{ImageFilterField, ImageFilterState, strength_ratio};
