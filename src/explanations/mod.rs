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
