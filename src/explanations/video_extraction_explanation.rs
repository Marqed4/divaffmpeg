use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Extraction pulls still frames out of a video and saves each one as an image, no re-encode of the clip itself required.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Set a start/end range to bound the clip, then choose how often a frame gets pulled and which image format to save it in:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG defaults to every frame, flip to a slower sampling rate if you just need a handful of thumbnails.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\".jpg\"{RESET}       : compressed, small files, minor quality loss\n\
                     {PINK}\".png\"{RESET}       : lossless, larger files, keeps transparency\n\
                     \".bmp\"        : uncompressed bitmap, largest files\n\
                     \".tiff\"       : lossless, common in editing/archival pipelines\n\
                     {PINK}\".webp\"{RESET}      : lossless or lossy, smaller than PNG at similar quality\n\
                     {PINK}\".gif\"{RESET}       : looping animated frames instead of one still per file"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap() // Be careful when you change this.
    )
}

pub fn explain_fps_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}every frame{RESET}   : one image per frame of the source, exact but can produce thousands of files\n\
                     1/sec         : one frame every second, good for a scrubbable thumbnail set\n\
                     1/2s, 1/5s, 1/10s : progressively sparser sampling for quick previews or storyboards"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
