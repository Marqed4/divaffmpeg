use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Merging stitches multiple video files together into one, provided they share compatible codecs, resolution, and frame rate.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG has two ways to join them, listed roughly most to least commonly reached for:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG reaches for the concat demuxer first, and only falls back to a re-encode when your files don't already match.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}concat demuxer{RESET}   : fast, lossless, ffmpeg -f concat -safe 0 -i list.txt -c copy out.mp4\n\
                     concat filter    : re-encode-based merge for mismatched codecs, resolutions, or frame rates\n\
                     codec mismatch   : Diva FFMPEG falls back to a re-encode automatically\n\
                     {PINK}\"-c copy\"{RESET}        : kept whenever possible so the merge stays lossless and fast"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
