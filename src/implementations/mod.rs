pub mod shared;
pub mod convert;
pub mod compress;
pub mod trim;
pub mod merge;
pub mod extraction;
pub mod resize;

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

pub use shared::{FfmpegJob, FieldSet};

pub use convert::{ConvertField, ConvertState};
pub use compress::{CompressField, CompressState, crf_quality_label, crf_ratio};
pub use trim::{TrimField, TrimState};
pub use merge::{MergeField, MergeState};
pub use extraction::{ExtractionField, ExtractionState};
pub use resize::{ResizeField, ResizeState};

pub use audio_convert::{AudioConvertField, AudioConvertState};
pub use audio_extract::{AudioExtractField, AudioExtractState};
pub use audio_trim::{AudioTrimField, AudioTrimState};
pub use audio_speed::{AudioSpeedField, AudioSpeedState, speed_label, speed_ratio};
pub use audio_fade::{AudioFadeField, AudioFadeState};
pub use audio_volume::{AudioVolumeField, AudioVolumeState, gain_label, gain_ratio};
