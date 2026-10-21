//                      <-- EXPLANATION MODULES -->
// Each operation (convert, compress, trim, merge, extraction) gets its own file here, and every
// one of those files exposes the same shape: `explain_intro_line_1`, `explain_intro_line_2`,
// `explain_outro_line`, plus whatever option lists that operation actually has (`explain_format_list`,
// and `explain_codec_list`/`explain_fps_list` for the ones with a second option type). Same names
// everywhere means `video.rs` can render any `ExplainX` screen with the same block of layout code,
// just swapping which functions get called.
//
// Since the function names collide across files (every one of them has an `explain_intro_line_1`),
// they can't all be `pub use`'d bare without stepping on each other. The `as` renames below are what
// let `video.rs` write `explanations::explain_trim_intro_line_1(...)` instead of importing each
// module separately and qualifying every call with its module path. That would be pretty cluttersome, bud...
pub mod convert_explanation;
pub mod compress_explanation;
pub mod trim_explanation;
pub mod merge_explanation;
pub mod extraction_explanation;
pub mod resize_explanation;
pub mod audio_convert_explanation;
pub mod audio_extract_explanation;
pub mod audio_trim_explanation;
pub mod audio_speed_explanation;
pub mod audio_fade_explanation;
pub mod audio_volume_explanation;
pub mod image_convert_explanation;
pub mod image_resize_explanation;
pub mod image_compress_explanation;
pub mod image_crop_explanation;
pub mod image_rotate_explanation;
pub mod image_filter_explanation;

pub use convert_explanation::{
    explain_intro_line_1 as explain_convert_intro_line_1,
    explain_intro_line_2 as explain_convert_intro_line_2,
    explain_outro_line as explain_convert_outro_line,
    explain_format_list as explain_convert_format_list,
    explain_codec_list as explain_convert_codec_list,
};
pub use compress_explanation::{
    explain_intro_line_1 as explain_compress_intro_line_1,
    explain_intro_line_2 as explain_compress_intro_line_2,
    explain_outro_line as explain_compress_outro_line,
    explain_format_list as explain_compress_format_list,
};
pub use trim_explanation::{
    explain_intro_line_1 as explain_trim_intro_line_1,
    explain_intro_line_2 as explain_trim_intro_line_2,
    explain_outro_line as explain_trim_outro_line,
    explain_format_list as explain_trim_format_list,
};
pub use merge_explanation::{
    explain_intro_line_1 as explain_merge_intro_line_1,
    explain_intro_line_2 as explain_merge_intro_line_2,
    explain_outro_line as explain_merge_outro_line,
    explain_format_list as explain_merge_format_list,
};
pub use extraction_explanation::{
    explain_intro_line_1 as explain_extraction_intro_line_1,
    explain_intro_line_2 as explain_extraction_intro_line_2,
    explain_outro_line as explain_extraction_outro_line,
    explain_format_list as explain_extraction_format_list,
    explain_fps_list as explain_extraction_fps_list,
};
pub use resize_explanation::{
    explain_intro_line_1 as explain_resize_intro_line_1,
    explain_intro_line_2 as explain_resize_intro_line_2,
    explain_outro_line as explain_resize_outro_line,
    explain_format_list as explain_resize_format_list,
};

// Audio screens follow the same renaming scheme, with `audio_` kept in the new
// name so an audio Convert/Trim guide can't be mistaken for the video one at the
// call site in `audio.rs`.
pub use audio_convert_explanation::{
    explain_intro_line_1 as explain_audio_convert_intro_line_1,
    explain_intro_line_2 as explain_audio_convert_intro_line_2,
    explain_outro_line as explain_audio_convert_outro_line,
    explain_format_list as explain_audio_convert_format_list,
    explain_bitrate_list as explain_audio_convert_bitrate_list,
};
pub use audio_extract_explanation::{
    explain_intro_line_1 as explain_audio_extract_intro_line_1,
    explain_intro_line_2 as explain_audio_extract_intro_line_2,
    explain_outro_line as explain_audio_extract_outro_line,
    explain_format_list as explain_audio_extract_format_list,
};
pub use audio_trim_explanation::{
    explain_intro_line_1 as explain_audio_trim_intro_line_1,
    explain_intro_line_2 as explain_audio_trim_intro_line_2,
    explain_outro_line as explain_audio_trim_outro_line,
    explain_format_list as explain_audio_trim_format_list,
};
pub use audio_speed_explanation::{
    explain_intro_line_1 as explain_audio_speed_intro_line_1,
    explain_intro_line_2 as explain_audio_speed_intro_line_2,
    explain_outro_line as explain_audio_speed_outro_line,
    explain_format_list as explain_audio_speed_format_list,
};
pub use audio_fade_explanation::{
    explain_intro_line_1 as explain_audio_fade_intro_line_1,
    explain_intro_line_2 as explain_audio_fade_intro_line_2,
    explain_outro_line as explain_audio_fade_outro_line,
    explain_format_list as explain_audio_fade_format_list,
};
pub use audio_volume_explanation::{
    explain_intro_line_1 as explain_audio_volume_intro_line_1,
    explain_intro_line_2 as explain_audio_volume_intro_line_2,
    explain_outro_line as explain_audio_volume_outro_line,
    explain_format_list as explain_audio_volume_format_list,
};

// Image screens use the same scheme, with `image_` kept in the new name.
pub use image_convert_explanation::{
    explain_intro_line_1 as explain_image_convert_intro_line_1,
    explain_intro_line_2 as explain_image_convert_intro_line_2,
    explain_outro_line as explain_image_convert_outro_line,
    explain_format_list as explain_image_convert_format_list,
};
pub use image_resize_explanation::{
    explain_intro_line_1 as explain_image_resize_intro_line_1,
    explain_intro_line_2 as explain_image_resize_intro_line_2,
    explain_outro_line as explain_image_resize_outro_line,
    explain_format_list as explain_image_resize_format_list,
};
pub use image_compress_explanation::{
    explain_intro_line_1 as explain_image_compress_intro_line_1,
    explain_intro_line_2 as explain_image_compress_intro_line_2,
    explain_outro_line as explain_image_compress_outro_line,
    explain_format_list as explain_image_compress_format_list,
};
pub use image_crop_explanation::{
    explain_intro_line_1 as explain_image_crop_intro_line_1,
    explain_intro_line_2 as explain_image_crop_intro_line_2,
    explain_outro_line as explain_image_crop_outro_line,
    explain_format_list as explain_image_crop_format_list,
};
pub use image_rotate_explanation::{
    explain_intro_line_1 as explain_image_rotate_intro_line_1,
    explain_intro_line_2 as explain_image_rotate_intro_line_2,
    explain_outro_line as explain_image_rotate_outro_line,
    explain_format_list as explain_image_rotate_format_list,
};
pub use image_filter_explanation::{
    explain_intro_line_1 as explain_image_filter_intro_line_1,
    explain_intro_line_2 as explain_image_filter_intro_line_2,
    explain_outro_line as explain_image_filter_outro_line,
    explain_format_list as explain_image_filter_format_list,
};
